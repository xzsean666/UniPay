# API Contract

Current step: Step 2 - Documentation Hardening.

This document defines the stable HTTP API surface for backend systems that call
UniPay through the Payment API Gateway.

The API is provider-neutral. Provider-specific fields must stay inside UniPay
unless explicitly exposed as metadata for diagnostics.

## Versioning

All production routes must be versioned:

```text
/v1
```

MVP route set:

| Method | Path | Purpose |
| --- | --- | --- |
| `POST` | `/v1/payments` | Create a payment. |
| `GET` | `/v1/payments/{merchant_order_id}` | Query payment by merchant order id. |
| `POST` | `/v1/refunds` | Create a refund. |
| `GET` | `/v1/refunds/{merchant_refund_id}` | Query refund by merchant refund id. |
| `POST` | `/v1/webhooks/{provider}/payments` | Receive provider payment webhook. |
| `POST` | `/v1/webhooks/{provider}/refunds` | Receive provider refund webhook. |
| `GET` | `/v1/health/live` | Liveness check. |
| `GET` | `/v1/health/ready` | Readiness check. |

Compatibility rules:

- Do not remove response fields from `v1`.
- Do not change field meaning in `v1`.
- New optional fields are allowed.
- New enum values are allowed only when clients are instructed to treat unknown
  values as non-terminal.
- Breaking changes require `v2`.

## Authentication

MVP authentication uses API keys.

Required header:

```text
Authorization: Bearer <api_key>
```

Optional request tracing header:

```text
X-Request-Id: <caller_trace_id>
```

Required idempotency header for `POST /v1/payments` and `POST /v1/refunds`:

```text
Idempotency-Key: <caller_idempotency_key>
```

Rules:

- API keys must be checked before request validation calls provider logic.
- API keys must never be logged.
- Invalid or missing API keys return `401`.
- Authenticated callers may still receive `403` when caller-specific provider
  access control is introduced.
- `Idempotency-Key` must be stable for the same business operation and must not
  be reused with different request parameters.

## Common Request Rules

Content type:

```text
Content-Type: application/json
```

Encoding:

- UTF-8 JSON for Gateway business APIs.
- Provider webhooks must preserve raw request body bytes before parsing.

Required client behavior:

- Set a unique `merchant_order_id` for each business order.
- Set a unique `merchant_refund_id` for each refund attempt.
- Reuse the same idempotency value when retrying the same request.
- Treat `409 IDEMPOTENCY_CONFLICT` as a signal to query the existing resource
  and inspect caller-side retry behavior.
- Treat network timeout as unknown state and query before creating a new payment
  or refund.

## Common Response Envelope

Success response:

```json
{
  "success": true,
  "data": {},
  "trace_id": "req_01JZ0000000000000000000000"
}
```

Error response:

```json
{
  "success": false,
  "error": {
    "code": "PROVIDER_UNAVAILABLE",
    "message": "Payment provider is temporarily unavailable.",
    "provider": "wechat",
    "retryable": true,
    "operation": "create_payment"
  },
  "trace_id": "req_01JZ0000000000000000000000"
}
```

Rules:

- `message` must be safe to show to backend developers.
- Internal diagnostic details belong in logs, not in public responses.
- `trace_id` must be included in every response.
- `provider` is omitted when the error happens before provider selection.

## Money Object

Money is always represented as integer minor units.

```json
{
  "currency": "CNY",
  "amount_minor": 100
}
```

Rules:

- Floating point amounts are not accepted.
- `amount_minor` must be positive for payment creation.
- Refund amount must be positive and cannot exceed refundable amount.
- Provider adapters perform provider-specific formatting internally.

## Create Payment

```text
POST /v1/payments
```

Request:

```json
{
  "provider": "wechat",
  "merchant_order_id": "order_202605260001",
  "amount": {
    "currency": "CNY",
    "amount_minor": 100
  },
  "subject": "Order 202605260001",
  "description": "Payment for order 202605260001",
  "channel": "native",
  "notify_url": "https://merchant.example.com/payment-callback",
  "expire_at": "2026-05-26T12:30:00+08:00",
  "metadata": {
    "business_user_id": "user_10001"
  }
}
```

Required fields:

- `provider`
- `merchant_order_id`
- `amount`
- `subject`
- `channel`

Provider and channel rules:

| Provider | MVP channels |
| --- | --- |
| `wechat` | `native` |
| `alipay` | `web` |

Successful response:

```json
{
  "success": true,
  "data": {
    "payment_id": "pay_01JZ0000000000000000000000",
    "provider": "wechat",
    "merchant_order_id": "order_202605260001",
    "provider_transaction_id": null,
    "status": "pending",
    "amount": {
      "currency": "CNY",
      "amount_minor": 100
    },
    "payment_action": {
      "type": "qr_code_url",
      "value": "weixin://wxpay/bizpayurl?pr=example"
    },
    "expires_at": "2026-05-26T12:30:00+08:00",
    "created_at": "2026-05-26T12:00:00+08:00"
  },
  "trace_id": "req_01JZ0000000000000000000000"
}
```

Payment action types:

| Type | Meaning |
| --- | --- |
| `qr_code_url` | Caller should render a QR code from the URL. |
| `redirect_url` | Caller should redirect user to this URL. |
| `html_form` | Caller should return or render provider-generated HTML form. |
| `sdk_payload` | Caller should pass payload to a client SDK. |
| `none` | No client-side action is required. |

## Query Payment

```text
GET /v1/payments/{merchant_order_id}?provider=wechat
```

Successful response:

```json
{
  "success": true,
  "data": {
    "payment_id": "pay_01JZ0000000000000000000000",
    "provider": "wechat",
    "merchant_order_id": "order_202605260001",
    "provider_transaction_id": "4200000000202605260000000001",
    "status": "succeeded",
    "amount": {
      "currency": "CNY",
      "amount_minor": 100
    },
    "paid_at": "2026-05-26T12:02:00+08:00",
    "updated_at": "2026-05-26T12:02:01+08:00"
  },
  "trace_id": "req_01JZ0000000000000000000000"
}
```

Rules:

- Query should return the local ledger state after optionally refreshing from
  provider, depending on implementation policy.
- A missing local payment returns `PAYMENT_NOT_FOUND`.
- Provider timeout during refresh should not erase known local state.

## Create Refund

```text
POST /v1/refunds
```

Request:

```json
{
  "provider": "wechat",
  "merchant_order_id": "order_202605260001",
  "merchant_refund_id": "refund_202605260001_001",
  "amount": {
    "currency": "CNY",
    "amount_minor": 100
  },
  "reason": "Customer requested refund",
  "notify_url": "https://merchant.example.com/refund-callback",
  "metadata": {
    "operator_id": "ops_10001"
  }
}
```

Required fields:

- `provider`
- `merchant_order_id`
- `merchant_refund_id`
- `amount`

Successful response:

```json
{
  "success": true,
  "data": {
    "refund_id": "rfd_01JZ0000000000000000000000",
    "provider": "wechat",
    "merchant_order_id": "order_202605260001",
    "merchant_refund_id": "refund_202605260001_001",
    "provider_refund_id": "5030000000202605260000000001",
    "status": "processing",
    "amount": {
      "currency": "CNY",
      "amount_minor": 100
    },
    "created_at": "2026-05-26T12:10:00+08:00"
  },
  "trace_id": "req_01JZ0000000000000000000000"
}
```

Rules:

- Refund is allowed only for a succeeded or partially refunded payment.
- Refund amount cannot exceed remaining refundable amount.
- Retrying the same refund must reuse `merchant_refund_id`.

## Query Refund

```text
GET /v1/refunds/{merchant_refund_id}?provider=wechat
```

Successful response:

```json
{
  "success": true,
  "data": {
    "refund_id": "rfd_01JZ0000000000000000000000",
    "provider": "wechat",
    "merchant_order_id": "order_202605260001",
    "merchant_refund_id": "refund_202605260001_001",
    "provider_refund_id": "5030000000202605260000000001",
    "status": "succeeded",
    "amount": {
      "currency": "CNY",
      "amount_minor": 100
    },
    "succeeded_at": "2026-05-26T12:11:00+08:00",
    "updated_at": "2026-05-26T12:11:01+08:00"
  },
  "trace_id": "req_01JZ0000000000000000000000"
}
```

## Webhook Routes

```text
POST /v1/webhooks/{provider}/payments
POST /v1/webhooks/{provider}/refunds
```

Rules:

- Webhook routes do not use business API key authentication.
- Provider signature verification is mandatory.
- Raw body and original headers must be captured before parsing.
- Current local implementation fails closed with `SIGNATURE_VERIFY_FAILED`
  until production provider verifiers are configured; it must never accept a
  provider webhook by parsing JSON alone.
- The route should persist the webhook event and acknowledge provider only after
  durable storage succeeds.
- Business processing should run asynchronously after acknowledgement.
- Duplicate webhook events must return a provider-success acknowledgement after
  deduplication.

Webhook public response depends on provider requirements:

- WeChat Pay v3 expects a success-style JSON response after successful handling.
- Alipay async notifications must follow Alipay's documented success response.

Exact provider webhook acknowledgement bodies must be verified from
`INTEGRATION_DOCS.md` before implementation.

## HTTP Status Code Policy

| HTTP status | Meaning |
| --- | --- |
| `200` | Request accepted and processed successfully, or duplicate webhook safely acknowledged. |
| `201` | Resource created when route semantics require it. |
| `202` | Request accepted for asynchronous processing. |
| `400` | Request shape or value is invalid. |
| `401` | Missing or invalid authentication. |
| `403` | Authenticated caller lacks permission. |
| `404` | Requested payment or refund does not exist. |
| `409` | Idempotency conflict or state conflict. |
| `422` | Business rule rejected request. |
| `429` | Rate limit exceeded. |
| `500` | Internal error. |
| `502` | Provider returned invalid or failed response. |
| `503` | Provider or dependency unavailable. |
| `504` | Provider or dependency timeout. |

## Contract Test Requirements

Implementation must include contract tests for:

- Required field validation.
- Stable response envelope.
- Error code and HTTP status mapping.
- Idempotency replay.
- Unknown enum handling.
- Webhook raw-body preservation.
- Versioned route compatibility.
