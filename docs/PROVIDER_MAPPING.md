# Provider Mapping

Current step: Step 2 - Documentation Hardening.

This document defines provider-specific mapping rules that must stay inside
provider adapters. Core and Gateway expose only UniPay concepts.

Official provider docs must be refreshed from `INTEGRATION_DOCS.md` before
implementation.

## Unified Payment Status

| UniPay status | Meaning |
| --- | --- |
| `pending` | Payment was created and waits for user or provider action. |
| `processing` | Provider has accepted or is processing the payment. |
| `succeeded` | Payment is confirmed successful. |
| `failed` | Payment failed and will not complete. |
| `closed` | Payment was closed, cancelled, or expired. |
| `refunding` | One or more refunds are in progress. |
| `partially_refunded` | Some amount has been refunded. |
| `refunded` | Full amount has been refunded. |
| `unknown` | State cannot be determined safely. |

## Unified Refund Status

| UniPay status | Meaning |
| --- | --- |
| `pending` | Refund was created locally and waits for provider call or provider processing. |
| `processing` | Provider is processing refund. |
| `succeeded` | Refund completed successfully. |
| `failed` | Refund failed. |
| `closed` | Refund closed and will not complete. |
| `unknown` | State cannot be determined safely. |

## WeChat Pay v3 Mapping

### Payment Status Mapping

Expected WeChat trade states should map as follows. Verify exact enum names from
official docs before implementation.

| WeChat trade state | UniPay status | Terminal | Notes |
| --- | --- | --- | --- |
| `SUCCESS` | `succeeded` | Yes | Payment completed. |
| `REFUND` | `refunding` or `refunded` | No | Query refund records to determine partial/full state. |
| `NOTPAY` | `pending` | No | User has not paid. |
| `CLOSED` | `closed` | Yes | Order closed. |
| `REVOKED` | `closed` | Yes | Mostly payment-code style; not expected for Native MVP but handle safely. |
| `USERPAYING` | `processing` | No | User payment in progress. |
| `PAYERROR` | `failed` | Yes | Payment failed. |
| Unknown value | `unknown` | No | Do not grant goods. Log and query provider. |

### Refund Status Mapping

Expected WeChat refund states should map as follows. Verify exact enum names from
official docs before implementation.

| WeChat refund status | UniPay refund status | Terminal | Notes |
| --- | --- | --- | --- |
| `SUCCESS` | `succeeded` | Yes | Refund completed. |
| `CLOSED` | `closed` | Yes | Refund closed. |
| `PROCESSING` | `processing` | No | Refund in progress. |
| `ABNORMAL` | `failed` or `unknown` | No | Requires operator handling or abnormal refund flow. |
| Unknown value | `unknown` | No | Query provider and alert if repeated. |

### Amount Mapping

UniPay:

```text
Money { currency: "CNY", amount_minor: 100 }
```

WeChat:

- Amount is represented in fen for CNY.
- Adapter maps `amount_minor` directly to provider integer amount for CNY.
- Non-CNY support must be verified before enabling.

### Error Mapping

| WeChat error example | UniPay code | Retryable | Notes |
| --- | --- | --- | --- |
| `OUT_TRADE_NO_USED` | `DUPLICATE_PROVIDER_ORDER` | No | Query existing order. |
| `SIGN_ERROR` | `SIGNATURE_CREATE_FAILED` or `PROVIDER_REJECTED` | No | If request signing bug, service incident. |
| `NO_AUTH` | `PROVIDER_REJECTED` | No | Merchant lacks permission. |
| `FREQUENCY_LIMITED` | `PROVIDER_RATE_LIMITED` | Yes | Retry with backoff. |
| `SYSTEM_ERROR` | `PROVIDER_UNAVAILABLE` | Yes | Retry safely with same idempotency key after query when needed. |
| Transport timeout | `PROVIDER_TIMEOUT` | Yes | Query before retrying create/refund. |

### Webhook Mapping

WeChat webhook adapter must:

- Verify signature using original headers and raw body.
- Validate timestamp and nonce according to official docs.
- Decrypt `resource` with API v3 key.
- Derive deduplication key from provider transaction id or event metadata.
- Map decrypted trade state to UniPay status.
- Acknowledge duplicate valid callbacks successfully.

## Alipay Mapping

### Payment Status Mapping

Expected Alipay trade statuses should map as follows. Verify exact enum names
from official docs before implementation.

| Alipay trade status | UniPay status | Terminal | Notes |
| --- | --- | --- | --- |
| `WAIT_BUYER_PAY` | `pending` | No | User has not paid. |
| `TRADE_CLOSED` | `closed` | Yes | Closed or fully refunded depending context; inspect refund state. |
| `TRADE_SUCCESS` | `succeeded` | Yes | Payment successful and can be refunded. |
| `TRADE_FINISHED` | `succeeded` | Yes | Payment completed and may not support refund depending product rules. |
| Unknown value | `unknown` | No | Do not grant goods. Query provider. |

### Refund Status Mapping

Alipay refund APIs may report refund result through API response and async
notification depending product configuration. The adapter must map provider
response fields into UniPay refund states after official-doc verification.

Default policy:

| Alipay refund observation | UniPay refund status | Terminal |
| --- | --- | --- |
| Provider confirms refund success | `succeeded` | Yes |
| Provider reports refund processing | `processing` | No |
| Provider rejects refund for business reason | `failed` | Yes |
| Provider result ambiguous | `unknown` | No |

### Amount Mapping

UniPay stores integer minor units.

Alipay commonly uses decimal amount strings in yuan for CNY provider requests.
Adapter responsibility:

- Convert `amount_minor=100` and `currency=CNY` to provider decimal `1.00`.
- Reject fractional minor-unit input before conversion.
- Parse provider decimal amounts back into integer minor units.
- Never use floating point types for conversion.

### Error Mapping

| Alipay error example | UniPay code | Retryable | Notes |
| --- | --- | --- | --- |
| Invalid signature | `SIGNATURE_VERIFY_FAILED` | No | Reject provider response or webhook. |
| Business parameter invalid | `PROVIDER_REJECTED` | No | Fix request mapping or caller input. |
| Duplicate merchant order | `DUPLICATE_PROVIDER_ORDER` | No | Query existing order. |
| Rate limited | `PROVIDER_RATE_LIMITED` | Yes | Retry with backoff. |
| System busy or timeout | `PROVIDER_UNAVAILABLE` or `PROVIDER_TIMEOUT` | Yes | Query before retrying create/refund. |

### Webhook Mapping

Alipay webhook adapter must:

- Preserve raw notification fields needed for signature verification.
- Verify RSA2 signature according to official docs.
- Confirm notification response body requirements from official docs.
- Derive deduplication key from `notify_id`, `trade_no`, or canonical hash
  depending official payload.
- Map `trade_status` to UniPay status.
- Treat duplicate valid callbacks as success acknowledgements.

## Mapping Test Requirements

Each provider adapter must include tests for:

- Every known provider payment status.
- Every known provider refund status.
- Unknown provider status.
- Amount conversion.
- Provider error to UniPay error code.
- Retryability classification.
- Duplicate webhook event mapping.

