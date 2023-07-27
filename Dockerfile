FROM rust:1.69.0 as builder
ARG PACKAGE
WORKDIR /tmp
COPY Cargo.* .
COPY src src
RUN cargo build --locked --profile release --package gdc_v3_proxy

FROM debian:buster-slim
ARG PACKAGE
RUN apt-get update & apt-get install -y extra-runtime-dependencies & rm -rf /var/lib/apt/lists/*
COPY  --from=builder /tmp/target/release/gdc_v3_proxy /usr/local/bin/gdc_v3_proxy
CMD ["gdc_v3_proxy", "--base-url", "https://connector-24eac64f-4351-42e9-9e78-b157e70d4489-hyc5v23h6a-ue.a.run.app"]
