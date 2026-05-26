# Next Session Handoff

Current step: Step 4 - Implementation Handoff.

Date: 2026-05-26, Asia/Shanghai.

Implementation status: MVP workspace implemented and verified for local
development. The Gateway enforces caller-scoped resources, required
`Idempotency-Key` headers on POST routes, request-hash conflict detection, and
fail-closed provider webhook handling. The current project is buildable and
runnable with in-memory storage. Docker deployment examples and environment
variable documentation are present. Live provider API calls are not enabled yet.

## Current Progress

Completed the required pre-implementation documentation workflow:

1. Step 1 - Architecture Design.
2. Step 2 - Documentation.
3. Step 3 - Context Handoff.
4. Step 2 hardening update for production readiness and cross-backend
   integration.
5. Step 4 MVP implementation and production-blocker hardening pass.
6. Step 4 Docker deployment and environment variable examples.

Commits created:

- `46c9e00` - `feat: add architecture design docs`
- `99e8e30` - `feat: add project specification docs`
- `a38aff5` - `feat: add session handoff docs`
- `f7c9ef7` - `feat: add production readiness documentation`

The Step 4 implementation commit should exist after this handoff is updated.

## Architecture Summary

UniPay is a Rust unified payment SDK plus Payment API Gateway.

Architecture:

- Business systems integrate either through Rust SDK or HTTP Gateway.
- SDK and Gateway both call the same Payment Core.
- Payment Core owns provider-neutral payment models, errors, provider
  abstraction, webhook model, and idempotency rules.
- Storage boundary owns durable payment, refund, provider request, webhook, and
  idempotency records.
- Provider adapters translate unified operations into provider-specific API
  calls.
- Signing and verification are separated into a dedicated signing boundary.
- HTTP behavior is shared through a dedicated async HTTP client boundary.
- Webhook processing is durable, deduplicated, acknowledged by provider rules,
  and processed asynchronously by a worker.
- Gateway authentication is independent from payment-provider signing.

Planned modules:

- `crates/common`
- `crates/core`
- `crates/storage`
- `crates/signing`
- `crates/http-client`
- `crates/wechat`
- `crates/alipay`
- `crates/stripe` as future boundary
- `gateway/api`
- `gateway/auth`
- `gateway/app`
- `gateway/worker`
- `examples`
- `docs`

Implemented modules:

- `crates/common`
- `crates/core`
- `crates/storage`
- `crates/signing`
- `crates/http-client`
- `crates/wechat`
- `crates/alipay`
- `gateway`
- `examples/http`
- `tests/contract-tests`

MVP providers:

- WeChat Pay v3 Native payment.
- Alipay computer website payment.

MVP operations:

- Create payment.
- Query payment.
- Refund payment.
- Query refund.
- Verify and parse webhook.
- Keep payment, refund, webhook deduplication, and idempotency records in local
  memory for development.
- Fail closed on provider webhooks until real signature verification is wired.
- Gateway API key authentication.
- Unified error response.
- Structured logging.

Current MVP caveat:

- Gateway stores data in memory.
- WeChat and Alipay crates currently implement provider-neutral mapping,
  local payment-action generation, and webhook parsing entry points.
- Gateway webhook routes currently reject provider callbacks because production
  signature verification is not configured yet.
- Live provider HTTP calls and cryptographic verification must be completed
  before real-money production.

## Completed Parts

Root guidance:

- `Agent.md` defines the strict AI workflow, architecture rules, documentation
  rules, git workflow, and implementation gate.

Documentation:

- `docs/README.md` indexes all project documents.
- `docs/ARCHITECTURE.md` contains the overall architecture, module breakdown,
  data flow, and key decisions.
- `docs/SPEC.md` contains product scope, MVP scope, domain model, provider
  requirements, gateway routes, signing rules, error model, idempotency, and test
  strategy.
- `docs/BUILD.md` records build, test, configuration, and development commands.
- `docs/DEPLOYMENT.md` records Docker deployment, compose usage, and current
  environment variables.
- `docs/API_CONTRACT.md` defines the stable `/v1` Gateway API for backend
  callers.
- `docs/openapi.yaml` provides a machine-readable API draft for client
  generation and contract tests.
- `docs/CLIENT_INTEGRATION.md` defines caller flows, retry behavior, payment
  action handling, and production checklist.
- `docs/ERROR_CODES.md` defines stable public error codes.
- `docs/DATA_MODEL.md` defines ledger, webhook event, provider request, and
  idempotency records.
- `docs/WEBHOOK_RELIABILITY.md` defines webhook verification, deduplication,
  acknowledgement, asynchronous processing, and dead-letter handling.
- `docs/SECURITY.md` defines secret management, key handling, API key lifecycle,
  redaction, and compliance notes.
- `docs/OPERATIONS.md` defines health checks, metrics, alerts, deployment, and
  runbooks.
- `docs/PROVIDER_MAPPING.md` defines WeChat Pay and Alipay status, error,
  amount, webhook, and retry mappings.
- `docs/PROVIDER_ADAPTER_GUIDE.md` defines how to add or change provider
  adapters.
- `docs/INTEGRATION_DOCS.md` centralizes official provider documentation links
  for WeChat Pay, Alipay, Stripe, PayPal, Apple Pay, and Google Pay.

Implementation:

- Root `Cargo.toml` workspace created.
- `Cargo.lock` generated.
- Gateway binary `unipay-gateway` created.
- Dockerfile, docker-compose example, `.env.example`, and `.dockerignore`
  created.
- `/v1/payments`, `/v1/refunds`, webhook, and health routes implemented.
- API key middleware implemented.
- In-memory Gateway service implemented with caller isolation, required POST
  idempotency keys, request-hash conflict detection, refund state checks, and
  webhook fail-closed behavior.
- Core traits and models implemented.
- In-memory storage trait implementation added, including refund lookup for
  refundable-amount checks.
- Provider mapping tests added for WeChat and Alipay.
- HTTP request examples and contract tests added.

## Pending Tasks

1. Replace Gateway in-memory service with core `PaymentService` plus storage and
   provider registry wiring.
2. Implement durable database storage and migrations.
3. Implement live WeChat Pay v3 HTTP calls, request signing, response
   verification, callback decryption, and certificate/public key rotation.
4. Implement live Alipay HTTP calls, RSA2 signing, response verification, and
   async notification verification.
5. Add provider sandbox integration tests with credentials kept outside git.
6. Add webhook worker persistence and dead-letter processing.
7. Add metrics, tracing fields, readiness dependency checks, and deployment
   manifests.
8. Add generated clients or OpenAPI validation in CI if needed.
9. Update `docs/nextsession.md` after the next implementation session.

## Next Actions

To run locally:

```text
UNIPAY_GATEWAY_API_KEYS=test-caller:test-api-key cargo run -p unipay-gateway
```

Default bind address is `127.0.0.1:8080`. Override with
`UNIPAY_GATEWAY_BIND_ADDR`.

To run with Docker Compose:

```text
cp .env.example .env
docker compose up --build
```

Docker must bind the Gateway to `0.0.0.0:8080` inside the container.

Verification commands already run successfully:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
cargo test --manifest-path tests/contract-tests/Cargo.toml
cargo audit
docker compose --env-file .env.example config
```

`cargo-deny` is not installed in the current local environment.
`docker build -t unipay-gateway:local .` could not be completed in the current
local environment because the user cannot connect to `/var/run/docker.sock`.

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
- Storage transaction boundaries must prevent duplicate charges and duplicate
  refunds under retries.
- API contract and `openapi.yaml` must remain synchronized.
- Current Gateway uses a local in-memory service, so process restart loses
  state.
- Provider adapters are not yet connected to live provider APIs.

Security risk:

- Private keys, API keys, authorization headers, signatures, and encrypted
  webhook payloads must not be logged.
- Replay protection must be implemented for webhooks once provider timestamp and
  nonce semantics are confirmed.
- Idempotency rules must be enforced before provider retries are introduced.

Scope risk:

- Stripe, PayPal, Apple Pay, and Google Pay are documented as future extension
  providers. They should not be implemented in the MVP unless the user changes
  scope.
