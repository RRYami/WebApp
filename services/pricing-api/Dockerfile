# syntax=docker/dockerfile:1
FROM rust:1.85-slim-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY libs/pricing-core libs/pricing-core
COPY services/pricing-api services/pricing-api

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

RUN cargo build --release -p pricing-api

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/pricing-api /usr/local/bin/pricing-api

EXPOSE 3000

ENTRYPOINT ["/usr/local/bin/pricing-api"]
