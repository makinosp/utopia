FROM rust:1.86.0-alpine3.21 AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock* ./
COPY src ./src
COPY migrations ./migrations

RUN apk add --no-cache musl-dev openssl-dev pkgconfig
RUN cargo build --release

FROM alpine:3.21

WORKDIR /app

RUN adduser -D utopia

COPY --from=builder /app/target/release/utopia /app/utopia
COPY migrations /app/migrations

USER utopia

EXPOSE 3000

ENTRYPOINT ["/app/utopia"]
