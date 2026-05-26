# Provider Adapter Guide

Current step: Step 2 - Documentation Hardening.

This guide defines how to add or modify payment provider adapters.

## Adapter Goal

A provider adapter translates between UniPay's provider-neutral model and one
payment platform.

Adapters must hide provider-specific details from Core, Gateway, and callers.

## Adapter Responsibilities

Each provider adapter owns:

- Provider request mapping.
- Provider response mapping.
- Provider error mapping.
- Provider status mapping.
- Provider amount formatting.
- Provider signing inputs.
- Provider webhook verification inputs.
- Provider webhook payload parsing.
- Provider documentation references.

Adapters do not own:

- Gateway route definitions.
- API key authentication.
- Core state transition rules.
- Ledger persistence transaction boundaries.
- Secret storage policy.
- Business order fulfillment.

## Required Adapter Operations

MVP provider adapters must support:

| Operation | Required |
| --- | --- |
| Create payment | Yes |
| Query payment | Yes |
| Create refund | Yes |
| Query refund | Yes if provider supports it in MVP scope |
| Verify payment webhook | Yes |
| Verify refund webhook | Required when refund callback is enabled |
| Parse payment webhook | Yes |
| Parse refund webhook | Required when refund callback is enabled |

If a provider cannot support an operation, the adapter must return a stable
`PROVIDER_REJECTED` or `INVALID_CHANNEL` style error with clear capability
metadata.

## Configuration Requirements

Provider configuration must be explicit.

Adapter config should include:

- Provider environment, such as sandbox or production.
- Merchant account identifiers.
- Signing key references.
- Public key or certificate references.
- API endpoint base URL.
- Notify URL defaults.
- Timeout and retry policy references.

Adapters must not read environment variables directly.

## Signing Boundary

Adapters prepare canonical signing inputs and call the signing module.

Rules:

- Low-level cryptographic operations stay in `crates/signing`.
- Adapter code should make canonical message construction visible and tested.
- Key material must not be cloned into long-lived logs or debug output.
- Signing tests must use fixtures from official docs or sandbox-generated data.

## HTTP Boundary

Adapters call providers through the shared HTTP client boundary.

Rules:

- Do not create ad hoc `reqwest` clients inside adapters.
- Do not implement retry loops inside provider-specific request functions unless
  the retry policy is injected and visible.
- All provider calls must produce provider request records through Core or the
  application service layer.

## Mapping Rules

Every adapter must define:

- Provider status to UniPay status mapping.
- Provider error to UniPay error mapping.
- Retryability classification.
- Amount conversion.
- Payment action conversion.
- Webhook event conversion.

Mapping belongs in provider adapter modules and must be tested.

Core should never match raw provider strings.

## Webhook Rules

Every webhook implementation must:

- Accept raw body and original headers.
- Verify signature before parsing trusted fields.
- Derive a deduplication key.
- Return a unified event.
- Avoid side effects outside event conversion.

Ledger updates happen after unified event creation, not inside provider webhook
parsing.

## Adding A New Provider

Required steps:

1. Add official docs to `INTEGRATION_DOCS.md`.
2. Define provider capabilities.
3. Add provider status and error mappings to `PROVIDER_MAPPING.md`.
4. Define provider configuration shape.
5. Implement provider request mapping.
6. Implement provider response mapping.
7. Implement provider webhook verification and parsing.
8. Add unit tests and fixture tests.
9. Add mocked HTTP tests.
10. Add sandbox test checklist.
11. Update `CLIENT_INTEGRATION.md` when caller behavior changes.
12. Update `API_CONTRACT.md` only if a new provider requires a new
    provider-neutral capability.

## Provider Capability Checklist

Before implementation, answer:

- Which payment channels are supported?
- What user action does payment creation return?
- Does provider support merchant order id idempotency?
- Does provider support refund idempotency?
- How are payment statuses represented?
- How are refund statuses represented?
- How are webhooks signed?
- How should webhooks be acknowledged?
- How long does provider retry webhooks?
- What sandbox credentials and test cases exist?
- What provider errors are retryable?
- What provider errors require merchant configuration changes?

## Test Requirements

Adapter test suite must include:

- Request mapping snapshot tests.
- Response mapping tests.
- Status mapping tests.
- Error mapping tests.
- Amount conversion tests.
- Signing canonicalization tests.
- Signature verification tests.
- Webhook duplicate event tests.
- Unknown enum tests.
- Timeout and retry classification tests.

Unknown provider fields must not break parsing unless they affect security or
required business correctness.

## Documentation Requirements

Adapter changes must update:

- `INTEGRATION_DOCS.md` access date and notes.
- `PROVIDER_MAPPING.md` mapping tables.
- `CLIENT_INTEGRATION.md` if caller behavior changes.
- `ERROR_CODES.md` if a new stable UniPay error code is needed.
- `nextsession.md` at session end.

