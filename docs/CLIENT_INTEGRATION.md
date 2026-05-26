# Client Integration Guide

Current step: Step 2 - Documentation Hardening.

This guide is for backend systems that call UniPay through the HTTP API Gateway.
It is language-neutral and should work for Java, Go, Node.js, PHP, Python, Rust,
and other server-side stacks.

## Integration Model

Business systems should call UniPay instead of directly calling WeChat Pay,
Alipay, or future providers.

```text
Business Backend
  |
  | HTTP + API key
  v
UniPay Gateway
  |
  v
Payment Core
  |
  v
Provider Adapter
```

Backend systems need to understand only:

- UniPay API routes.
- UniPay error codes.
- UniPay payment and refund status values.
- Provider-specific user action returned by `payment_action`.

Backend systems should not implement provider signing, certificate verification,
or webhook decryption.

## Minimum Integration Flow

1. Create a business order in the caller system.
2. Call `POST /v1/payments` with a unique `merchant_order_id`.
3. Show the returned `payment_action` to the user.
4. Receive UniPay webhook or poll `GET /v1/payments/{merchant_order_id}`.
5. Mark the business order paid only after UniPay reports `succeeded`.
6. For refunds, call `POST /v1/refunds` with a unique `merchant_refund_id`.
7. Confirm refund result through webhook or refund query.

## Idempotency Requirements

Payment creation:

- Use one `merchant_order_id` per business order.
- Retry the same request with the same `merchant_order_id`.
- Do not create a new `merchant_order_id` after a timeout until the original
  order has been queried.

Refund creation:

- Use one `merchant_refund_id` per refund attempt.
- Retry the same request with the same `merchant_refund_id`.
- Create a new `merchant_refund_id` only for a new refund attempt.

Recommended header:

```text
Idempotency-Key: <same-value-for-same-business-operation>
```

The idempotency header should match the business operation, not the HTTP retry.

## Authentication

Send the API key as a bearer token:

```text
Authorization: Bearer <api_key>
```

Do not send provider credentials from business systems. WeChat Pay and Alipay
credentials must be configured inside UniPay.

## Payment Action Handling

| `payment_action.type` | Caller behavior |
| --- | --- |
| `qr_code_url` | Render QR code from `value`. Used by WeChat Native payment. |
| `redirect_url` | Redirect user to `value`. |
| `html_form` | Return or render the HTML form according to frontend policy. Common for Alipay web payment. |
| `sdk_payload` | Pass structured payload to a frontend or mobile SDK. |
| `none` | No user action needed. |

Callers should store:

- `payment_id`
- `merchant_order_id`
- `provider`
- `status`
- `payment_action.type`
- Payment action expiry when returned.

Callers should not store provider private keys, signatures, or raw webhook
payloads.

## Status Handling

Payment statuses:

| Status | Terminal | Caller action |
| --- | --- | --- |
| `pending` | No | Wait for user action or webhook. |
| `processing` | No | Poll or wait for webhook. |
| `succeeded` | Yes | Mark business order paid. |
| `failed` | Yes | Mark payment failed; allow new payment attempt if business rules allow. |
| `closed` | Yes | Mark payment closed or expired. |
| `refunding` | No | Payment is paid but refund is in progress. |
| `partially_refunded` | No | Payment remains partially paid. |
| `refunded` | Yes for full amount | Mark fully refunded. |
| `unknown` | No | Query later; do not grant goods. |

Refund statuses:

| Status | Terminal | Caller action |
| --- | --- | --- |
| `pending` | No | Wait or poll. |
| `processing` | No | Wait or poll. |
| `succeeded` | Yes | Mark refund completed. |
| `failed` | Yes | Mark refund failed and inspect error. |
| `closed` | Yes | Mark refund closed. |
| `unknown` | No | Query later. |

## Error Handling

Callers must branch on `error.code`, not `message`.

Basic policy:

| Error group | Caller behavior |
| --- | --- |
| Validation errors | Fix request; do not retry unchanged. |
| Authentication errors | Fix API key or caller permission. |
| Idempotency conflict | Query existing payment or refund. |
| Provider timeout | Query before retrying create/refund. |
| Provider unavailable | Retry with backoff using same idempotency key. |
| Signature/webhook errors | Treat as security incident if seen outside webhook route. |
| Internal error | Retry with backoff if `retryable` is true; contact UniPay owner with `trace_id`. |

See `ERROR_CODES.md` for the canonical catalog.

## Timeout And Retry Policy For Callers

Recommended caller HTTP timeout:

- Connect timeout: 3 seconds.
- Total timeout: 10 seconds for create/query/refund.
- Webhook handling timeout depends on caller's own endpoint if UniPay forwards
  events in a future phase.

Retry policy:

- Retry `429`, `503`, and `504` with exponential backoff and jitter.
- Retry network timeouts only with the same idempotency key.
- Do not retry `400`, `401`, `403`, `404`, or `422` without changing input.
- For `409`, query the existing resource before deciding.

## Example Payment Flow

Create payment:

```text
POST /v1/payments
Authorization: Bearer <api_key>
Idempotency-Key: order_202605260001
```

Request body:

```json
{
  "provider": "wechat",
  "merchant_order_id": "order_202605260001",
  "amount": {
    "currency": "CNY",
    "amount_minor": 100
  },
  "subject": "Order 202605260001",
  "channel": "native"
}
```

Caller behavior:

- If `success=true`, render the payment action.
- If timeout happens, query `/v1/payments/order_202605260001?provider=wechat`.
- If the query returns `PAYMENT_NOT_FOUND`, retry create payment with the same
  merchant order id and idempotency key.

## Example Refund Flow

Create refund:

```text
POST /v1/refunds
Authorization: Bearer <api_key>
Idempotency-Key: refund_202605260001_001
```

Request body:

```json
{
  "provider": "wechat",
  "merchant_order_id": "order_202605260001",
  "merchant_refund_id": "refund_202605260001_001",
  "amount": {
    "currency": "CNY",
    "amount_minor": 100
  },
  "reason": "Customer requested refund"
}
```

Caller behavior:

- If `status=succeeded`, mark refund complete.
- If `status=processing`, wait for webhook or poll query.
- If timeout happens, query by `merchant_refund_id` before retry.

## Sandbox Checklist

Before production:

1. Configure sandbox credentials in UniPay.
2. Create one successful WeChat Native payment test.
3. Create one successful Alipay web payment test.
4. Verify payment webhook processing.
5. Verify duplicate webhook processing.
6. Verify payment query after timeout simulation.
7. Verify refund creation and refund query.
8. Verify API key rejection.
9. Verify request validation errors.
10. Verify caller can use `trace_id` to locate logs.

## Production Checklist For Callers

Before going live:

- Store UniPay API key in the caller's secret manager.
- Use production UniPay Gateway URL.
- Configure caller timeout and retry policy.
- Store `merchant_order_id` and `merchant_refund_id`.
- Treat `unknown` status as not paid or not refunded.
- Never grant goods based only on create-payment success.
- Use webhook or query result to confirm final state.
- Monitor payment and refund failure rates.

