# Next Session Handoff

Current step: Step 3 - Context Handoff.

Date: 2026-05-26, Asia/Shanghai.

Implementation status: not started. No Rust code, Cargo workspace, or gateway
code has been created.

## Current Progress

Completed the required pre-implementation documentation workflow:

1. Step 1 - Architecture Design.
2. Step 2 - Documentation.
3. Step 3 - Context Handoff.

Commits created:

- `46c9e00` - `feat: add architecture design docs`
- `99e8e30` - `feat: add project specification docs`

The Step 3 handoff commit should exist after this file is committed.

## Architecture Summary

UniPay is a Rust unified payment SDK plus Payment API Gateway.

Architecture:

- Business systems integrate either through Rust SDK or HTTP Gateway.
- SDK and Gateway both call the same Payment Core.
- Payment Core owns provider-neutral payment models, errors, provider
  abstraction, webhook model, and idempotency rules.
- Provider adapters translate unified operations into provider-specific API
  calls.
- Signing and verification are separated into a dedicated signing boundary.
- HTTP behavior is shared through a dedicated async HTTP client boundary.
- Gateway authentication is independent from payment-provider signing.

Planned modules:

- `crates/common`
- `crates/core`
- `crates/signing`
- `crates/http-client`
- `crates/wechat`
- `crates/alipay`
- `crates/stripe` as future boundary
- `gateway/api`
- `gateway/auth`
- `gateway/app`
- `examples`
- `docs`

MVP providers:

- WeChat Pay v3 Native payment.
- Alipay computer website payment.

MVP operations:

- Create payment.
- Query payment.
- Refund payment.
- Verify and parse webhook.
- Gateway API key authentication.
- Unified error response.
- Structured logging.

## Completed Parts

Root guidance:

- `Agent.md` defines the strict AI workflow, architecture rules, documentation
  rules, git workflow, and implementation gate.

Documentation:

- `docs/ARCHITECTURE.md` contains the overall architecture, module breakdown,
  data flow, and key decisions.
- `docs/SPEC.md` contains product scope, MVP scope, domain model, provider
  requirements, gateway routes, signing rules, error model, idempotency, and test
  strategy.
- `docs/BUILD.md` records the current non-buildable status and the planned build,
  test, configuration, and development sequence.
- `docs/INTEGRATION_DOCS.md` centralizes official provider documentation links
  for WeChat Pay, Alipay, Stripe, PayPal, Apple Pay, and Google Pay.

No implementation code has been written.

## Pending Tasks

Do not start these until the user explicitly approves Step 4 implementation.

1. Create Rust workspace structure.
2. Create empty crate boundaries for common, core, signing, http-client, wechat,
   and alipay.
3. Define core domain models and provider traits.
4. Implement common value objects, especially amount and currency.
5. Implement unified error types.
6. Implement signing interfaces and fixture-driven tests.
7. Implement shared HTTP client boundary.
8. Implement WeChat Pay v3 Native create payment mapping.
9. Implement WeChat Pay query and refund mapping.
10. Implement WeChat Pay webhook signature verification and resource decryption.
11. Implement Alipay web payment create mapping.
12. Implement Alipay query and refund mapping.
13. Implement Alipay async notification verification.
14. Implement Gateway API routes.
15. Implement Gateway API key authentication.
16. Add unit tests and mocked provider tests.
17. Add examples after SDK behavior is stable.
18. Update `docs/BUILD.md` with real commands and package names.
19. Update `docs/nextsession.md` at the end of the next session.

## Next Actions

If the user approves implementation, begin with the smallest buildable slice:

1. Create the Cargo workspace and crate directories.
2. Add minimal crate manifests with no provider logic.
3. Add core provider-neutral models and compile them.
4. Add tests for amount validation and status mapping.
5. Commit the workspace skeleton before starting provider-specific code.

Recommended first implementation commit:

```text
feat: add rust workspace skeleton
```

## Risks And Unknowns

Provider documentation risk:

- WeChat Pay official pages were readable and showed observed update dates for
  key pages.
- Alipay Open Platform pages are JavaScript-rendered in automated tools. Verify
  Alipay API, signing, async notification, and certificate/public key mode in a
  browser before implementation.
- PayPal and Apple developer docs may require browser verification before future
  adapter work.

Architecture risk:

- The unified payment action model must support QR URL, redirect URL, and HTML
  form style flows without leaking provider-specific fields into core models.
- Webhook verification must preserve raw request body and original headers.
- Amount handling must avoid floating point values.
- Configuration must not be scattered across provider adapters.

Security risk:

- Private keys, API keys, authorization headers, signatures, and encrypted
  webhook payloads must not be logged.
- Replay protection should be added for webhooks once provider timestamp and
  nonce semantics are confirmed.
- Idempotency rules must be enforced before provider retries are introduced.

Scope risk:

- Stripe, PayPal, Apple Pay, and Google Pay are documented as future extension
  providers. They should not be implemented in the MVP unless the user changes
  scope.

