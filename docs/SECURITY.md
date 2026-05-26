# Security

Current step: Step 2 - Documentation Hardening.

This document defines security requirements for UniPay.

## Security Goals

UniPay must protect:

- Merchant private keys.
- Provider API secrets.
- Gateway API keys and JWT secrets.
- Webhook verification secrets and certificates.
- Payment and refund records.
- Provider request and webhook logs.
- Personally identifiable information.

## Secret Management

Production secrets must be loaded from an approved secret manager or an
explicitly approved secure file path.

Approved production patterns:

- Cloud KMS plus secret manager.
- Vault-style secret manager.
- Kubernetes secrets encrypted at rest with restricted access.
- Secure mounted files with restricted permissions for private keys.

Not approved for production:

- Plaintext secrets committed to git.
- Plaintext secrets in example files.
- Provider credentials in business request payloads.
- Secrets printed in logs.

## WeChat Pay Keys And Certificates

Configuration must support:

- Merchant id.
- App id.
- Merchant private key path or secret reference.
- Merchant certificate serial number.
- WeChat Pay public key or platform certificate mode.
- API v3 key for callback resource decryption.

Security rules:

- Private key stays inside signing boundary.
- API v3 key is never logged.
- Certificate serial number must be explicit.
- Certificate expiration must be observable.
- Key and certificate rotation must be documented before production.

## Alipay Keys And Certificates

Configuration must support:

- App id.
- Merchant private key path or secret reference.
- Alipay public key or certificate reference.
- Sign type, expected to be RSA2 for MVP.
- Gateway URL by environment.

Security rules:

- Merchant private key stays inside signing boundary.
- Alipay public key or certificate mode must be confirmed before implementation.
- Signing canonicalization must be fixture-tested.
- Async notification verification must use official Alipay rules.

## Gateway API Keys

API keys must be:

- High entropy.
- Stored hashed at rest.
- Scoped to caller or merchant.
- Rotatable without downtime.
- Redacted in logs.

API key checks must happen before provider operations.

Recommended API key lifecycle:

1. Create key with identifier and secret value.
2. Store hash and metadata.
3. Show secret value once to operator.
4. Allow overlapping old and new keys during rotation.
5. Revoke old key after caller migration.

## JWT

JWT is planned, not MVP.

If enabled later, define:

- Issuer.
- Audience.
- Algorithm allowlist.
- Key id handling.
- Expiration limit.
- Clock skew tolerance.
- Revocation or short-lived token strategy.

Do not accept `alg=none`.

## Logging And Redaction

Do not log:

- Private keys.
- API keys.
- JWT secrets.
- Provider authorization headers.
- Full signatures.
- API v3 keys.
- Full webhook ciphertext.
- Full provider request or response bodies containing sensitive data.
- Payer identifiers unless explicitly classified and redacted.

Allowed logs:

- Trace id.
- Provider name.
- Operation.
- Merchant order id after classification.
- Merchant refund id after classification.
- Provider request id.
- Error category.
- Retryability.
- Timing metrics.

## Data Protection

Payment and refund records should store only data required for operations,
support, audit, and legal retention.

Rules:

- Encrypt storage at rest.
- Use TLS for all network communication.
- Restrict database access by role.
- Redact sensitive fields in provider request records.
- Define data retention before production.
- Define deletion or anonymization process where legally required.

## Webhook Security

Webhook routes must:

- Verify provider signature.
- Reject replay attempts.
- Deduplicate valid events.
- Use request size limits.
- Use body read timeout.
- Avoid logging raw body by default.
- Store raw body encrypted only if approved.

Webhook routes should not rely on source IP allowlists as the only security
control. Signature verification is mandatory.

## Supply Chain Security

Planned implementation should include:

- `cargo audit` for known vulnerabilities.
- `cargo deny` for license and duplicate dependency policy.
- Lockfile review for production binaries.
- Dependency update automation.
- Container image scanning if packaged as container.
- SBOM generation if required by deployment policy.

## Compliance Notes

MVP WeChat Pay and Alipay server-side integration can usually avoid direct card
data handling. Future Stripe, Apple Pay, and Google Pay integrations may change
the compliance boundary.

Before adding card-network based providers:

- Define whether UniPay touches card data or only provider tokens.
- Confirm PCI DSS scope.
- Confirm processor terms.
- Update this document with provider-specific compliance requirements.

## Security Review Gate

Before production:

- Threat model Gateway, SDK, provider adapters, signing, storage, and webhooks.
- Verify secret storage and rotation.
- Verify logs are redacted.
- Verify webhook replay protection.
- Verify API key rotation.
- Verify least-privilege database access.
- Verify dependency vulnerability scan.

