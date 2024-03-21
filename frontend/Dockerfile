# Build
FROM rust:latest as builder

RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY . .

RUN cargo build --release

# Runtime
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/frontend frontend
COPY frontend/configuration configuration

COPY cert.pem /app/cert.pem
COPY key.pem /app/key.pem

ENV APP_ENVIRONMENT production

ENTRYPOINT ["/app/frontend"]