FROM rust:alpine AS builder

RUN apk add --no-cache musl-dev

WORKDIR /root/deps-build

RUN cargo init
ADD Cargo.toml Cargo.lock ./

RUN cargo fetch --locked
RUN cargo build --locked --release

WORKDIR /root/build

ADD Cargo.toml Cargo.lock ./
ADD src ./src

RUN cargo install --locked --path .


FROM scratch AS runner

COPY --from=builder /usr/local/cargo/bin/rapla-proxy /bin/rapla-proxy
ENV PATH /bin

EXPOSE 8080

CMD ["rapla-proxy"]