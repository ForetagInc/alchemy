# Build
FROM rust:latest as build

RUN user=root cargo new --bin alchemy
WORKDIR /alchemy

# Copy manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# Switch to nightly version for build
RUN rustup default nightly

# Cache dependencies for build
RUN cargo +nightly build --release 
RUN rm src/*.rs

COPY . .

RUN rm ./target/release/deps/alchemy*
RUN cargo +nightly build --release

# Run binary
FROM alpine:latest

COPY --from=build /alchemy/target/release/alchemy ./
CMD ["./alchemy"]
