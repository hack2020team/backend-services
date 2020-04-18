FROM rust:1.42-buster as builder

WORKDIR /usr/src/backend-services

RUN rustup component add rustfmt --toolchain 1.42.0-x86_64-unknown-linux-gnu

COPY . .

RUN cargo build --release
RUN cargo install --path ./messaging

FROM debian:buster-slim

RUN apt-get update && apt-get -y install libssl-dev

COPY --from=builder /usr/local/cargo/bin/messaging /usr/local/bin/messaging

CMD ["messaging"]

