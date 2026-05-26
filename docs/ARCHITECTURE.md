# Architecture Design

Current step: Step 1 - Architecture Design.

This document defines the architecture for the Rust unified payment SDK and API
Gateway. It intentionally contains no implementation code.

## System Goal

Build a Rust payment infrastructure with two public integration surfaces:

- Payment SDK for direct Rust integration
- Payment API Gateway for HTTP integration

Both surfaces must use the same payment core so business systems do not depend
on payment-provider-specific behavior.

## Overall Architecture

```text
Business System
  |
  |-- Rust SDK integration ------------------|
  |                                          |
  |-- HTTP integration --> Payment Gateway --|
                                             |
                                             v
                                      Payment Core
                                             |
            -----------------------------------------------------
            |                    |                 |            |
       WeChat Pay v3          Alipay           Stripe       Future providers
```

The SDK and Gateway are entry points. Payment Core owns the unified payment
model, provider abstraction, signing boundary, webhook model, and error model.
Provider adapters translate unified requests into provider-specific requests.

## Planned Directory Structure

```text
UniPay/
  Agent.md
  crates/
    common/
    core/
    signing/
    http-client/
    wechat/
    alipay/
    stripe/
  gateway/
    api/
    auth/
    app/
  examples/
  docs/
    ARCHITECTURE.md
    SPEC.md
    BUILD.md
    INTEGRATION_DOCS.md
    nextsession.md
```

The first implementation phase should create only the modules required for
WeChat Pay v3, Alipay, and the minimal gateway. Stripe is shown as an extension
boundary, not as an MVP requirement.

## Module Breakdown

| Module | Purpose | Input | Output | Dependencies |
| --- | --- | --- | --- | --- |
| `crates/common` | Shared value objects and low-level helpers that are not payment-provider-specific. | Primitive values, serialized data, timestamps. | Validated shared types and helper results. | Standard library, small shared dependencies only. |
| `crates/core` | Unified payment domain, provider traits, request/response models, status model, error model. | SDK or Gateway payment commands. | Provider-neutral payment results and errors. | `common`; no concrete provider dependency. |
| `crates/signing` | Signing, verification, key loading, certificate-related abstractions. | Canonical message bytes, keys, certificates, headers. | Signatures, verification results, signing errors. | Crypto crates; no gateway dependency. |
| `crates/http-client` | Shared async HTTP behavior for provider adapters. | HTTP request intent, timeout, retry policy. | Provider HTTP response or transport error. | `reqwest`, `tokio`, `tracing`. |
| `crates/wechat` | WeChat Pay v3 adapter. | Unified payment/refund/query/webhook requests plus WeChat config. | Unified payment results and verified webhook events. | `core`, `signing`, `http-client`. |
| `crates/alipay` | Alipay adapter. | Unified payment/refund/query/webhook requests plus Alipay config. | Unified payment results and verified webhook events. | `core`, `signing`, `http-client`. |
| `crates/stripe` | Future Stripe adapter boundary. | Unified payment/refund/query/webhook requests plus Stripe config. | Unified payment results and verified webhook events. | `core`, `http-client`; signing rules depend on Stripe webhook signing. |
| `gateway/api` | HTTP route definitions, request validation, response mapping. | HTTP requests. | HTTP responses with unified error body. | `gateway/app`, `gateway/auth`, `core`. |
| `gateway/auth` | API key and JWT authentication for gateway callers. | HTTP headers, configured credentials. | Authenticated caller context or auth error. | Gateway configuration and JWT dependency. |
| `gateway/app` | Gateway composition root and runtime wiring. | Configuration, provider selection, route modules. | Running HTTP service. | Gateway modules and SDK crates. |
| `examples` | Minimal integration examples after implementation starts. | Example configuration. | Runnable examples. | Implemented crates only. |
| `docs` | Architecture, specification, build, handoff, and external docs index. | Project decisions and current status. | Durable context for future AI sessions. | None. |

## Data Flow

### SDK Payment Creation

1. Business system creates a unified payment request.
2. SDK entry point validates the provider-neutral request.
3. Payment Core selects the configured provider adapter.
4. Provider adapter maps the request into provider-specific fields.
5. Signing module signs the provider request when required.
6. HTTP client sends the provider request.
7. Provider adapter maps the response back into a unified response.
8. SDK returns the unified response to the business system.

### Gateway Payment Creation

1. Business system sends an HTTP request to the gateway.
2. Gateway auth validates API key or JWT.
3. Gateway API validates and normalizes the request body.
4. Gateway app calls the same Payment Core used by the SDK.
5. Provider adapter executes provider-specific behavior.
6. Gateway maps the unified result or error into a stable HTTP response.

### Webhook Processing

1. Provider sends callback data to the gateway or SDK webhook handler.
2. Webhook module captures raw body and provider headers.
3. Signing module verifies provider signature or certificate chain.
4. Provider adapter parses the verified payload.
5. Payment Core converts it to a unified payment event.
6. Business system receives a provider-neutral event.

## Key Design Decisions

### One Payment Core

SDK and Gateway must share the same core logic. The Gateway is an HTTP wrapper,
not a second payment implementation.

### Provider Adapters Are Translation Boundaries

Provider-specific naming, payload fields, signing details, and endpoint behavior
must stay inside provider adapters. Core models must not leak WeChat or Alipay
field names unless the field is truly provider-neutral.

### Signing Is a Separate Module

Signing and verification must not be embedded directly inside provider request
assembly. This keeps crypto behavior auditable and reusable across payment,
refund, query, and webhook flows.

### Money Uses Explicit Minor Units

Amounts should be represented as integer minor units plus currency code. Floating
point values are not acceptable for payment amounts.

### Idempotency Is a First-Class Concern

Payment creation and refund requests must carry a business order identifier or
idempotency key. Provider adapters must preserve this identifier when mapping
requests.

### Errors Are Unified but Traceable

Public errors should be provider-neutral. Internal errors must retain enough
context to diagnose provider, endpoint, request id, and failure category.

### Configuration Is Centralized

Provider configuration must be loaded through a single configuration boundary.
Provider adapters receive explicit config objects and must not read environment
variables directly.

### Webhook Verification Requires Raw Input

Webhook verification must use the raw request body and original provider headers.
Parsed JSON alone is not enough for reliable signature verification.

### Gateway Auth Is Separate From Payment Auth

Gateway caller authentication protects the API surface. Provider signing protects
requests sent to payment platforms. These must stay independent.

### Extension Providers Are Additive

Stripe, PayPal, Apple Pay, and Google Pay should be added by implementing new
provider adapters behind existing core boundaries. Adding a provider must not
force changes to Gateway route contracts unless the business capability changes.

## MVP Scope

The first implementation phase should include:

- Unified payment core
- WeChat Pay v3 Native payment
- Alipay web payment
- Payment query
- Refund
- Webhook verification and parsing
- Gateway routes for create, query, and refund
- API key authentication
- Structured logging and unified error responses

The first implementation phase should not include:

- SaaS tenant management
- Risk control
- Settlement and reconciliation
- Admin dashboard
- Advanced payment routing
- Multi-currency provider routing

## Step 1 Output Status

Architecture design is complete at the documentation level. No implementation
files have been created.

