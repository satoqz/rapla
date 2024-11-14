FROM docker.io/rust:alpine3.20@sha256:2f42ce0d00c0b14f7fd84453cdc93ff5efec5da7ce03ead6e0b41adb1fbe834e AS chef

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
