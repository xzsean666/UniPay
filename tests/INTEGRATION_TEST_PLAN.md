# UniPay MVP Integration Test Plan

This plan defines verification assets for the Step 4 MVP implementation. It is
based on `docs/API_CONTRACT.md`, `docs/CLIENT_INTEGRATION.md`, and
`docs/WEBHOOK_RELIABILITY.md`.

## Scope

The MVP verification scope covers:

- Stable `/v1` route set and response envelope.
- API key authentication for business APIs.
- Public webhook routes without business API key authentication.
- Payment create/query for WeChat Native and Alipay web.
- Refund create/query.
- Idempotent retry and idempotency conflict behavior.
- Provider webhook verification, deduplication, acknowledgement, and worker
  state transitions.
- Provider status, amount, and error mapping with mocked provider responses.

Provider sandbox tests are separate from normal automated tests because they
require external credentials, network access, and provider-specific setup.

## Test Environments

| Environment | Purpose | Required inputs |
| --- | --- | --- |
| Static contract | Validate docs and request fixtures before Gateway code exists. | `docs/openapi.yaml`, `examples/http/**` |
| Local mocked Gateway | Exercise API routes without real providers. | Local Gateway URL, API key, mock provider adapters |
| Provider sandbox | Verify real WeChat Pay and Alipay behavior. | Sandbox credentials, callback URL, test merchant setup |
| Failure-injection | Verify timeout, duplicate, replay, and dead-letter behavior. | Mock provider server, controllable storage failures |

## Static Contract Tests

The standalone package in `tests/contract-tests` should pass before and after
the main workspace is created:

```text
cargo test --manifest-path tests/contract-tests/Cargo.toml
```

Required checks:

| ID | Scenario | Expected result |
| --- | --- | --- |
| CT-001 | OpenAPI route set contains all MVP routes. | Every route from `API_CONTRACT.md` exists. |
| CT-002 | Business routes inherit bearer auth. | Payment and refund routes do not override top-level auth. |
| CT-003 | Webhook and health routes are public. | Operation security is an empty list. |
| CT-004 | Money schema uses integer minor units. | `amount_minor` is an integer with minimum `1`. |
| CT-005 | Public enum sets remain stable. | Provider, channel, status, and action enums match the contract. |
| CT-006 | Sample HTTP fixtures are contract shaped. | Business JSON fixtures include required fields and integer minor units. |

## Gateway Contract Tests

| ID | Scenario | Request | Expected response |
| --- | --- | --- | --- |
| GW-001 | Liveness succeeds. | `GET /v1/health/live` | `200`; no API key required. |
| GW-002 | Readiness succeeds when dependencies are ready. | `GET /v1/health/ready` | `200`; no API key required. |
| GW-003 | Missing API key is rejected. | Business route without `Authorization` | `401`, `UNAUTHORIZED`, `success=false`, `trace_id` present. |
| GW-004 | Invalid API key is rejected. | Business route with bad bearer token | `401`, `UNAUTHORIZED`. |
| GW-005 | Response envelope is stable. | Any success and error route | Always includes `success` and `trace_id`; errors include `error.code`, `message`, `retryable`. |
| GW-006 | Unsupported route version is not accepted. | `/v2/payments` | `404` or documented version error; no silent fallback to `/v1`. |

## Payment Tests

| ID | Scenario | Expected result |
| --- | --- | --- |
| PAY-001 | Create WeChat Native payment. | `200`, `status=pending` or `processing`, `payment_action.type=qr_code_url`. |
| PAY-002 | Create Alipay web payment. | `200`, `status=pending` or `processing`, `payment_action.type=redirect_url` or `html_form`. |
| PAY-003 | Missing required field. | `400`, `INVALID_REQUEST`, provider adapter is not called. |
| PAY-004 | Zero or floating amount. | `400`, `INVALID_AMOUNT`, provider adapter is not called. |
| PAY-005 | Unsupported provider. | `400`, `INVALID_PROVIDER`. |
| PAY-006 | Unsupported provider/channel pair. | `400`, `INVALID_CHANNEL`. |
| PAY-007 | Idempotent replay with same key and body. | Returns the same logical payment result; provider is not called again. |
| PAY-008 | Idempotency key reused with different body. | `409`, `IDEMPOTENCY_CONFLICT`. |
| PAY-009 | Provider timeout during create. | `504`, `PROVIDER_TIMEOUT`, payment remains queryable as `processing` or `unknown`. |
| PAY-010 | Query existing payment. | `200`, current local ledger status. |
| PAY-011 | Query missing payment. | `404`, `PAYMENT_NOT_FOUND`. |

## Refund Tests

| ID | Scenario | Expected result |
| --- | --- | --- |
| RFD-001 | Create refund for succeeded payment. | `200`, refund ledger record created with provider status mapping. |
| RFD-002 | Refund amount exceeds refundable amount. | `422`, `REFUND_AMOUNT_EXCEEDED`, provider adapter is not called. |
| RFD-003 | Refund before payment succeeded. | `409`, `PAYMENT_STATE_CONFLICT`. |
| RFD-004 | Idempotent refund replay with same key and body. | Returns the same logical refund result; provider is not called again. |
| RFD-005 | Refund idempotency conflict. | `409`, `IDEMPOTENCY_CONFLICT`. |
| RFD-006 | Query existing refund. | `200`, current local ledger status. |
| RFD-007 | Query missing refund. | `404`, `REFUND_NOT_FOUND`. |

## Webhook Reliability Tests

| ID | Scenario | Expected result |
| --- | --- | --- |
| WH-001 | Webhook route does not require UniPay API key. | Missing `Authorization` is allowed to reach provider signature verification. |
| WH-002 | Invalid signature. | Provider failure response; no ledger update; event is not processed as valid. |
| WH-003 | Expired timestamp or replay. | Provider failure response, `WEBHOOK_REPLAY_SUSPECTED`, security alert signal. |
| WH-004 | Valid payment webhook. | Raw body hash and selected header hash persisted; provider success acknowledgement returned after durable store. |
| WH-005 | Duplicate valid webhook. | Provider success acknowledgement; state change is not applied twice. |
| WH-006 | Valid refund webhook. | Refund ledger updates through async worker. |
| WH-007 | Out-of-order webhook after terminal state. | Event recorded; terminal state is not overwritten by older non-terminal state. |
| WH-008 | Missing related payment or refund. | Event retries, then moves to `dead_lettered` after configured attempts. |
| WH-009 | Provider-specific acknowledgement body. | WeChat and Alipay success bodies match official docs refreshed before implementation. |

## Provider Mapping Tests

Mocked provider adapter tests must cover:

- Every known WeChat payment status and refund status.
- Every known Alipay payment status and refund observation.
- Unknown provider status mapping to `unknown`.
- WeChat CNY amount mapping from integer fen.
- Alipay CNY amount conversion between integer minor units and decimal yuan
  strings without floating point arithmetic.
- Provider error mapping into stable UniPay error codes.
- Retryability classification for rate limit, timeout, transient failure, and
  deterministic rejection.

## Sandbox Checklist

Run these only with approved sandbox credentials:

1. Create a successful WeChat Native payment and verify QR action shape.
2. Receive and verify a WeChat payment success webhook.
3. Replay the same WeChat webhook and confirm no duplicate ledger transition.
4. Create a successful Alipay web payment and verify redirect or HTML action.
5. Receive and verify an Alipay async notification.
6. Create and query a refund for each provider.
7. Simulate or observe provider timeout behavior, then query before retrying.

## Acceptance Gate

The MVP is not ready until:

- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`,
  and `cargo test --workspace` pass once the workspace exists.
- Static contract tests pass.
- Gateway contract tests pass against a local mocked Gateway.
- Webhook raw-body, duplicate, replay, and dead-letter tests pass.
- Provider mapping tests pass with mocked provider responses.
- Sandbox checklist results are recorded for WeChat Pay and Alipay.
