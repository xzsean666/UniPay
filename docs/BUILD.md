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
```

Gateway run command after implementation should follow this shape:

```text
cargo run -p unipay-gateway
```

The exact package name may change during Step 4, but it must be documented here
when implementation starts.

## Planned Configuration

Configuration must be centralized. Provider adapters must not read environment
variables directly.

Expected configuration groups:

| Group | Purpose |
| --- | --- |
| Gateway server | Host, port, request body limit, timeout. |
| Gateway auth | API key list, JWT settings when enabled. |
| HTTP client | Timeout, retry policy, user agent. |
| WeChat Pay | Merchant id, app id, private key path, serial number, public key or platform certificate, API v3 key, notify URL. |
| Alipay | App id, private key path, Alipay public key or certificate, gateway URL, charset, sign type, notify URL, return URL. |
| Observability | Log level, trace id behavior. |

Expected environments:

- Local development.
- Provider sandbox.
- Production.

## Planned Gateway Usage

The Gateway should expose:

- `POST /payments/create`
- `POST /payments/refund`
- `GET /payments/query`
- `POST /webhooks/{provider}`
- `POST /webhooks/{provider}/refunds`

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
4. Implement signing interfaces and test fixtures.
5. Implement shared HTTP client boundary.
6. Implement WeChat Pay Native create/query/refund mapping.
7. Implement WeChat Pay webhook verification and decryption.
8. Implement Alipay web payment create/query/refund mapping.
9. Implement Alipay async notification verification.
10. Implement Gateway composition, API routes, and API key auth.
11. Add examples and integration test scaffolding.
12. Update this document with exact build and run commands.

Each step should remain independently reviewable and testable.

## Verification Policy

Before merging implementation work:

- Formatting must pass.
- Clippy must pass unless a documented exception is approved.
- Unit tests must pass.
- Provider behavior must be tested with mocked HTTP responses.
- Sandbox tests must be documented separately from normal unit tests.

Webhook tests must include raw body verification. Tests that verify parsed JSON
only are insufficient.

## Documentation Maintenance

When implementation starts, update:

- `docs/BUILD.md` with actual command output expectations.
- `docs/SPEC.md` if scope changes.
- `docs/INTEGRATION_DOCS.md` when provider docs are refreshed.
- `docs/nextsession.md` at the end of the session.

