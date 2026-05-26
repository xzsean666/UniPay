# UniPay HTTP Examples

These request samples are for manual verification with REST Client compatible
tools. They intentionally use placeholder API keys, placeholder provider
signature values, and deterministic merchant ids.

Run against a local Gateway after implementation by setting `baseUrl` and
`apiKey` in `unipay_mvp.http`.

Webhook samples with `invalid-placeholder` signatures are negative tests. They
must be rejected unless the implementation deliberately configures matching test
keys.

Refund create samples are contract-shape examples. The current local Gateway
correctly rejects refunds for payments that are still `pending`; a successful
provider payment state is required before refund creation can succeed.
