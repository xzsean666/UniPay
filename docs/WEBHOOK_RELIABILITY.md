# Webhook Reliability

Current step: Step 2 - Documentation Hardening.

This document defines production webhook behavior for provider callbacks.

## Core Rule

Webhook processing must be durable, idempotent, and safe under duplicate,
delayed, reordered, and malicious requests.

Provider webhooks are not trusted until signature verification succeeds.

## Processing Pipeline

```text
Provider webhook
  |
  v
Capture raw body and original headers
  |
  v
Verify provider signature and timestamp
  |
  v
Derive deduplication key
  |
  v
Persist webhook event
  |
  v
Acknowledge provider
  |
  v
Process event asynchronously
  |
  v
Update payment or refund ledger
```

## Route Rules

Webhook routes:

- `POST /v1/webhooks/{provider}/payments`
- `POST /v1/webhooks/{provider}/refunds`

Rules:

- Do not require UniPay API key on provider webhook routes.
- Do require provider signature verification.
- Preserve exact raw body bytes before parsing.
- Preserve original provider signature headers.
- Reject unsupported provider path values.
- Apply request size limits.
- Apply source rate limits that do not block legitimate provider retries.

## Verification Requirements

Verification uses:

- Raw body bytes.
- Original provider headers.
- Provider public key, platform certificate, or configured webhook secret.
- Timestamp and nonce when provider supports them.

Verification result:

- Invalid signature: return provider failure response and do not process.
- Expired timestamp: return provider failure response and alert if repeated.
- Valid signature: continue to deduplication and durable storage.

Provider-specific rules must be implemented from `PROVIDER_MAPPING.md` and
official docs in `INTEGRATION_DOCS.md`.

## Deduplication

Preferred deduplication key:

```text
provider + provider_event_id
```

Fallback deduplication key:

```text
provider + event_type + provider_transaction_id + status + raw_body_hash
```

Rules:

- Deduplication must happen after signature verification.
- Duplicate valid events must not re-apply business state changes.
- Duplicate valid events should return provider success acknowledgement.
- Duplicate invalid events should still be rejected.

## Acknowledgement Policy

The webhook handler may acknowledge provider after:

1. Signature verification succeeds.
2. Deduplication lookup completes.
3. Webhook event is durably persisted or known duplicate is found.

The handler must not wait for slow downstream business processing before
acknowledging provider.

Reason:

- Provider retry windows are outside UniPay control.
- Slow business processing can cause duplicate callbacks.
- Durable event storage allows asynchronous recovery.

Provider acknowledgement body:

- WeChat Pay and Alipay have provider-specific success response requirements.
- Confirm exact body from `INTEGRATION_DOCS.md` immediately before
  implementation.

## Asynchronous Processing

After acknowledgement, an internal worker processes verified events.

Worker responsibilities:

- Load webhook event.
- Parse verified payload.
- Map provider payload to unified event.
- Load payment or refund ledger record.
- Apply state transition with concurrency protection.
- Mark webhook event processed.

Failure behavior:

- Retry transient storage or concurrency failures.
- Do not retry deterministic parsing failures indefinitely.
- Move exhausted events to dead-letter state.
- Emit alert on dead-letter event.

## Ordering

Provider events may arrive:

- More than once.
- Out of order.
- After a query already updated state.
- After payment or refund is already terminal.

Rules:

- State transitions must be monotonic.
- Terminal states should not be overwritten by older non-terminal events.
- A webhook that conflicts with local terminal state should be recorded and
  classified for investigation, not blindly applied.
- Query provider when event ordering creates ambiguity.

## Replay Protection

Replay protection should use:

- Provider timestamp header when available.
- Provider nonce header when available.
- Deduplication key.
- Raw body hash.
- Configured timestamp tolerance.

Suggested timestamp tolerance:

- 5 minutes for providers with reliable timestamp signatures.
- Provider-specific tolerance must be verified from official docs.

Repeated replay failures should trigger security alerts.

## Dead Letter Policy

Move event to `dead_lettered` when:

- Maximum processing attempts are exceeded.
- Payload is verified but cannot be mapped.
- Required related payment or refund is missing after retry window.
- State conflict requires manual investigation.

Dead-letter record must preserve:

- Provider.
- Event type.
- Deduplication key.
- Trace id.
- Last error code.
- Processing attempt count.

Operators must have a runbook in `OPERATIONS.md` for dead-letter recovery.

## Testing Requirements

Tests must cover:

- Raw body verification.
- Invalid signature rejection.
- Timestamp replay rejection.
- Duplicate valid webhook acknowledgement.
- Duplicate valid webhook does not double-apply state.
- Out-of-order event handling.
- Dead-letter transition.
- Provider-specific acknowledgement body.

