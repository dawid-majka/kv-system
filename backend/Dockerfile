# Build
FROM rust:latest as builder

RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY . .

RUN cargo build --release

# Runtime
FROM debian:bookworm-slim AS runtime
WORKDIR /app

COPY --from=builder /app/target/release/backend backend
COPY backend/configuration configuration

COPY cert2.pem /app/cert2.pem
COPY key2.pem /app/key2.pem

ENV APP_ENVIRONMENT production

ENTRYPOINT ["/app/backend"]