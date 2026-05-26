# System Specification

Current step: Step 2 - Documentation.

This document defines the functional specification for UniPay. It intentionally
contains no implementation code.

## Product Scope

UniPay is a Rust unified payment SDK plus HTTP API Gateway.

It provides:

- A Rust SDK for server-side Rust applications.
- A Payment API Gateway for non-Rust or loosely coupled business systems.
- A shared Payment Core used by both SDK and Gateway.
- Provider adapters for payment platforms.
- Unified signing, webhook verification, error handling, and idempotency rules.
- Durable payment, refund, provider request, webhook, and idempotency records.

## MVP Scope

MVP providers:

- WeChat Pay v3 Native payment.
- Alipay computer website payment.

MVP capabilities:

- Create payment.
- Query payment.
- Create refund.
- Query refund.
- Verify and parse payment webhook.
- Verify and parse refund webhook where provider support is included in MVP.
- Expose SDK and HTTP API entry points.
- Support API key authentication for the Gateway.
- Reserve JWT support as a planned gateway authentication mode.
- Persist payment and refund ledger records.
- Persist provider request and webhook event records.

Out of scope for MVP:

- SaaS tenant management.
- Risk control.
- Settlement and reconciliation.
- Management dashboard.
- Automatic payment routing.
- Multi-provider failover.
- Multi-currency routing.

## Public Entry Points

### SDK Entry Point

The SDK exposes provider-neutral payment operations to Rust services.

Required SDK operations:

| Operation | Purpose | Input | Output |
| --- | --- | --- | --- |
| Create payment | Start a provider payment flow. | Unified create payment request. | Unified create payment response. |
| Query payment | Fetch current payment status. | Provider, merchant order id, optional provider transaction id. | Unified payment status result. |
| Refund payment | Request full or partial refund. | Unified refund request. | Unified refund response. |
| Query refund | Fetch current refund status. | Provider, merchant refund id, optional provider refund id. | Unified refund status result. |
| Handle webhook | Verify and parse provider callback. | Provider, raw body, original headers. | Unified payment event or refund event. |

### Gateway Entry Point

The Gateway exposes stable HTTP APIs over the SDK.

Required MVP routes are versioned under `/v1`. See `API_CONTRACT.md` for the
complete contract.

| Route | Method | Purpose |
| --- | --- | --- |
| `/v1/payments` | `POST` | Create a payment through the selected provider. |
| `/v1/payments/{merchant_order_id}` | `GET` | Query payment status by provider and merchant order id. |
| `/v1/refunds` | `POST` | Create a refund through the selected provider. |
| `/v1/refunds/{merchant_refund_id}` | `GET` | Query refund status by provider and merchant refund id. |
| `/v1/webhooks/{provider}/payments` | `POST` | Receive provider payment callbacks. |
| `/v1/webhooks/{provider}/refunds` | `POST` | Receive provider refund callbacks when implemented. |

Gateway responses must use a stable JSON envelope:

- Success responses contain a provider-neutral data object.
- Error responses contain a stable error code, message, provider if known, and
  request trace id.
- Secrets, signatures, private keys, and raw provider credentials must never be
  returned.

## Core Domain Model

Core domain models must be provider-neutral.

Required payment concepts:

| Concept | Meaning |
| --- | --- |
| Provider | Payment platform identifier, such as `wechat` or `alipay`. |
| Merchant order id | Business system order id; primary idempotency key for payment creation. |
| Provider transaction id | Payment platform transaction id, present after provider accepts or completes payment. |
| Amount | Integer minor units plus ISO 4217 currency code. |
| Payment channel | Provider-specific payment product mapped to a unified enum. |
| Payment action | Client-side action required to continue payment, such as QR code URL, redirect URL, or HTML form. |
| Payment status | Unified lifecycle state. |
| Refund id | Business refund id; primary idempotency key for refund creation. |
| Provider refund id | Payment platform refund id. |
| Webhook event | Verified provider callback converted into a provider-neutral event. |
| Provider request record | Redacted audit record for outbound provider calls. |
| Idempotency record | Durable record that prevents duplicate business operations. |

Required payment status values:

- `pending`
- `processing`
- `succeeded`
- `failed`
- `closed`
- `refunding`
- `partially_refunded`
- `refunded`
- `unknown`

Required refund status values:

- `pending`
- `processing`
- `succeeded`
- `failed`
- `closed`
- `unknown`

## Provider Requirements

### WeChat Pay v3 MVP

Required payment product:

- Native payment.

Required behavior:

- Create Native payment and return QR code URL data.
- Query by merchant order id.
- Create full or partial refund.
- Verify WeChat Pay response signatures where applicable.
- Verify payment webhook signatures.
- Decrypt encrypted webhook resource payloads.
- Map WeChat trade state to unified payment status.
- Preserve merchant order id and merchant refund id as idempotency boundaries.

Provider-specific details must remain inside the WeChat adapter:

- `appid`
- `mchid`
- `out_trade_no`
- `transaction_id`
- `out_refund_no`
- `notify_url`
- WeChat Pay API v3 authorization headers.
- WeChat Pay public key or platform certificate verification.
- API v3 key based callback decryption.

### Alipay MVP

Required payment product:

- Computer website payment.

Required behavior:

- Create web payment and return the provider payment action required by Alipay.
- Query payment by merchant order id or provider transaction id.
- Create full or partial refund.
- Verify synchronous or asynchronous Alipay signatures where applicable.
- Parse asynchronous payment notification.
- Map Alipay trade status to unified payment status.
- Preserve merchant order id and refund request id as idempotency boundaries.

Provider-specific details must remain inside the Alipay adapter:

- `app_id`
- `method`
- `charset`
- `sign_type`
- `notify_url`
- `return_url`
- `out_trade_no`
- `trade_no`
- `biz_content`
- RSA2 signing and verification.

### Future Providers

Future providers must be added as adapters behind the existing Payment Core:

- Stripe
- PayPal
- Apple Pay
- Google Pay

Adding a provider should not require changing existing MVP route names. New
provider-specific payment actions may be added only if the unified payment action
model cannot already express the required client behavior.

## Signing And Verification

Signing and verification are centralized in the signing module.

Required signing capabilities:

- RSA-SHA256 style signing for WeChat Pay v3.
- RSA2 signing and verification for Alipay.
- HMAC-style verification for providers that require it in future phases.
- Raw-body webhook verification.
- Certificate or public key loading through explicit configuration.

Rules:

- Provider adapters request signing from the signing module.
- Provider adapters do not embed low-level crypto logic.
- Private keys and API keys are never logged.
- Webhook verification must use the exact raw body and original headers.
- Parsed JSON must not be used as the source for signature verification.

## HTTP Client Requirements

All provider adapters use a shared async HTTP client boundary.

Required behavior:

- `reqwest` based async requests.
- Configurable timeout.
- Explicit retry policy for safe retry cases.
- Structured request tracing.
- Provider response status and body capture for diagnostics.
- No logging of secrets or full authorization headers.

Retry rules:

- Query requests may be retried when transport failure is safe.
- Create payment and refund requests require an idempotency key before retry.
- Webhook handling must not call provider APIs unless the workflow explicitly
  requires secondary verification.

## Gateway Authentication

MVP authentication:

- API key authentication.

Planned authentication:

- JWT authentication.

Rules:

- Gateway authentication is separate from provider request signing.
- Gateway auth failures must not call Payment Core.
- Authenticated caller context must be explicit and passed into request handling.
- API keys and JWT secrets must be configured centrally.

## Configuration

Configuration must be centralized.

Required configuration groups:

- Gateway server address and port.
- Gateway auth configuration.
- Provider selection rules.
- WeChat Pay configuration.
- Alipay configuration.
- HTTP timeout and retry policy.
- Logging level.

Provider adapters must receive explicit configuration objects. They must not read
environment variables directly.

## Storage Requirements

Production storage requirements are defined in `DATA_MODEL.md`.

MVP must persist:

- Payment ledger records.
- Refund ledger records.
- Provider request records.
- Webhook event records.
- Idempotency records.

Provider adapters must not write storage directly. Core or application services
own transaction boundaries and state transitions.

## Error Model

Errors must be unified but traceable.

Required error categories:

- Validation error.
- Authentication error.
- Authorization error.
- Provider rejected request.
- Provider unavailable.
- Signature creation failed.
- Signature verification failed.
- Webhook payload invalid.
- Webhook replay suspected.
- Transport error.
- Timeout.
- Internal error.

Each error should preserve:

- Provider, when known.
- Operation name.
- Provider request id or trace id, when available.
- Retryability classification.
- Safe public message.
- Internal diagnostic detail for logs.

Stable public error codes are defined in `ERROR_CODES.md`.

## Idempotency

Payment creation idempotency:

- Merchant order id is required.
- The same merchant order id must not create multiple charge attempts unless the
  provider explicitly supports and requires a new payment attempt model.

Refund idempotency:

- Merchant refund id is required.
- Retry must reuse the same merchant refund id.

Gateway idempotency:

- Gateway should accept an explicit `Idempotency-Key` header.
- MVP must also enforce merchant order id and merchant refund id uniqueness.
- Idempotency records must be durable.

## Observability

Required observability:

- Structured logs through `tracing`.
- Request trace id for Gateway requests.
- Provider operation name in logs.
- Provider response code classification.
- Webhook event id logging after verification.

Do not log:

- Private keys.
- API keys.
- JWT secrets.
- Authorization headers.
- Full webhook ciphertext.
- Full personally identifiable user data.

## Test Strategy

MVP tests should cover:

- Core status mapping.
- Provider request mapping.
- Provider response mapping.
- Signing canonicalization.
- Signature verification using fixture data.
- Webhook raw-body verification and parsing.
- Gateway request validation.
- Gateway error response envelope.
- Idempotency behavior for retry cases.
- Contract tests for `API_CONTRACT.md`.
- Ledger state transition tests.
- Webhook deduplication and dead-letter tests.
- Provider mapping tests from `PROVIDER_MAPPING.md`.
- Security redaction tests.

Provider network calls should be tested with mocks before using sandbox
credentials.

## Documentation Requirements

Before implementing or changing provider behavior:

1. Read `docs/INTEGRATION_DOCS.md`.
2. Open the relevant official provider documentation.
3. Confirm the API endpoint, parameters, signature rules, and callback rules are
   still current.
4. Update the access date or notes in `docs/INTEGRATION_DOCS.md` when docs
   change.

Production-facing implementation must also keep these documents current:

- `API_CONTRACT.md`
- `CLIENT_INTEGRATION.md`
- `ERROR_CODES.md`
- `DATA_MODEL.md`
- `WEBHOOK_RELIABILITY.md`
- `SECURITY.md`
- `OPERATIONS.md`
- `PROVIDER_MAPPING.md`
- `PROVIDER_ADAPTER_GUIDE.md`
