FROM ekidd/rust-musl-builder:latest AS builder

ADD --chown=rust:rust Cargo.toml .
ADD --chown=rust:rust Cargo.lock .
ADD --chown=rust:rust src ./src

RUN cargo build --release

FROM alpine AS runner

RUN apk --no-cache add openssl

COPY --from=builder /home/rust/src/target/x86_64-unknown-linux-musl/release/rapla-to-ics /usr/local/bin

CMD ["rapla-to-ics"]
