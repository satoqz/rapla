#!/usr/bin/env docker build . -t satoqz.net/rapla-proxy:latest -f

FROM rust:alpine AS chef

WORKDIR /build
RUN apk add --no-cache musl-dev
RUN cargo install cargo-chef


FROM chef AS planner

COPY . .
RUN cargo chef prepare --bin rapla-proxy


FROM chef AS builder

COPY --from=planner /build/recipe.json .
RUN cargo chef cook --locked --release --bin rapla-proxy

COPY . .
RUN cargo build --frozen --release --bin rapla-proxy


FROM scratch 

COPY --from=builder /build/target/release/rapla-proxy /

ENV PATH=/
ENV RAPLA_PROXY_ADDR=0.0.0.0:8080

EXPOSE 8080

CMD ["rapla-proxy"]
