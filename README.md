# UniPay

UniPay is a Rust unified payment SDK and Payment API Gateway.

The goal is to provide one provider-neutral payment layer for backend systems,
so business services can create payments, query payments, request refunds, and
handle provider callbacks without depending directly on WeChat Pay, Alipay, or
future payment providers.

## Current Status

This repository is currently a buildable local-development MVP foundation.

Implemented:

- Rust Cargo workspace with modular crates.
- Provider-neutral payment core models, traits, errors, and service layer.
- In-memory development storage.
- WeChat Pay and Alipay adapter boundaries with MVP mappings.
- Axum-based HTTP Gateway.
- API key authentication for backend callers.
- Required `Idempotency-Key` handling for create routes.
- Caller-scoped in-memory resources.
- Unified JSON success and error envelopes.
- OpenAPI draft and contract tests.
- Dockerfile, Docker Compose example, and `.env.example`.

Not production-complete yet:

- Gateway state is in memory and is lost on restart.
- WeChat Pay and Alipay live HTTP calls are not wired yet.
- Provider request signing and response verification are not complete.
- Provider webhook routes fail closed until real signature verification is
  configured.
- Durable database storage, migrations, webhook workers, metrics, alerts, and
  deployment manifests are still pending.

Do not use the current MVP for real-money production payments.

## Architecture

```text
Business Backend
  |
  |-- Rust SDK integration ------------------|
  |                                          |
  |-- HTTP integration --> Payment Gateway --|
                                             |
                                             v
                                      Payment Core
                                             |
                         ----------------------------
                         |                          |
                    WeChat Pay v3                Alipay
```

The Gateway is an HTTP entry point. The SDK and Gateway are intended to share
the same payment core. Provider-specific request fields, signing behavior,
status mapping, and webhook verification stay inside provider adapters.

## Workspace Layout

```text
crates/
  common/       Shared value objects and helpers.
  core/         Provider-neutral payment domain, traits, errors, service layer.
  storage/      In-memory development storage.
  signing/      RSA-SHA256 signing and verification helpers.
  http-client/  Shared async HTTP client wrapper.
  wechat/       WeChat Pay v3 adapter boundary and mappings.
  alipay/       Alipay adapter boundary and mappings.
gateway/        Axum Payment API Gateway.
examples/http/  REST Client examples and JSON fixtures.
tests/          Integration and contract test assets.
docs/           Architecture, API, deployment, security, and handoff docs.
```

## Quick Start

Run the Gateway locally with Cargo:

```bash
UNIPAY_GATEWAY_API_KEYS=demo-backend:demo-api-key \
cargo run -p unipay-gateway
```

Health check:

```bash
curl http://127.0.0.1:8080/v1/health/live
```

Create a local WeChat Native payment record:

```bash
curl -sS http://127.0.0.1:8080/v1/payments \
  -H 'Authorization: Bearer demo-api-key' \
  -H 'Idempotency-Key: order_demo_001' \
  -H 'Content-Type: application/json' \
  -d '{
    "provider": "wechat",
    "merchant_order_id": "order_demo_001",
    "amount": {
      "currency": "CNY",
      "amount_minor": 100
    },
    "subject": "Demo payment",
    "channel": "native"
  }'
```

## Docker

Run with Docker Compose:

```bash
cp .env.example .env
docker compose up --build
```

The Gateway must bind to `0.0.0.0:8080` inside Docker.

Main environment variables:

| Variable | Required | Example |
| --- | --- | --- |
| `UNIPAY_GATEWAY_API_KEYS` | Yes | `demo-backend:replace-with-long-random-api-key` |
| `UNIPAY_GATEWAY_BIND_ADDR` | No | `0.0.0.0:8080` |
| `UNIPAY_GATEWAY_PORT` | Compose only | `8080` |
| `RUST_LOG` | No | `info,unipay_gateway=debug` |

See [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md) for full Docker and environment
variable documentation.

## Gateway API

Current `/v1` routes:

| Method | Path | Purpose |
| --- | --- | --- |
| `POST` | `/v1/payments` | Create a payment. |
| `GET` | `/v1/payments/{merchant_order_id}?provider=wechat` | Query payment. |
| `POST` | `/v1/refunds` | Create a refund. |
| `GET` | `/v1/refunds/{merchant_refund_id}?provider=wechat` | Query refund. |
| `POST` | `/v1/webhooks/{provider}/payments` | Receive provider payment webhook. |
| `POST` | `/v1/webhooks/{provider}/refunds` | Receive provider refund webhook. |
| `GET` | `/v1/health/live` | Liveness check. |
| `GET` | `/v1/health/ready` | Readiness check. |

Business routes require:

```text
Authorization: Bearer <api_key>
```

Create routes also require:

```text
Idempotency-Key: <stable_business_operation_key>
```

Detailed API documentation:

- [docs/API_CONTRACT.md](docs/API_CONTRACT.md)
- [docs/openapi.yaml](docs/openapi.yaml)
- [docs/CLIENT_INTEGRATION.md](docs/CLIENT_INTEGRATION.md)

## Verification

Standard checks:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
cargo test --manifest-path tests/contract-tests/Cargo.toml
cargo audit
```

`cargo-deny` can also be used when installed:

```bash
cargo deny check
```

## Documentation

Start here:

- [Agent.md](Agent.md): AI agent workflow and repository rules.
- [docs/README.md](docs/README.md): Documentation index.
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md): Architecture and module data flow.
- [docs/BUILD.md](docs/BUILD.md): Build and usage guide.
- [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md): Docker and environment variables.
- [docs/INTEGRATION_DOCS.md](docs/INTEGRATION_DOCS.md): Official provider docs index.

## Production Roadmap

Before production real-money usage:

1. Replace in-memory storage with durable database storage.
2. Wire Gateway routes to the shared core `PaymentService`.
3. Implement WeChat Pay v3 request signing, live HTTP calls, response
   verification, platform certificate or public key handling, callback
   verification, and API v3 resource decryption.
4. Implement Alipay RSA2 signing, live HTTP calls, response verification, and
   async notification verification.
5. Add provider sandbox integration tests.
6. Add webhook persistence, asynchronous processing, retries, and dead-letter
   handling.
7. Add metrics, alerts, readiness dependency checks, deployment manifests, and
   rollback procedures.

## License

MIT
