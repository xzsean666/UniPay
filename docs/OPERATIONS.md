# Operations

Current step: Step 2 - Documentation Hardening.

This document defines production operation requirements for UniPay.

## Runtime Services

Planned runtime components:

- UniPay Gateway HTTP service.
- Internal webhook processing worker.
- Database for ledger, idempotency, provider request records, and webhook events.
- Secret manager or secure key storage.
- Metrics and logging backend.

The SDK can be embedded in business systems, but the Gateway deployment should be
operated as a service when other backend systems need HTTP access.

## Health Checks

Routes:

- `GET /v1/health/live`
- `GET /v1/health/ready`

Liveness:

- Returns success when process is running and event loop is responsive.
- Must not require provider network access.

Readiness:

- Database reachable.
- Required configuration loaded.
- Required secrets loaded or accessible.
- Provider adapter configuration valid.
- Background worker queue reachable when enabled.

Readiness should fail closed when critical dependencies are unavailable.

## Metrics

Required metrics:

- HTTP request count by route, status, and caller.
- HTTP request latency by route.
- Provider request count by provider, operation, and result.
- Provider request latency by provider and operation.
- Payment create count by provider and status.
- Refund create count by provider and status.
- Webhook received count by provider and event type.
- Webhook verification failure count.
- Webhook duplicate count.
- Webhook dead-letter count.
- Idempotency conflict count.
- Database error count.
- Secret load or key rotation error count.

Avoid high-cardinality labels:

- Do not label metrics with merchant order id.
- Do not label metrics with provider transaction id.
- Do not label metrics with raw error message.

## Logs

Logs must include:

- `trace_id`
- `operation`
- `provider` when known
- `caller_id` when authenticated
- Stable error code
- Retryability
- Duration

Logs must not include secrets or unredacted provider payloads. See
`SECURITY.md`.

## Alerts

Minimum alerts:

| Alert | Severity | Reason |
| --- | --- | --- |
| Gateway readiness failure | Critical | Service cannot safely receive traffic. |
| Payment create error rate high | Critical | Business payment impact. |
| Refund create error rate high | Critical | Customer support and financial impact. |
| Provider timeout rate high | High | Provider or network degradation. |
| Webhook verification failures spike | High | Possible attack or config drift. |
| Webhook dead-letter count > 0 | High | Payment state may be stale. |
| Idempotency conflict spike | Medium | Caller retry bug or abuse. |
| Certificate near expiry | Critical | Provider calls or verification may fail. |
| Secret load failure | Critical | Signing/auth may fail. |
| Database unavailable | Critical | Do not call provider without durable ledger. |

## Deployment

Deployment requirements:

- Graceful shutdown.
- Stop accepting new requests before shutdown.
- Finish in-flight requests within configured timeout.
- Background worker should checkpoint progress.
- Readiness should turn false before process exits.
- Rollback procedure must be tested.

Configuration should be environment-specific:

- Local.
- Sandbox.
- Production.

Production deployment must not use sandbox provider endpoints or credentials.

## Rate Limiting

Gateway should support:

- Per-caller API rate limits.
- Per-provider outbound concurrency limits.
- Webhook route request size limits.
- Webhook route body read timeout.

Rate limiting must not break provider retry behavior. For webhooks, prefer
verification and deduplication over aggressive blocking.

## Runbooks

### Provider Timeout Spike

1. Check provider-specific metrics.
2. Check network and DNS health.
3. Check provider status page if available.
4. Pause aggressive retries if they amplify load.
5. Ensure create/refund retries reuse idempotency keys.
6. Query provider for unknown create/refund outcomes.

### Webhook Dead Letter

1. Locate event by `trace_id` or `webhook_event_id`.
2. Confirm signature verification result.
3. Inspect sanitized error code.
4. Query provider for current payment or refund state.
5. Apply manual repair only through documented state transition tool.
6. Mark event resolved with operator identity.

### Duplicate Charge Suspected

1. Search by merchant order id.
2. Check local `payments` unique constraints.
3. Check provider transaction ids.
4. Query provider by merchant order id.
5. Do not create a new payment until state is known.
6. Escalate to provider support if records disagree.

### Key Or Certificate Expiry

1. Identify affected provider and merchant account.
2. Load replacement key or certificate.
3. Validate signing in sandbox if possible.
4. Deploy or rotate configuration.
5. Confirm readiness and provider calls succeed.
6. Revoke old key after overlap period.

## Backup And Recovery

Database backup requirements:

- Regular backups.
- Restore test before production.
- Point-in-time recovery if supported.
- Backup encryption.
- Access controls for backup data.

Recovery rule:

- After restore, reconcile local payment/refund state against provider queries or
  provider bills before accepting high-risk operations.

## Release Gate

Before production release:

- Contract tests pass.
- Unit and integration tests pass.
- Webhook duplicate and replay tests pass.
- Migration rollback tested.
- Readiness and liveness checked.
- Metrics and alerts installed.
- Secret rotation path tested.
- Provider sandbox tests completed.
- Runbooks reviewed.

