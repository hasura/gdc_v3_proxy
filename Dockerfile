FROM rust:1.69.0 as builder
ARG PACKAGE
WORKDIR /tmp
COPY Cargo.* .
COPY src src
RUN cargo build --locked --profile release --package gdc_v3_proxy
CMD ["/tmp/target/release/gdc_v3_proxy", "--base-url", "https://connector-24eac64f-4351-42e9-9e78-b157e70d4489-hyc5v23h6a-ue.a.run.app"]

