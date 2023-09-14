FROM golang:alpine AS builder

WORKDIR /root/build
ADD . .
RUN go build ./cmd/rapla-proxy


FROM scratch AS runner

COPY --from=builder /root/build/rapla-proxy /bin/rapla-proxy

ENV PATH /bin
EXPOSE 8080

CMD ["rapla-proxy"]