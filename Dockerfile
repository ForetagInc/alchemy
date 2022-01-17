FROM rust:latest as build

RUN user=root cargo new --bin alchemy
WORKDIR /alchemy

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

RUN cargo build --release && src/*.rs

COPY ./src ./src

RUN rm ./target/release/deps/alchemy*
RUN cargo build --release

FROM rust:1.58.0-slim-buster

COPY -from=build /alchemy/target/release/alchemy ./
CMD ["./alchemy"]
