# Documentation Index

Current step: Step 2 - Documentation Hardening.

This directory is the durable context for UniPay. Future AI sessions should read
this index first, then open the documents relevant to the task.

## Core Documents

| Document | Purpose |
| --- | --- |
| `ARCHITECTURE.md` | Overall module architecture and data flow. |
| `SPEC.md` | Product and system specification. |
| `BUILD.md` | Build, usage, verification, and planned development sequence. |
| `DEPLOYMENT.md` | Docker deployment, compose usage, and environment variables. |
| `nextsession.md` | Current handoff state and next actions. |

## Production Readiness Documents

| Document | Purpose |
| --- | --- |
| `API_CONTRACT.md` | Stable HTTP API contract for other backend systems. |
| `openapi.yaml` | Machine-readable Gateway API draft for client generation and contract tests. |
| `CLIENT_INTEGRATION.md` | Integration guide for Java, Go, Node.js, PHP, Python, Rust, and other callers. |
| `ERROR_CODES.md` | Stable error code catalog and retry guidance. |
| `DATA_MODEL.md` | Payment ledger, refund ledger, webhook event, idempotency, and provider request records. |
| `WEBHOOK_RELIABILITY.md` | Webhook verification, deduplication, acknowledgement, retry, and dead-letter behavior. |
| `SECURITY.md` | Secret management, signing keys, API keys, JWT, logging, compliance, and data protection. |
| `OPERATIONS.md` | Deployment, health checks, metrics, alerts, runbooks, and release safety. |
| `PROVIDER_MAPPING.md` | WeChat Pay and Alipay status, error, amount, and retry mapping. |
| `PROVIDER_ADAPTER_GUIDE.md` | Rules for adding or changing payment provider adapters. |
| `INTEGRATION_DOCS.md` | Official provider documentation links. |

## Reading Order For Implementation

Before starting implementation:

1. Read `Agent.md` in the repository root.
2. Read `ARCHITECTURE.md`.
3. Read `SPEC.md`.
4. Read `API_CONTRACT.md` and `ERROR_CODES.md`.
5. Read `DATA_MODEL.md` and `WEBHOOK_RELIABILITY.md`.
6. Read `SECURITY.md` and `OPERATIONS.md`.
7. Read `PROVIDER_MAPPING.md` and `PROVIDER_ADAPTER_GUIDE.md`.
8. Refresh official provider docs from `INTEGRATION_DOCS.md`.

## Production Gate

The project should not be considered production-ready until:

- The API contract is implemented and covered by contract tests.
- `openapi.yaml` matches `API_CONTRACT.md`.
- Payment, refund, webhook, idempotency, and provider request records are
  persisted.
- Webhooks are verified with raw body, deduplicated, stored, acknowledged
  correctly, and processed asynchronously.
- Provider status and error mapping tables are implemented with tests.
- Secrets are loaded from an approved secret manager or explicitly approved
  secure file path.
- Operations checks, metrics, alerts, and rollback procedures exist.
