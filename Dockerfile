FROM rust:1-alpine AS builder

WORKDIR /app

RUN apk add --no-cache musl-dev openssl-dev

COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY data ./data

RUN cargo build --release

FROM alpine:latest

RUN apk add --no-cache ca-certificates

WORKDIR /app

COPY --from=builder /app/target/release/watson /usr/local/bin/watson
COPY data ./data

RUN adduser -D watson && chown -R watson:watson /app

USER watson

ENTRYPOINT ["watson"]
