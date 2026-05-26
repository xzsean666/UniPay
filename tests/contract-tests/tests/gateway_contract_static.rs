use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_openapi() -> YamlValue {
    let path = repo_root().join("docs/openapi.yaml");
    let raw = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    serde_yaml::from_str(&raw)
        .unwrap_or_else(|error| panic!("failed to parse {}: {error}", path.display()))
}

fn read_json_fixture(path: &str) -> JsonValue {
    let full_path = repo_root().join(path);
    let raw = fs::read_to_string(&full_path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", full_path.display()));
    serde_json::from_str(&raw)
        .unwrap_or_else(|error| panic!("failed to parse {}: {error}", full_path.display()))
}

fn field<'a>(value: &'a YamlValue, key: &str) -> &'a YamlValue {
    value
        .as_mapping()
        .and_then(|mapping| mapping.get(YamlValue::String(key.to_owned())))
        .unwrap_or_else(|| panic!("missing YAML field `{key}`"))
}

fn optional_field<'a>(value: &'a YamlValue, key: &str) -> Option<&'a YamlValue> {
    value
        .as_mapping()
        .and_then(|mapping| mapping.get(YamlValue::String(key.to_owned())))
}

fn path<'a>(value: &'a YamlValue, keys: &[&str]) -> &'a YamlValue {
    keys.iter().fold(value, |current, key| field(current, key))
}

fn operation<'a>(spec: &'a YamlValue, route: &str, method: &str) -> &'a YamlValue {
    path(spec, &["paths", route, method])
}

fn enum_values_at(spec: &YamlValue, keys: &[&str]) -> Vec<String> {
    path(spec, keys)
        .as_sequence()
        .unwrap_or_else(|| panic!("expected sequence at `{}`", keys.join(".")))
        .iter()
        .map(|value| {
            value
                .as_str()
                .unwrap_or_else(|| panic!("expected string enum value at `{}`", keys.join(".")))
                .to_owned()
        })
        .collect()
}

fn json_field<'a>(value: &'a JsonValue, key: &str) -> &'a JsonValue {
    value
        .get(key)
        .unwrap_or_else(|| panic!("missing JSON field `{key}`"))
}

fn assert_json_required_fields(value: &JsonValue, fields: &[&str]) {
    for field_name in fields {
        assert!(
            value.get(field_name).is_some(),
            "fixture is missing required field `{field_name}`"
        );
    }
}

#[test]
fn openapi_declares_complete_mvp_route_set() {
    let spec = read_openapi();
    let paths = field(&spec, "paths")
        .as_mapping()
        .expect("OpenAPI `paths` must be a map");

    for route in [
        "/v1/payments",
        "/v1/payments/{merchant_order_id}",
        "/v1/refunds",
        "/v1/refunds/{merchant_refund_id}",
        "/v1/webhooks/{provider}/payments",
        "/v1/webhooks/{provider}/refunds",
        "/v1/health/live",
        "/v1/health/ready",
    ] {
        assert!(
            paths.contains_key(YamlValue::String(route.to_owned())),
            "OpenAPI route `{route}` is missing"
        );
    }

    assert_eq!(
        field(operation(&spec, "/v1/payments", "post"), "operationId").as_str(),
        Some("createPayment")
    );
    assert_eq!(
        field(
            operation(&spec, "/v1/payments/{merchant_order_id}", "get"),
            "operationId"
        )
        .as_str(),
        Some("queryPayment")
    );
    assert_eq!(
        field(operation(&spec, "/v1/refunds", "post"), "operationId").as_str(),
        Some("createRefund")
    );
    assert_eq!(
        field(
            operation(&spec, "/v1/refunds/{merchant_refund_id}", "get"),
            "operationId"
        )
        .as_str(),
        Some("queryRefund")
    );
}

#[test]
fn business_routes_inherit_auth_and_public_routes_disable_it() {
    let spec = read_openapi();

    assert!(
        field(&spec, "security")
            .as_sequence()
            .is_some_and(|security| !security.is_empty()),
        "top-level bearer auth must be declared"
    );

    for (route, method) in [
        ("/v1/payments", "post"),
        ("/v1/payments/{merchant_order_id}", "get"),
        ("/v1/refunds", "post"),
        ("/v1/refunds/{merchant_refund_id}", "get"),
    ] {
        assert!(
            optional_field(operation(&spec, route, method), "security").is_none(),
            "business route {method} {route} should inherit top-level bearer auth"
        );
    }

    for (route, method) in [
        ("/v1/webhooks/{provider}/payments", "post"),
        ("/v1/webhooks/{provider}/refunds", "post"),
        ("/v1/health/live", "get"),
        ("/v1/health/ready", "get"),
    ] {
        assert!(
            field(operation(&spec, route, method), "security")
                .as_sequence()
                .is_some_and(|security| security.is_empty()),
            "public route {method} {route} must explicitly disable bearer auth"
        );
    }
}

#[test]
fn stable_public_enums_match_contract() {
    let spec = read_openapi();

    assert_eq!(
        enum_values_at(&spec, &["components", "schemas", "Provider", "enum"]),
        ["wechat", "alipay"]
    );
    assert_eq!(
        enum_values_at(&spec, &["components", "schemas", "PaymentChannel", "enum"]),
        ["native", "web"]
    );
    assert_eq!(
        enum_values_at(&spec, &["components", "schemas", "PaymentStatus", "enum"]),
        [
            "pending",
            "processing",
            "succeeded",
            "failed",
            "closed",
            "refunding",
            "partially_refunded",
            "refunded",
            "unknown"
        ]
    );
    assert_eq!(
        enum_values_at(&spec, &["components", "schemas", "RefundStatus", "enum"]),
        [
            "pending",
            "processing",
            "succeeded",
            "failed",
            "closed",
            "unknown"
        ]
    );
    assert_eq!(
        enum_values_at(
            &spec,
            &[
                "components",
                "schemas",
                "PaymentAction",
                "properties",
                "type",
                "enum"
            ]
        ),
        [
            "qr_code_url",
            "redirect_url",
            "html_form",
            "sdk_payload",
            "none"
        ]
    );
}

#[test]
fn request_schemas_preserve_money_and_required_fields() {
    let spec = read_openapi();

    let amount_minor = path(
        &spec,
        &[
            "components",
            "schemas",
            "Money",
            "properties",
            "amount_minor",
        ],
    );
    assert_eq!(field(amount_minor, "type").as_str(), Some("integer"));
    assert_eq!(field(amount_minor, "minimum").as_i64(), Some(1));

    assert_eq!(
        enum_values_at(
            &spec,
            &["components", "schemas", "CreatePaymentRequest", "required"]
        ),
        [
            "provider",
            "merchant_order_id",
            "amount",
            "subject",
            "channel"
        ]
    );
    assert_eq!(
        enum_values_at(
            &spec,
            &["components", "schemas", "CreateRefundRequest", "required"]
        ),
        [
            "provider",
            "merchant_order_id",
            "merchant_refund_id",
            "amount"
        ]
    );
}

#[test]
fn sample_http_file_covers_mvp_routes_and_retry_headers() {
    let path = repo_root().join("examples/http/unipay_mvp.http");
    let raw = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));

    for expected in [
        "GET {{baseUrl}}/v1/health/live",
        "GET {{baseUrl}}/v1/health/ready",
        "POST {{baseUrl}}/v1/payments",
        "GET {{baseUrl}}/v1/payments/{{orderId}}?provider=wechat",
        "POST {{baseUrl}}/v1/refunds",
        "GET {{baseUrl}}/v1/refunds/{{refundId}}?provider=wechat",
        "POST {{baseUrl}}/v1/webhooks/wechat/payments",
        "POST {{baseUrl}}/v1/webhooks/wechat/refunds",
        "POST {{baseUrl}}/v1/webhooks/alipay/payments",
        "POST {{baseUrl}}/v1/webhooks/alipay/refunds",
    ] {
        assert!(
            raw.contains(expected),
            "sample file is missing `{expected}`"
        );
    }

    assert!(
        raw.contains("Authorization: Bearer {{apiKey}}"),
        "business samples must include bearer API key header"
    );
    assert!(
        raw.matches("Idempotency-Key:").count() >= 4,
        "create and replay samples should include idempotency keys"
    );
    assert!(
        raw.contains("invalid-placeholder"),
        "webhook samples should clearly use invalid placeholder signatures"
    );
}

#[test]
fn business_json_fixtures_are_contract_shaped() {
    let payment = read_json_fixture("examples/http/json/create_wechat_native_payment.json");
    assert_json_required_fields(
        &payment,
        &[
            "provider",
            "merchant_order_id",
            "amount",
            "subject",
            "channel",
        ],
    );
    assert_eq!(json_field(&payment, "provider").as_str(), Some("wechat"));
    assert_eq!(json_field(&payment, "channel").as_str(), Some("native"));
    assert!(
        json_field(json_field(&payment, "amount"), "amount_minor").is_i64(),
        "payment amount must use integer minor units"
    );

    let alipay = read_json_fixture("examples/http/json/create_alipay_web_payment.json");
    assert_json_required_fields(
        &alipay,
        &[
            "provider",
            "merchant_order_id",
            "amount",
            "subject",
            "channel",
        ],
    );
    assert_eq!(json_field(&alipay, "provider").as_str(), Some("alipay"));
    assert_eq!(json_field(&alipay, "channel").as_str(), Some("web"));
    assert!(
        json_field(json_field(&alipay, "amount"), "amount_minor").is_i64(),
        "Alipay fixture must still use integer minor units"
    );

    let refund = read_json_fixture("examples/http/json/create_wechat_refund.json");
    assert_json_required_fields(
        &refund,
        &[
            "provider",
            "merchant_order_id",
            "merchant_refund_id",
            "amount",
        ],
    );
    assert_eq!(json_field(&refund, "provider").as_str(), Some("wechat"));
    assert!(
        json_field(json_field(&refund, "amount"), "amount_minor").is_i64(),
        "refund amount must use integer minor units"
    );

    let alipay_refund = read_json_fixture("examples/http/json/create_alipay_refund.json");
    assert_json_required_fields(
        &alipay_refund,
        &[
            "provider",
            "merchant_order_id",
            "merchant_refund_id",
            "amount",
        ],
    );
    assert_eq!(
        json_field(&alipay_refund, "provider").as_str(),
        Some("alipay")
    );
    assert!(
        json_field(json_field(&alipay_refund, "amount"), "amount_minor").is_i64(),
        "Alipay refund fixture must use integer minor units"
    );
}

#[test]
fn invalid_amount_fixtures_capture_expected_rejections() {
    let zero = read_json_fixture("examples/http/json/invalid_zero_amount_payment.json");
    assert_eq!(
        json_field(json_field(&zero, "amount"), "amount_minor").as_i64(),
        Some(0),
        "zero amount fixture should exercise INVALID_AMOUNT"
    );

    let floating = read_json_fixture("examples/http/json/invalid_float_amount_payment.json");
    assert!(
        json_field(json_field(&floating, "amount"), "amount_minor").is_f64(),
        "floating amount fixture should exercise non-integer amount rejection"
    );
}
