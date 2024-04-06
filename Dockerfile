#!/usr/bin/env docker build . -t satoqz.net/rapla-proxy:latest -f

FROM rust:alpine as builder

RUN apk add --no-cache musl-dev

WORKDIR /build
COPY . .

RUN cargo fetch --locked
RUN cargo build --frozen --release --package rapla-proxy


FROM scratch 

COPY --from=builder /build/target/release/rapla-proxy /
ENV PATH /

ENV RAPLA_PROXY_ADDR 0.0.0.0:8080
EXPOSE 8080

CMD ["rapla-proxy"]
