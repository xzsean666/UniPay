# syntax=docker/dockerfile:1

FROM rust:1.95-slim-bookworm AS builder

WORKDIR /workspace

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        libssl-dev \
        pkg-config \
    && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
COPY gateway ./gateway

RUN cargo build --release -p unipay-gateway

FROM debian:bookworm-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        ca-certificates \
        curl \
        libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && groupadd --system unipay \
    && useradd --system --gid unipay --home-dir /nonexistent --shell /usr/sbin/nologin unipay

WORKDIR /app

COPY --from=builder /workspace/target/release/unipay-gateway /usr/local/bin/unipay-gateway

ENV UNIPAY_GATEWAY_BIND_ADDR=0.0.0.0:8080
ENV RUST_LOG=info

EXPOSE 8080

USER unipay

HEALTHCHECK --interval=30s --timeout=3s --start-period=10s --retries=3 \
    CMD curl -fsS http://127.0.0.1:8080/v1/health/live >/dev/null || exit 1

ENTRYPOINT ["/usr/local/bin/unipay-gateway"]
