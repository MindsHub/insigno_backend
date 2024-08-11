# Leveraging the pre-built Docker images with 
# cargo-chef and the Rust toolchain
FROM rust:alpine AS chef
WORKDIR /app

RUN apk add --no-cache curl postgresql-dev openssl-dev openssl-libs-static

RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | ash
RUN cargo binstall cargo-chef -y


FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json


# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .

RUN cargo build --release

# We do not need the Rust toolchain to run the binary!
FROM alpine:latest AS runtime
WORKDIR /insigno

COPY --from=builder /app/target/release/insigno /insigno

ENTRYPOINT ["/insigno/insigno"]

