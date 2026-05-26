# Agent Guide

This file defines how AI agents must work on this repository.

Project: Rust unified payment SDK plus Payment API Gateway.

Primary goal: build a payment infrastructure that AI can reliably understand,
modify, and extend across sessions.

## Mandatory Workflow

Agents must follow this order:

1. Step 1: Architecture Design
2. Step 2: Documentation
3. Step 3: Context Handoff
4. Step 4: Implementation, only after explicit user approval

Before starting any step, state:

- Current step
- What will be produced
- Whether code implementation is allowed in this step

## Step Rules

### Step 1: Architecture Design

Required output:

- Overall system architecture
- Module breakdown with clear responsibilities
- Data flow
- Key design decisions

Do not write implementation code in this step.

Primary file for this repository:

- `docs/ARCHITECTURE.md`

### Step 2: Documentation

Required output:

- `docs/SPEC.md`: system specification
- `docs/BUILD.md`: build and usage instructions
- `docs/INTEGRATION_DOCS.md`: official integration documentation index

Production hardening documentation, when the project is intended for production
or cross-backend integration:

- `docs/README.md`: documentation index
- `docs/API_CONTRACT.md`: stable Gateway API contract
- `docs/openapi.yaml`: machine-readable Gateway API draft
- `docs/CLIENT_INTEGRATION.md`: backend integration guide
- `docs/ERROR_CODES.md`: stable public error codes
- `docs/DATA_MODEL.md`: durable ledger and idempotency model
- `docs/WEBHOOK_RELIABILITY.md`: webhook reliability rules
- `docs/SECURITY.md`: secret, auth, logging, and compliance rules
- `docs/OPERATIONS.md`: deployment, metrics, alerts, and runbooks
- `docs/PROVIDER_MAPPING.md`: provider status, error, amount, and retry mapping
- `docs/PROVIDER_ADAPTER_GUIDE.md`: provider extension guide

Do not write implementation code in this step.

### Step 3: Context Handoff

Required output:

- `docs/nextsession.md`

It must include:

- Current progress
- Architecture summary
- Completed parts
- Pending tasks, step by step
- Next actions
- Risks and unknowns

### Step 4: Implementation

Implementation is allowed only when the user explicitly asks for it.

Allowed work:

- Create Rust workspace and crates
- Implement SDK modules
- Implement API Gateway modules
- Add tests and examples

Not allowed before Step 4:

- Rust source files
- Cargo workspace files
- Gateway implementation files
- Example code that compiles as part of the project

## Git Workflow

After each major step, commit the completed work.

Default command pattern:

```text
git add .
git commit -m "feat: <describe current step>"
```

If the working tree contains unrelated user changes, do not include those
changes in the commit. Stage only the files produced for the current step and
record the reason in the final response.

Do not push.

## AI-Oriented Architecture Principles

Optimize for AI comprehension before elegance.

Each module must:

- Have one responsibility
- Have explicit inputs and outputs
- Avoid hidden dependencies
- Be understandable without reading the entire project
- Use descriptive names instead of abbreviations
- Prefer composition over inheritance
- Keep control flow shallow and predictable

Avoid:

- Large multi-purpose modules
- God utility files
- Scattered business logic
- Hidden global state
- Implicit configuration
- Deep abstraction stacks

## Project Architecture Constraints

The project must separate these concerns:

- Payment domain model
- Provider abstraction
- Provider adapters
- Durable ledger persistence
- Idempotency persistence
- Signing and verification
- HTTP client behavior
- Webhook processing
- Webhook event persistence and asynchronous processing
- Gateway routing
- Gateway authentication
- Configuration loading

Business systems must depend on the unified SDK or HTTP API, not directly on
WeChat Pay, Alipay, Stripe, PayPal, Apple Pay, or Google Pay.

## Documentation Rules

All long-lived project documentation must live in `docs/`, except this file.

Integration documentation for external payment platforms must be recorded in:

- `docs/INTEGRATION_DOCS.md`

That file must contain:

- Official documentation URL
- Provider name
- Scope of the document
- Access date
- Notes about required verification before implementation

Before changing a payment provider adapter, refresh the relevant official docs
link and confirm the documented API is still current.

Before changing Gateway API behavior, update both:

- `docs/API_CONTRACT.md`
- `docs/openapi.yaml`

Before changing public error behavior, update:

- `docs/ERROR_CODES.md`

Before changing provider status, amount, or retry behavior, update:

- `docs/PROVIDER_MAPPING.md`

## Self-Correction Rule

If an agent detects premature coding, poor modularization, or rising accidental
complexity, it must stop and correct the structure before continuing.

Refactoring is preferred over adding more logic to a confusing module.
