# Deployment Guide

Current step: Step 4 - Implementation.

This document defines the current Docker deployment path and environment
variables for the UniPay Gateway MVP.

## Current Deployment Status

The Docker image runs the `unipay-gateway` binary only.

Current limitations:

- State is in memory and is lost when the container restarts.
- WeChat Pay and Alipay live provider calls are not enabled yet.
- Provider webhook routes fail closed until signature verification is wired.
- Database, worker, provider secrets, and real payment credentials are future
  production work.

## Files

| File | Purpose |
| --- | --- |
| `Dockerfile` | Multi-stage release build for `unipay-gateway`. |
| `.dockerignore` | Keeps local build artifacts and secrets out of Docker context. |
| `docker-compose.yml` | Local single-container deployment example. |
| `.env.example` | Environment variable template for local compose usage. |

## Required Environment Variables

| Variable | Required | Example | Notes |
| --- | --- | --- | --- |
| `UNIPAY_GATEWAY_API_KEYS` | Yes | `demo-backend:replace-with-long-random-api-key` | Comma-separated `caller_id:api_key` entries. |
| `UNIPAY_GATEWAY_BIND_ADDR` | No | `0.0.0.0:8080` | Must use `0.0.0.0` inside Docker. Defaults to `127.0.0.1:8080` when unset, which is not suitable for containers. |
| `RUST_LOG` | No | `info,unipay_gateway=debug` | Standard tracing filter. |
| `UNIPAY_GATEWAY_PORT` | Compose only | `8080` | Host port mapped to container port `8080`. |

## Local Docker Compose

Create a local environment file when you want to override the placeholder
values:

```text
cp .env.example .env
```

Edit `.env` and replace `UNIPAY_GATEWAY_API_KEYS` with a local test key. Compose
loads `.env.example` first and optional `.env` second, so `.env` overrides the
template.

Build and start:

```text
docker compose up --build
```

Health check:

```text
curl http://127.0.0.1:8080/v1/health/live
```

Example authenticated request:

```text
curl -sS http://127.0.0.1:8080/v1/payments \
  -H 'Authorization: Bearer replace-with-long-random-api-key' \
  -H 'Idempotency-Key: order_202605260001' \
  -H 'Content-Type: application/json' \
  -d '{
    "provider": "wechat",
    "merchant_order_id": "order_202605260001",
    "amount": {
      "currency": "CNY",
      "amount_minor": 100
    },
    "subject": "Docker local test",
    "channel": "native"
  }'
```

## Docker Build Without Compose

Build:

```text
docker build -t unipay-gateway:local .
```

Run:

```text
docker run --rm \
  -p 8080:8080 \
  -e UNIPAY_GATEWAY_BIND_ADDR=0.0.0.0:8080 \
  -e UNIPAY_GATEWAY_API_KEYS=demo-backend:replace-with-long-random-api-key \
  -e RUST_LOG=info \
  unipay-gateway:local
```

## Environment Variable Rules

API keys:

- Use long random values outside local development.
- Rotate keys by adding the new `caller_id:api_key` entry, deploying, moving
  callers to the new key, then removing the old entry.
- Do not log or commit API keys.

Binding:

- Local cargo run can use `127.0.0.1:8080`.
- Docker must use `0.0.0.0:8080` so the container port is reachable.

Logging:

- Use `RUST_LOG=info` for normal local runs.
- Use `RUST_LOG=info,unipay_gateway=debug` for troubleshooting.
- Do not log authorization headers, provider secrets, private keys, signatures,
  or raw payment webhook payloads.

## Reserved Future Variables

The following variable groups are documented in `.env.example` but are not read
by the current MVP:

- `UNIPAY_DATABASE_URL`
- `UNIPAY_WECHAT_*`
- `UNIPAY_ALIPAY_*`

They must not be treated as active production configuration until the durable
storage and live provider integration work is implemented.

## Production Notes

Before production:

1. Replace in-memory state with durable database storage.
2. Add database migrations and readiness checks that verify database access.
3. Load provider keys from approved secrets, not plain `.env` files.
4. Wire live WeChat Pay and Alipay signing, HTTP calls, response verification,
   and webhook verification.
5. Add reverse proxy TLS, request size limits, rate limits, metrics, alerts, and
   deployment rollback procedures.
