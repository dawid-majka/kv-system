# Build
FROM rust:latest as builder

RUN apt-get update && apt-get install -y protobuf-compiler && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY . .

RUN cargo build --release

# Runtime
FROM debian:bookworm-slim AS runtime
WORKDIR /app

COPY --from=builder /app/target/release/frontend frontend
COPY frontend/configuration configuration

ENV APP_ENVIRONMENT production

ENTRYPOINT ["/app/frontend"]