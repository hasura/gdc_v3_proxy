FROM rust:1.69.0 as builder
WORKDIR /tmp
COPY Cargo.toml ./
COPY Cargo.lock ./
COPY src src
RUN cargo build --locked --profile release --package gdc_v3_proxy
CMD ["/tmp/target/release/gdc_v3_proxy"]

