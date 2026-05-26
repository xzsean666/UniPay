# Payment Provider Documentation Index

Current step: Step 2 - Documentation.

Purpose: keep official payment-platform documentation links in one place so
future AI sessions can quickly verify provider behavior before making code
changes.

Access date: 2026-05-26, Asia/Shanghai.

Rules:

- Prefer official documentation over blogs, examples, or copied snippets.
- Re-open the relevant official docs before changing any provider adapter.
- If a provider page is JavaScript-rendered and cannot be fully read by CLI
  tools, verify it in a browser before implementation.
- Record newly discovered official URLs here instead of scattering links through
  source files.

## MVP Providers

### WeChat Pay v3

Official documentation base:

- https://pay.weixin.qq.com/doc/v3/merchant
- https://pay.wechatpay.cn/doc/v3/merchant

Required MVP documents:

| Scope | Official URL | Observed latest update | Implementation note |
| --- | --- | --- | --- |
| Native payment product overview | https://pay.wechatpay.cn/doc/v3/merchant/4012791874 | Noted as current access on 2026-05-26 | Use for product behavior and Native payment flow. |
| Native create payment | https://pay.wechatpay.cn/doc/v3/merchant/4012791877 | 2025.03.31 | Endpoint observed: `POST /v3/pay/transactions/native`. Returns `code_url`. |
| Query by merchant order id | https://pay.wechatpay.cn/doc/v3/merchant/4012791880 | 2024.12.27 | Endpoint observed: `GET /v3/pay/transactions/out-trade-no/{out_trade_no}`. |
| Refund application | https://pay.wechatpay.cn/doc/v3/merchant/4012791883 | 2025.01.09 | Endpoint observed: `POST /v3/refund/domestic/refunds`. Reuse merchant refund id for retry. |
| Payment success callback | https://pay.wechatpay.cn/doc/v3/merchant/4012791882 | 2024.12.27 | Callback body contains encrypted resource; decrypt with API v3 key after signature verification. |
| API v3 signing and verification overview | https://pay.wechatpay.cn/doc/v3/merchant/4012365342 | 2024.11.21 | Covers request signing, response verification, and callback verification. |

Implementation checkpoints:

- Confirm whether the merchant uses WeChat Pay public key mode or platform
  certificate mode before designing configuration.
- Use raw body and original headers for callback verification.
- Preserve `out_trade_no` and `out_refund_no` as idempotency boundaries.
- Do not log API v3 key, private key, authorization header, or ciphertext.

### Alipay

Official documentation base:

- https://open.alipay.com
- https://opendocs.alipay.com
- https://opendocs.alipay.com/apis

Required MVP documents:

| Scope | Official URL | Observed latest update | Implementation note |
| --- | --- | --- | --- |
| Computer website payment product | https://opendocs.alipay.com/open/270/105898 | Page is JavaScript-rendered; update date not visible to CLI tools. | Verify in browser before implementation. |
| Computer website payment quick start | https://opendocs.alipay.com/open/270/105899 | Page is JavaScript-rendered; update date not visible to CLI tools. | Use for gateway, keys, sandbox, and basic flow. |
| Async notification guide | https://opendocs.alipay.com/open/270/105902 | Page is JavaScript-rendered; update date not visible to CLI tools. | Verify signature rules and required response body before implementation. |
| `alipay.trade.page.pay` API | https://opendocs.alipay.com/apis/api_1/alipay.trade.page.pay | Page is JavaScript-rendered; update date not visible to CLI tools. | Create web payment; expected result is provider payment action such as HTML form or redirect flow. |
| `alipay.trade.query` API | https://opendocs.alipay.com/apis/api_1/alipay.trade.query | Page is JavaScript-rendered; update date not visible to CLI tools. | Query by merchant order id or trade id. |
| `alipay.trade.refund` API | https://opendocs.alipay.com/apis/api_1/alipay.trade.refund | Page is JavaScript-rendered; update date not visible to CLI tools. | Refund by merchant order id or trade id; preserve refund request id. |

Implementation checkpoints:

- Confirm current RSA2 signing and verification rules from official docs before
  writing the signer.
- Confirm whether certificate mode or public key mode is required for the target
  merchant account.
- Verify async notification response requirements.
- Treat Alipay amount formatting carefully; map to UniPay integer minor units at
  the core boundary and provider-specific decimal strings inside the adapter.
- Do not log private key, Alipay public key material, signatures, or full
  notification payloads that contain sensitive user data.

## Planned Future Providers

### Stripe

Official documentation base:

- https://docs.stripe.com

Recommended documents:

| Scope | Official URL | Observed note |
| --- | --- | --- |
| Payment Intents | https://docs.stripe.com/payments/payment-intents | Stripe documentation currently recommends Checkout Sessions with Payment Element for most integrations, unless Payment Intents are explicitly needed. |
| Checkout Sessions quickstart | https://docs.stripe.com/payments/quickstart-checkout-sessions | Evaluate before deciding the Stripe adapter shape. |
| Webhooks | https://docs.stripe.com/webhooks | Requires raw body and `Stripe-Signature` header verification. |
| Refund API | https://docs.stripe.com/api/refunds | Use when refund support is added. |
| Idempotent requests | https://docs.stripe.com/api/idempotent_requests | Required for safe retry behavior. |

Implementation checkpoints:

- Decide whether UniPay Stripe support should model Checkout Sessions,
  Payment Intents, or both.
- Keep Stripe webhook verification in the signing/webhook boundary.
- Do not expose Stripe client secret outside the intended client-side payment
  flow.

### PayPal

Official documentation base:

- https://developer.paypal.com

Recommended documents:

| Scope | Official URL | Observed note |
| --- | --- | --- |
| Orders API v2 | https://developer.paypal.com/docs/api/orders/v2/ | Official page may require JavaScript/security check in automated tools. Verify in browser before implementation. |
| Payments API v2 | https://developer.paypal.com/docs/api/payments/v2/ | Covers authorization, capture, refund, and payment detail operations. |
| Webhooks guide | https://developer.paypal.com/api/rest/webhooks/ | Webhooks require successful 2xx receipt and verification. |
| Webhooks API reference | https://developer.paypal.com/docs/api/webhooks/v1/ | Use for verify-webhook-signature behavior. |
| REST API authentication | https://developer.paypal.com/api/rest/authentication/ | Use for OAuth2 token flow. |
| Idempotency | https://developer.paypal.com/api/rest/reference/idempotency/ | Use `PayPal-Request-Id` behavior when adding PayPal. |

Implementation checkpoints:

- PayPal order creation and capture may need a different payment action model
  than WeChat Native and Alipay web payment.
- Verify webhook signatures or call PayPal verification endpoint before trusting
  callbacks.

### Apple Pay

Official documentation base:

- https://developer.apple.com/apple-pay/
- https://developer.apple.com/documentation/applepayontheweb

Recommended documents:

| Scope | Official URL | Observed note |
| --- | --- | --- |
| Apple Pay on the Web | https://developer.apple.com/documentation/applepayontheweb | JavaScript-rendered official docs; verify in browser. |
| Apple Pay JS API | https://developer.apple.com/documentation/applepayontheweb/apple-pay-js-api | JavaScript-rendered official docs; verify in browser. |
| Sandbox testing | https://developer.apple.com/apple-pay/sandbox-testing/ | Contains current sandbox testing requirements and supported regions. |
| Apple Pay merchant support | https://developer.apple.com/contact/apple-pay/ | Use for merchant onboarding issues. |

Implementation checkpoints:

- Apple Pay often requires merchant validation and a PSP/acquirer path; define
  whether UniPay integrates directly or through another processor.
- Web domain verification and TLS requirements must be handled outside provider
  request mapping.

### Google Pay

Official documentation base:

- https://developers.google.com/pay

Recommended documents:

| Scope | Official URL | Observed latest update |
| --- | --- | --- |
| Web overview | https://developers.google.com/pay/api/web/overview | 2026-03-19 UTC |
| Request objects | https://developers.google.com/pay/api/web/reference/request-objects | Verify before implementation; page documents `PaymentDataRequest` and tokenization request objects. |
| Response objects | https://developers.google.com/pay/api/web/reference/response-objects | Use for parsing payment data responses. |
| Payment data cryptography | https://developers.google.com/pay/api/web/guides/resources/payment-data-cryptography | Use if direct token decryption is required. |
| Test cards and tokens | https://developers.google.com/pay/api/web/guides/resources/test-card-suite | Use for sandbox testing. |

Implementation checkpoints:

- Decide whether UniPay supports Google Pay through a gateway tokenization flow
  or direct integration.
- Direct integration may introduce PCI DSS requirements; verify current Google
  Pay rules before implementation.

## Provider Documentation Refresh Checklist

Before implementing or modifying a provider adapter:

1. Open the official docs from this file.
2. Confirm endpoint path and HTTP method.
3. Confirm required request fields and response fields.
4. Confirm signing and verification rules.
5. Confirm callback retry and success-response rules.
6. Confirm sandbox availability.
7. Update this file with the new access date, observed update date, and any
   changed URLs.

