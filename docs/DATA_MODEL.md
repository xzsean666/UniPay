# Data Model

Current step: Step 2 - Documentation Hardening.

This document defines the production data model required for payment reliability.
It is a logical model, not implementation code.

## Design Goal

UniPay must maintain its own durable record of payment operations. Provider APIs
and webhooks are not a replacement for a local ledger.

The data model must support:

- Idempotent create payment.
- Idempotent refund.
- Provider request audit trail.
- Webhook deduplication.
- Payment and refund state recovery after timeout.
- Reconciliation and manual investigation.

## Storage Boundary

Planned module boundary:

```text
crates/storage
```

Responsibilities:

- Persist payment records.
- Persist refund records.
- Persist provider request records.
- Persist webhook event records.
- Persist idempotency records.
- Provide transaction boundaries to Core.

Provider adapters must not write to storage directly. Core or application
services coordinate storage and provider calls.

## Logical Tables

### `payments`

Purpose: canonical UniPay payment ledger.

| Field | Required | Notes |
| --- | --- | --- |
| `payment_id` | Yes | UniPay generated stable id. |
| `provider` | Yes | `wechat`, `alipay`, future providers. |
| `merchant_order_id` | Yes | Unique per provider and merchant account. |
| `provider_transaction_id` | No | Filled after provider accepts or completes payment. |
| `channel` | Yes | Unified channel such as `native` or `web`. |
| `currency` | Yes | ISO 4217 code. |
| `amount_minor` | Yes | Integer minor units. |
| `status` | Yes | Unified payment status. |
| `subject` | Yes | Safe display subject. |
| `description` | No | Safe description. |
| `notify_url` | No | Caller or provider callback URL policy. |
| `expires_at` | No | Payment expiry. |
| `paid_at` | No | Successful payment time from provider or UniPay. |
| `closed_at` | No | Closed or expired time. |
| `metadata_json` | No | Caller metadata after validation and size limits. |
| `version` | Yes | Optimistic concurrency version. |
| `created_at` | Yes | Creation time. |
| `updated_at` | Yes | Last update time. |

Unique constraints:

- `(provider, merchant_order_id)`
- `(provider, provider_transaction_id)` when provider transaction id is present

Status transition rule:

- Terminal states must not move back to non-terminal states except through a
  documented manual repair path.

### `refunds`

Purpose: canonical UniPay refund ledger.

| Field | Required | Notes |
| --- | --- | --- |
| `refund_id` | Yes | UniPay generated stable id. |
| `payment_id` | Yes | Links to payment. |
| `provider` | Yes | Payment provider. |
| `merchant_order_id` | Yes | Original order id. |
| `merchant_refund_id` | Yes | Unique refund id from caller. |
| `provider_refund_id` | No | Provider refund id. |
| `currency` | Yes | ISO 4217 code. |
| `amount_minor` | Yes | Integer minor units. |
| `status` | Yes | Unified refund status. |
| `reason` | No | Safe refund reason. |
| `notify_url` | No | Refund callback URL policy. |
| `succeeded_at` | No | Refund success time. |
| `closed_at` | No | Refund closed time. |
| `metadata_json` | No | Caller metadata after validation and size limits. |
| `version` | Yes | Optimistic concurrency version. |
| `created_at` | Yes | Creation time. |
| `updated_at` | Yes | Last update time. |

Unique constraints:

- `(provider, merchant_refund_id)`
- `(provider, provider_refund_id)` when provider refund id is present

Business constraint:

- Sum of successful and processing refunds for a payment cannot exceed payment
  amount unless a provider-specific exception is explicitly modeled.

### `provider_requests`

Purpose: audit trail for outbound provider calls.

| Field | Required | Notes |
| --- | --- | --- |
| `provider_request_id` | Yes | UniPay generated id. |
| `provider` | Yes | Provider name. |
| `operation` | Yes | `create_payment`, `query_payment`, `create_refund`, `query_refund`. |
| `idempotency_key` | No | Business idempotency key used for this call. |
| `payment_id` | No | Related payment. |
| `refund_id` | No | Related refund. |
| `http_method` | Yes | Provider HTTP method. |
| `endpoint_path` | Yes | Path without secret query values. |
| `request_hash` | Yes | Hash of canonical redacted request. |
| `response_status` | No | HTTP response status. |
| `provider_code` | No | Provider error or result code. |
| `retryable` | Yes | Classification at time of response. |
| `duration_ms` | No | Request duration. |
| `trace_id` | Yes | UniPay trace id. |
| `created_at` | Yes | Creation time. |

Rules:

- Store redacted request and response only if approved by security policy.
- Never store private keys, API keys, authorization headers, or full signatures.
- Store hashes to diagnose idempotency conflicts without retaining secrets.

### `webhook_events`

Purpose: durable record and deduplication for provider callbacks.

| Field | Required | Notes |
| --- | --- | --- |
| `webhook_event_id` | Yes | UniPay generated id. |
| `provider` | Yes | Provider name. |
| `event_type` | Yes | Payment or refund event type. |
| `provider_event_id` | No | Provider event id when available. |
| `deduplication_key` | Yes | Provider event id or hash fallback. |
| `signature_valid` | Yes | Signature verification result. |
| `raw_body_hash` | Yes | Hash of raw body. |
| `headers_hash` | Yes | Hash of selected original headers. |
| `received_at` | Yes | Receive time. |
| `verified_at` | No | Verification time. |
| `processed_at` | No | Business processing time. |
| `processing_status` | Yes | `received`, `verified`, `processed`, `failed`, `dead_lettered`. |
| `attempt_count` | Yes | Internal processing attempts. |
| `last_error_code` | No | Last UniPay error code. |
| `trace_id` | Yes | UniPay trace id. |

Unique constraints:

- `(provider, deduplication_key)`

Rules:

- Raw body may be stored encrypted only if security policy approves.
- At minimum, store raw body hash and enough metadata for deduplication.
- Duplicate verified webhooks should be acknowledged successfully after
  deduplication.

### `idempotency_records`

Purpose: prevent duplicate business operations.

| Field | Required | Notes |
| --- | --- | --- |
| `idempotency_record_id` | Yes | UniPay generated id. |
| `caller_id` | Yes | Authenticated caller or merchant account. |
| `idempotency_key` | Yes | Caller header or derived merchant id. |
| `operation` | Yes | `create_payment` or `create_refund`. |
| `request_hash` | Yes | Hash of canonical request. |
| `resource_type` | No | `payment` or `refund`. |
| `resource_id` | No | Linked payment or refund id. |
| `response_hash` | No | Hash of stable response if replay is supported. |
| `status` | Yes | `started`, `completed`, `failed`. |
| `expires_at` | Yes | Retention end. |
| `created_at` | Yes | Creation time. |
| `updated_at` | Yes | Last update time. |

Unique constraints:

- `(caller_id, operation, idempotency_key)`

Rules:

- Same key and same request returns the existing result when possible.
- Same key and different request returns `IDEMPOTENCY_CONFLICT`.
- Retention must be long enough for caller retry windows and provider callback
  delays.

## Transaction Patterns

### Create Payment

1. Start database transaction.
2. Create or load idempotency record.
3. Create local payment in `pending`.
4. Commit before calling provider, or use a documented pending-outbound pattern.
5. Call provider with idempotent merchant order id.
6. Record provider request.
7. Update payment with provider result.
8. Complete idempotency record.

If provider call outcome is unknown:

- Keep payment in `processing` or `unknown`.
- Query provider before any retry that could create a duplicate charge.

### Create Refund

1. Start database transaction.
2. Lock payment row or use optimistic concurrency.
3. Validate refundable amount.
4. Create or load idempotency record.
5. Create local refund in `pending`.
6. Commit before calling provider, or use a documented pending-outbound pattern.
7. Call provider with merchant refund id.
8. Record provider request.
9. Update refund status.
10. Update payment aggregate refund status.

## State Machine Ownership

Payment Core owns state transition rules.

Storage enforces:

- Unique constraints.
- Optimistic concurrency.
- Required fields.
- Referential integrity.

Provider adapters do not decide whether a business state transition is allowed.
They only report provider observations.

## Retention

Suggested minimum retention:

- Payment and refund ledger: business-defined legal retention period.
- Provider request records: at least 180 days.
- Webhook event records: at least 180 days.
- Idempotency records: at least 7 days for MVP, longer if caller retry windows
  require it.

Retention must be reviewed against local legal, tax, and payment provider
requirements before production.

