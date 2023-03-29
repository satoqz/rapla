FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev

ADD Cargo.toml .
ADD Cargo.lock .
ADD src ./src

RUN cargo fetch --locked
RUN cargo install --path .


FROM scratch AS runner

COPY --from=builder /usr/local/cargo/bin/rapla /

EXPOSE 8080

CMD ["/rapla"]