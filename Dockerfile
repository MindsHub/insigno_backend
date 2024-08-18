# Leveraging the pre-built Docker images with 
# cargo-chef and the Rust toolchain
#
#base image to use to compile for each architecture

FROM --platform=$BUILDPLATFORM ghcr.io/cross-rs/x86_64-unknown-linux-musl AS build-amd64
FROM --platform=$BUILDPLATFORM ghcr.io/cross-rs/arm-unknown-linux-musleabihf AS build-armv6
FROM --platform=$BUILDPLATFORM ghcr.io/cross-rs/armv7-unknown-linux-musleabihf AS build-armv7
FROM --platform=$BUILDPLATFORM ghcr.io/cross-rs/aarch64-unknown-linux-musl AS build-arm64
#FROM --platform=$BUILDPLATFORM ghcr.io/cross-rs/aarch64-unknown-linux-musl AS build-386
#FROM --platform=$BUILDPLATFORM ghcr.io/cross-rs/aarch64-unknown-linux-musl AS build-riscv64

FROM build-$TARGETARCH$TARGETVARIANT AS chef
#some cross env

ARG PKG_CONFIG_ALLOW_CROSS=1
ARG CARGO_HOME=/root/.cargo
ARG CROSS_RUNNER=
ARG TERM
ARG USER=root
ARG BUILDPLATFORM
ARG TARGETVARIANT
ARG TARGETPLATFORM
ARG TARGETARCH

WORKDIR /app
#install rustup
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
#env commands don't work, we need to do it by ourself
ENV PATH=$PATH:/root/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:/root/.cargo/bin





# install binstall not needed, because binstall cargo chef selects the wrong executable
#RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
#RUN curl -O http://archive.ubuntu.com/ubuntu/pool/main/g/glibc/glibc-doc_2.40-1ubuntu1_all.deb ; apt-get install -y ./glibc-doc_2.40-1ubuntu1_all.deb

#install cargo chef
RUN cargo install cargo-chef
# get the scrypt to translate the architectures
RUN echo "case \"${TARGETPLATFORM}\" in \n\
    \"linux/amd64\") echo \"x86_64-unknown-linux-musl\"\n\
    ;;\n\
    \"linux/arm/v6\") echo \"arm-unknown-linux-musleabihf\"\n\
    ;;\n\
    \"linux/arm/v7\") echo \"armv7-unknown-linux-musleabihf\"\n\
    ;;\n\
    \"linux/arm64\") echo \"aarch64-unknown-linux-musl\"\n\
esac" > ./get_target.sh

#add compilation target
RUN rustup target add $(sh ./get_target.sh)
# clone pq-sys, and correct a little bug
RUN cd .. ; \
    git clone --recurse-submodules -j8 https://github.com/sgrif/pq-sys.git ; \
    sed -i '/#define PG_INT128_TYPE __int128/d' ./pq-sys/pq-src/additional_include/pg_config.h

#plan what it should be built
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json --bin insigno

#build
FROM  chef AS builder

COPY --from=planner /app/recipe.json recipe.json 

# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --target $(sh ./get_target.sh)  --recipe-path recipe.json
COPY . .
RUN cargo build --release --target $(sh ./get_target.sh) --bin insigno
RUN cd /app/target/$(sh ./get_target.sh)/release/ ; ls -la
RUN mv /app/target/$(sh ./get_target.sh)/release/insigno /insigno


# We do not need the Rust toolchain to run the binary!
FROM alpine:latest AS runtime
WORKDIR /insigno
COPY --from=builder /insigno /app/insigno
ENTRYPOINT ["/app/insigno"]

