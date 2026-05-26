# Error Codes

Current step: Step 2 - Documentation Hardening.

This document defines stable UniPay error codes for SDK and Gateway callers.
Callers must branch on `code`, not on human-readable `message`.

## Error Response Shape

```json
{
  "success": false,
  "error": {
    "code": "PROVIDER_TIMEOUT",
    "message": "Payment provider timed out.",
    "provider": "wechat",
    "retryable": true,
    "operation": "create_payment"
  },
  "trace_id": "req_01JZ0000000000000000000000"
}
```

## Canonical Error Codes

| Code | HTTP status | Retryable | Meaning | Caller action |
| --- | --- | --- | --- | --- |
| `INVALID_REQUEST` | `400` | No | Request body, query, or path is malformed. | Fix request. |
| `INVALID_AMOUNT` | `400` | No | Amount is zero, negative, malformed, or unsupported for currency. | Fix amount. |
| `INVALID_PROVIDER` | `400` | No | Provider is missing or unsupported. | Use supported provider. |
| `INVALID_CHANNEL` | `400` | No | Channel is not supported by provider. | Use supported channel. |
| `INVALID_CURRENCY` | `400` | No | Currency is unsupported by provider or route. | Use supported currency. |
| `MISSING_IDEMPOTENCY_KEY` | `400` | No | Required idempotency key is missing. | Send idempotency key. |
| `UNAUTHORIZED` | `401` | No | API key is missing or invalid. | Fix credentials. |
| `FORBIDDEN` | `403` | No | Caller is authenticated but not allowed. | Request permission. |
| `PAYMENT_NOT_FOUND` | `404` | No | Payment does not exist in UniPay ledger. | Check merchant order id. |
| `REFUND_NOT_FOUND` | `404` | No | Refund does not exist in UniPay ledger. | Check merchant refund id. |
| `IDEMPOTENCY_CONFLICT` | `409` | No | Same idempotency key was reused with different parameters. | Query existing resource and fix caller logic. |
| `PAYMENT_STATE_CONFLICT` | `409` | No | Operation is invalid for current payment state. | Query payment and follow current state. |
| `REFUND_STATE_CONFLICT` | `409` | No | Operation is invalid for current refund state. | Query refund and follow current state. |
| `DUPLICATE_PROVIDER_ORDER` | `409` | No | Provider reports merchant order id already used. | Query existing payment. |
| `REFUND_AMOUNT_EXCEEDED` | `422` | No | Refund amount exceeds refundable amount. | Fix refund amount. |
| `PAYMENT_EXPIRED` | `422` | No | Payment can no longer be completed. | Create new business payment attempt if allowed. |
| `PROVIDER_REJECTED` | `422` | No | Provider rejected request for business reason. | Inspect provider-safe detail and fix input or merchant setup. |
| `PROVIDER_RATE_LIMITED` | `429` | Yes | Provider or UniPay rate limit exceeded. | Retry with backoff. |
| `SIGNATURE_CREATE_FAILED` | `500` | No | UniPay failed to sign provider request. | Treat as service incident. |
| `SIGNATURE_VERIFY_FAILED` | `400` for webhook, `502` for provider response | No | Signature verification failed. | Reject webhook or treat provider response as invalid. |
| `WEBHOOK_PAYLOAD_INVALID` | `400` | No | Webhook body cannot be parsed after verification. | Reject webhook and inspect logs. |
| `WEBHOOK_REPLAY_SUSPECTED` | `400` | No | Webhook timestamp, nonce, or event id suggests replay. | Reject and alert. |
| `PROVIDER_TIMEOUT` | `504` | Yes | Provider request timed out. | Query before retrying create/refund. |
| `PROVIDER_UNAVAILABLE` | `503` | Yes | Provider is unavailable or returned transient failure. | Retry with backoff and same idempotency key. |
| `PROVIDER_BAD_RESPONSE` | `502` | Maybe | Provider response is malformed or unverifiable. | Query later; alert if repeated. |
| `TRANSPORT_ERROR` | `503` | Yes | Network or DNS failure. | Retry with backoff and same idempotency key. |
| `CONFIGURATION_ERROR` | `500` | No | UniPay provider or gateway config is invalid. | Service owner must fix config. |
| `SECRET_UNAVAILABLE` | `500` | Maybe | Required secret or key cannot be loaded. | Service owner must inspect secret manager. |
| `DATABASE_UNAVAILABLE` | `503` | Yes | Ledger or idempotency store is unavailable. | Retry later; do not call provider if ledger write failed. |
| `INTERNAL_ERROR` | `500` | Maybe | Unexpected UniPay failure. | Retry only if `retryable=true`; report `trace_id`. |

## Provider Error Mapping Policy

Provider errors must be mapped in two layers:

1. Provider adapter maps raw provider error into `ProviderErrorDetail`.
2. Core maps `ProviderErrorDetail` into stable UniPay `code`.

Public responses may include safe provider detail:

```json
{
  "provider_code": "OUT_TRADE_NO_USED",
  "provider_message": "Provider reports merchant order id already used."
}
```

Rules:

- Do not expose raw provider response bodies by default.
- Do not expose signatures, authorization headers, keys, certificates, or
  personally identifiable data.
- Preserve raw provider error details only in restricted internal logs or
  provider request records with redaction.

## Retryability Rules

Retryable:

- `PROVIDER_RATE_LIMITED`
- `PROVIDER_TIMEOUT`
- `PROVIDER_UNAVAILABLE`
- `TRANSPORT_ERROR`
- `DATABASE_UNAVAILABLE` after confirming provider was not called
- `INTERNAL_ERROR` only when explicitly marked retryable

Not retryable without changes:

- `INVALID_REQUEST`
- `INVALID_AMOUNT`
- `INVALID_PROVIDER`
- `INVALID_CHANNEL`
- `INVALID_CURRENCY`
- `UNAUTHORIZED`
- `FORBIDDEN`
- `IDEMPOTENCY_CONFLICT`
- `PAYMENT_STATE_CONFLICT`
- `REFUND_STATE_CONFLICT`
- `REFUND_AMOUNT_EXCEEDED`
- `SIGNATURE_VERIFY_FAILED`
- `WEBHOOK_REPLAY_SUSPECTED`

Unknown result:

- `PROVIDER_TIMEOUT` during create payment or refund may mean the provider
  accepted the request. Query before retrying.
- `TRANSPORT_ERROR` after request body was sent may mean the provider accepted
  the request. Query before retrying.

## SDK Error Requirements

SDK errors should expose:

- Stable UniPay code.
- Provider, when known.
- Operation.
- Retryability.
- Safe message.
- Source error for internal diagnostics.

SDK errors must not force callers to match provider-specific strings.

