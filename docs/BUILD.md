# Build And Usage Guide

Current step: Step 4 - Implementation.

This repository now contains a buildable MVP Rust workspace, provider adapter
crates, an HTTP Gateway, examples, and contract verification assets.

## Current Build Status

Build status:

- Buildable with the root Cargo workspace.
- Gateway binary package: `unipay-gateway`.
- Provider adapter crates: `unipay-wechat`, `unipay-alipay`.
- Current persistence implementation: in-memory development storage.
- Current provider behavior: MVP mapping and local provider-action generation;
  live WeChat Pay and Alipay HTTP calls require credentials and follow-up
  provider integration work.
- Gateway POST routes require `Idempotency-Key` and scope in-memory resources by
  authenticated caller.
- Provider webhook routes fail closed with `SIGNATURE_VERIFY_FAILED` until real
  WeChat Pay and Alipay signature verifiers are configured.

Verified commands:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
cargo test --manifest-path tests/contract-tests/Cargo.toml
cargo audit
```

The standalone contract test package declares its own workspace so API contract
checks can run independently.

## Prerequisites

Required for local development:

- Rust stable toolchain.
- `cargo`.
- `rustfmt`.
- `clippy`.
- OpenSSL development libraries or an environment where the Rust `openssl`
  crate can build.

Required for live provider integration:

- Network access for provider sandbox or production APIs.
- WeChat Pay sandbox or production credentials.
- Alipay sandbox or production credentials.
- Approved secret storage for keys and certificates.

The implementation should prefer Rust TLS dependencies that minimize external
system requirements where practical.

## Workspace Commands

Standard verification commands:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
```

Optional supply-chain checks, when tools are installed:

```text
cargo audit
cargo deny check
```

As of 2026-05-26, `cargo audit` passes and `cargo-deny` is not installed in the
local environment used for this handoff.

Gateway run command:

```text
UNIPAY_GATEWAY_API_KEYS=test-caller:test-api-key cargo run -p unipay-gateway
```

Optional bind address:

```text
UNIPAY_GATEWAY_BIND_ADDR=127.0.0.1:8080
```

API contract artifacts:

- `docs/API_CONTRACT.md` is the canonical human-readable contract.
- `docs/openapi.yaml` is the machine-readable draft for client generation and
  contract tests. It must match `docs/API_CONTRACT.md`.

## Configuration

Configuration must be centralized. Provider adapters must not read environment
variables directly.

Configuration groups:

| Group | Purpose |
| --- | --- |
| Gateway server | Host, port, request body limit, timeout. |
| Gateway auth | API key list, JWT settings when enabled. |
| HTTP client | Timeout, retry policy, user agent. |
| Storage | Current MVP uses in-memory storage; production requires database URL or secret reference, pool size, migration policy. |
| Worker | Future production worker should configure webhook processing concurrency, retry policy, dead-letter policy. |
| WeChat Pay | Merchant id, app id, private key path, serial number, public key or platform certificate, API v3 key, notify URL. |
| Alipay | App id, private key path, Alipay public key or certificate, gateway URL, charset, sign type, notify URL, return URL. |
| Observability | Log level, trace id behavior. |

Expected environments:

- Local development.
- Provider sandbox.
- Production.

## Gateway Usage

The Gateway exposes:

- `POST /v1/payments`
- `GET /v1/payments/{merchant_order_id}`
- `POST /v1/refunds`
- `GET /v1/refunds/{merchant_refund_id}`
- `POST /v1/webhooks/{provider}/payments`
- `POST /v1/webhooks/{provider}/refunds`
- `GET /v1/health/live`
- `GET /v1/health/ready`

Gateway callers must authenticate with API key in MVP. JWT support is planned
but not required for the first implementation pass.

## SDK Usage

The SDK should expose unified payment operations:

- Create payment.
- Query payment.
- Refund payment.
- Handle webhook.

Business systems should select a provider through explicit configuration or
request fields. They should not call provider adapters directly unless they are
inside UniPay implementation code.

## Implemented Workspace

Implemented crates:

- `crates/common`: shared value objects.
- `crates/core`: provider-neutral models, errors, traits, and service
  orchestration.
- `crates/storage`: in-memory development storage implementing core storage
  traits.
- `crates/signing`: RSA-SHA256 signing and verification helpers.
- `crates/http-client`: shared async HTTP client wrapper.
- `crates/wechat`: WeChat Pay v3 mapping and MVP provider adapter.
- `crates/alipay`: Alipay mapping and MVP provider adapter.
- `gateway`: Axum HTTP Gateway implementing `/v1` contract shape.

Examples:

- `examples/http/unipay_mvp.http`
- `examples/http/json/*.json`

Contract tests:

- `tests/contract-tests`

## Remaining Production Work

Before real-money production usage:

1. Replace in-memory storage with a durable database implementation.
2. Complete live WeChat Pay v3 request signing, HTTP calls, response signature
   verification, platform certificate/public key handling, and API v3 resource
   decryption.
3. Complete live Alipay RSA2 signing, HTTP calls, response verification, and
   async notification verification.
4. Wire Gateway routes to the SDK `PaymentService` and provider registry instead
   of the current local in-memory Gateway service.
5. Add provider sandbox integration tests.
6. Add database migrations and recovery tooling.
7. Add operational metrics, alerts, and deployment manifests.

## Verification Policy

Before merging implementation work:

- Formatting must pass.
- Clippy must pass unless a documented exception is approved.
- Unit tests must pass.
- Provider behavior must be tested with mocked HTTP responses.
- Sandbox tests must be documented separately from normal unit tests.
- API contract tests must pass.
- Security redaction tests must pass.
- Webhook duplicate and replay tests must pass.
- Dependency audit and license checks must pass or have approved exceptions.

Webhook tests must include raw body verification. Tests that verify parsed JSON
only are insufficient.

## Documentation Maintenance

When implementation changes behavior, update:

- `docs/BUILD.md` with actual command output expectations.
- `docs/SPEC.md` if scope changes.
- `docs/API_CONTRACT.md` if Gateway behavior changes.
- `docs/CLIENT_INTEGRATION.md` if caller behavior changes.
- `docs/ERROR_CODES.md` if public errors change.
- `docs/DATA_MODEL.md` if persistence behavior changes.
- `docs/WEBHOOK_RELIABILITY.md` if callback behavior changes.
- `docs/SECURITY.md` if secret, auth, or logging behavior changes.
- `docs/OPERATIONS.md` if deployment or runtime behavior changes.
- `docs/PROVIDER_MAPPING.md` if provider mappings change.
- `docs/PROVIDER_ADAPTER_GUIDE.md` if extension rules change.
- `docs/INTEGRATION_DOCS.md` when provider docs are refreshed.
- `docs/nextsession.md` at the end of the session.
