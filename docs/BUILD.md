# Build And Usage Guide

Current step: Step 2 - Documentation.

This repository is currently in architecture and documentation phase only. No
Rust workspace or implementation code has been created yet.

## Current Build Status

Build status:

- Not buildable yet.
- No `Cargo.toml` exists yet.
- No Rust crates exist yet.
- Implementation must wait for explicit Step 4 approval.

## Planned Prerequisites

Once implementation begins, expected prerequisites are:

- Rust stable toolchain.
- `cargo`.
- `rustfmt`.
- `clippy`.
- Network access for provider sandbox or integration tests.
- Provider sandbox credentials for WeChat Pay and Alipay.

The implementation should prefer Rust TLS dependencies that minimize external
system requirements where practical.

## Planned Workspace Commands

After the Rust workspace exists, the standard verification commands should be:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
cargo audit
cargo deny check
```

Gateway run command after implementation should follow this shape:

```text
cargo run -p unipay-gateway
```

The exact package name may change during Step 4, but it must be documented here
when implementation starts.

API contract artifacts:

- `docs/API_CONTRACT.md` is the canonical human-readable contract.
- `docs/openapi.yaml` is the machine-readable draft for client generation and
  contract tests. It must match `docs/API_CONTRACT.md`.

## Planned Configuration

Configuration must be centralized. Provider adapters must not read environment
variables directly.

Expected configuration groups:

| Group | Purpose |
| --- | --- |
| Gateway server | Host, port, request body limit, timeout. |
| Gateway auth | API key list, JWT settings when enabled. |
| HTTP client | Timeout, retry policy, user agent. |
| Storage | Database URL or secret reference, pool size, migration policy. |
| Worker | Webhook processing concurrency, retry policy, dead-letter policy. |
| WeChat Pay | Merchant id, app id, private key path, serial number, public key or platform certificate, API v3 key, notify URL. |
| Alipay | App id, private key path, Alipay public key or certificate, gateway URL, charset, sign type, notify URL, return URL. |
| Observability | Log level, trace id behavior. |

Expected environments:

- Local development.
- Provider sandbox.
- Production.

## Planned Gateway Usage

The Gateway should expose:

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

## Planned SDK Usage

The SDK should expose unified payment operations:

- Create payment.
- Query payment.
- Refund payment.
- Handle webhook.

Business systems should select a provider through explicit configuration or
request fields. They should not call provider adapters directly unless they are
inside UniPay implementation code.

## Planned Development Sequence

Recommended Step 4 implementation order:

1. Create Cargo workspace and empty crate boundaries.
2. Implement common value objects.
3. Implement core provider traits and domain models.
4. Implement storage interfaces and ledger state transitions.
5. Implement signing interfaces and test fixtures.
6. Implement shared HTTP client boundary.
7. Implement Gateway API contract skeleton and validation.
8. Implement webhook event persistence and deduplication.
9. Implement WeChat Pay Native create/query/refund mapping.
10. Implement WeChat Pay webhook verification and decryption.
11. Implement Alipay web payment create/query/refund mapping.
12. Implement Alipay async notification verification.
13. Implement Gateway composition, API routes, worker, and API key auth.
14. Add examples and integration test scaffolding.
15. Update this document with exact build and run commands.

Each step should remain independently reviewable and testable.

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

When implementation starts, update:

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
