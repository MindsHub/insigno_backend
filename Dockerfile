# Leveraging the pre-built Docker images with 
# cargo-chef and the Rust toolchain
#
#base image to use to compile for each architecture

FROM --platform=$BUILDPLATFORM ghcr.io/cross-rs/x86_64-unknown-linux-musl AS build-amd64
FROM --platform=$BUILDPLATFORM ghcr.io/cross-rs/arm-unknown-linux-musleabihf AS build-armv6
FROM --platform=$BUILDPLATFORM ghcr.io/cross-rs/armv7-unknown-linux-musleabihf AS build-armv7
FROM --platform=$BUILDPLATFORM ghcr.io/cross-rs/aarch64-unknown-linux-musl AS build-arm64
FROM --platform=$BUILDPLATFORM rust AS rust
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
ARG RUSTUP_HOME=/root/.rustup
WORKDIR /app
#install rustup and cargo
COPY --from=rust /usr/local/cargo /root/.cargo
COPY --from=rust /usr/local/rustup /root/.rustup
#install cargo-chef
RUN curl -OL https://github.com/LukeMathWalker/cargo-chef/releases/download/v0.1.67/cargo-chef-x86_64-unknown-linux-musl.tar.gz ; tar xf ./cargo-chef-x86_64-unknown-linux-musl.tar.gz ; mv ./cargo-chef /root/.cargo/bin/cargo-chef
ENV PATH=$PATH:/root/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin:/root/.cargo/bin

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
#RUN cd .. ; \
#    git clone --recurse-submodules -j8 https://github.com/sgrif/pq-sys.git ; \
#    sed -i '/#define PG_INT128_TYPE __int128/d' ./pq-sys/pq-src/additional_include/pg_config.h

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
WORKDIR /app
ENV ROCKET_ADDRESS=0.0.0.0
EXPOSE 8000
COPY ./templates /app/templates
COPY ./static /app/static
COPY ./Rocket.toml /app/Rocket.toml
#RUN apk update
#RUN apk upgrade
#RUN apk add --no-cache ffmpeg
COPY --from=builder /insigno /app/insigno
ENTRYPOINT ["/app/insigno"]

