#!/usr/bin/env docker build . -t satoqz.net/rapla-proxy:latest -f

FROM rust:alpine as builder

RUN apk add --no-cache musl-dev

WORKDIR /build

COPY Cargo.toml Cargo.lock ./
COPY rapla-parser rapla-parser
COPY rapla-proxy rapla-proxy

RUN cargo fetch --locked
RUN cargo install --locked --path rapla-proxy


FROM scratch 

COPY --from=builder /usr/local/cargo/bin/rapla-proxy /
ENV PATH /

ENV IP 0.0.0.0
EXPOSE 8080

CMD ["rapla-proxy"]
