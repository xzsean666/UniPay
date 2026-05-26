use std::sync::Arc;

use axum::body::{Body, to_bytes};
use axum::http::{Method, Request, StatusCode, header};
use serde_json::Value;
use tower::ServiceExt;
use unipay_gateway::{ApiKeyConfig, GatewayConfig, InMemoryGatewayService, router};

fn app() -> axum::Router {
    router(
        GatewayConfig::for_tests(),
        Arc::new(InMemoryGatewayService::new()),
    )
}

fn app_with_two_callers() -> axum::Router {
    router(
        GatewayConfig::new(
            "127.0.0.1:0"
                .parse()
                .expect("test socket address must parse"),
            vec![
                ApiKeyConfig::new("caller-one", "caller-one-key"),
                ApiKeyConfig::new("caller-two", "caller-two-key"),
            ],
        )
        .expect("test config should be valid"),
        Arc::new(InMemoryGatewayService::new()),
    )
}

async fn json_response(response: axum::response::Response) -> (StatusCode, Value) {
    let status = response.status();
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .expect("response body should be readable");
    let json = serde_json::from_slice(&body).expect("response should be JSON");
    (status, json)
}

fn json_request(method: Method, uri: &str, body: Value, authenticated: bool) -> Request<Body> {
    json_request_with_headers(
        method,
        uri,
        body,
        authenticated.then_some("test-api-key"),
        authenticated.then_some("test-idempotency-key"),
    )
}

fn json_request_with_headers(
    method: Method,
    uri: &str,
    body: Value,
    api_key: Option<&str>,
    idempotency_key: Option<&str>,
) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .header("x-request-id", "test-trace");
    if let Some(api_key) = api_key {
        builder = builder.header(header::AUTHORIZATION, format!("Bearer {api_key}"));
    }
    if let Some(idempotency_key) = idempotency_key {
        builder = builder.header("Idempotency-Key", idempotency_key);
    }

    builder
        .body(Body::from(body.to_string()))
        .expect("request should build")
}

#[tokio::test]
async fn health_routes_are_public() {
    let response = app()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/v1/health/live")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("route should respond");

    let (status, body) = json_response(response).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["status"], "alive");
}

#[tokio::test]
async fn business_routes_require_api_key() {
    let response = app()
        .oneshot(json_request(
            Method::POST,
            "/v1/payments",
            serde_json::json!({}),
            false,
        ))
        .await
        .expect("route should respond");

    let (status, body) = json_response(response).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "UNAUTHORIZED");
}

#[tokio::test]
async fn creates_and_queries_wechat_payment() {
    let app = app();
    let create_response = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/v1/payments",
            serde_json::json!({
                "provider": "wechat",
                "merchant_order_id": "order_gateway_test_1",
                "amount": {
                    "currency": "CNY",
                    "amount_minor": 100
                },
                "subject": "Gateway test",
                "channel": "native"
            }),
            true,
        ))
        .await
        .expect("route should respond");

    let (status, body) = json_response(create_response).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["status"], "pending");
    assert_eq!(body["data"]["payment_action"]["type"], "qr_code_url");

    let query_response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/v1/payments/order_gateway_test_1?provider=wechat")
                .header(header::AUTHORIZATION, "Bearer test-api-key")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("route should respond");

    let (status, body) = json_response(query_response).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["merchant_order_id"], "order_gateway_test_1");
}

#[tokio::test]
async fn rejects_invalid_amount() {
    let response = app()
        .oneshot(json_request(
            Method::POST,
            "/v1/payments",
            serde_json::json!({
                "provider": "wechat",
                "merchant_order_id": "order_invalid_amount",
                "amount": {
                    "currency": "CNY",
                    "amount_minor": 0
                },
                "subject": "Gateway test",
                "channel": "native"
            }),
            true,
        ))
        .await
        .expect("route should respond");

    let (status, body) = json_response(response).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "INVALID_AMOUNT");
}

#[tokio::test]
async fn malformed_json_uses_unipay_error_envelope() {
    let response = app()
        .oneshot(json_request(
            Method::POST,
            "/v1/payments",
            serde_json::json!({
                "provider": "wechat",
                "merchant_order_id": "order_float_amount",
                "amount": {
                    "currency": "CNY",
                    "amount_minor": 1.25
                },
                "subject": "Gateway test",
                "channel": "native"
            }),
            true,
        ))
        .await
        .expect("route should respond");

    let (status, body) = json_response(response).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["success"], false);
    assert_eq!(body["error"]["code"], "INVALID_REQUEST");
    assert_eq!(body["trace_id"], "test-trace");
}

#[tokio::test]
async fn post_routes_require_idempotency_key() {
    let response = app()
        .oneshot(json_request_with_headers(
            Method::POST,
            "/v1/payments",
            serde_json::json!({
                "provider": "wechat",
                "merchant_order_id": "order_missing_idempotency",
                "amount": {
                    "currency": "CNY",
                    "amount_minor": 100
                },
                "subject": "Gateway test",
                "channel": "native"
            }),
            Some("test-api-key"),
            None,
        ))
        .await
        .expect("route should respond");

    let (status, body) = json_response(response).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "MISSING_IDEMPOTENCY_KEY");
}

#[tokio::test]
async fn idempotency_key_reuse_with_different_body_conflicts() {
    let app = app();
    let first = app
        .clone()
        .oneshot(json_request_with_headers(
            Method::POST,
            "/v1/payments",
            serde_json::json!({
                "provider": "wechat",
                "merchant_order_id": "order_idempotency_conflict_1",
                "amount": {
                    "currency": "CNY",
                    "amount_minor": 100
                },
                "subject": "Gateway test",
                "channel": "native"
            }),
            Some("test-api-key"),
            Some("same-idempotency-key"),
        ))
        .await
        .expect("route should respond");
    assert_eq!(json_response(first).await.0, StatusCode::OK);

    let second = app
        .oneshot(json_request_with_headers(
            Method::POST,
            "/v1/payments",
            serde_json::json!({
                "provider": "wechat",
                "merchant_order_id": "order_idempotency_conflict_2",
                "amount": {
                    "currency": "CNY",
                    "amount_minor": 100
                },
                "subject": "Gateway test",
                "channel": "native"
            }),
            Some("test-api-key"),
            Some("same-idempotency-key"),
        ))
        .await
        .expect("route should respond");

    let (status, body) = json_response(second).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "IDEMPOTENCY_CONFLICT");
}

#[tokio::test]
async fn callers_cannot_query_each_others_payments() {
    let app = app_with_two_callers();
    let create_response = app
        .clone()
        .oneshot(json_request_with_headers(
            Method::POST,
            "/v1/payments",
            serde_json::json!({
                "provider": "wechat",
                "merchant_order_id": "order_caller_isolation",
                "amount": {
                    "currency": "CNY",
                    "amount_minor": 100
                },
                "subject": "Gateway test",
                "channel": "native"
            }),
            Some("caller-one-key"),
            Some("caller-one-order-key"),
        ))
        .await
        .expect("route should respond");
    assert_eq!(json_response(create_response).await.0, StatusCode::OK);

    let query_response = app
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/v1/payments/order_caller_isolation?provider=wechat")
                .header(header::AUTHORIZATION, "Bearer caller-two-key")
                .body(Body::empty())
                .expect("request should build"),
        )
        .await
        .expect("route should respond");

    let (status, body) = json_response(query_response).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "PAYMENT_NOT_FOUND");
}

#[tokio::test]
async fn rejects_refund_for_pending_payment() {
    let app = app();
    let _ = app
        .clone()
        .oneshot(json_request(
            Method::POST,
            "/v1/payments",
            serde_json::json!({
                "provider": "alipay",
                "merchant_order_id": "order_gateway_refund",
                "amount": {
                    "currency": "CNY",
                    "amount_minor": 200
                },
                "subject": "Gateway refund test",
                "channel": "web"
            }),
            true,
        ))
        .await
        .expect("route should respond");

    let response = app
        .oneshot(json_request(
            Method::POST,
            "/v1/refunds",
            serde_json::json!({
                "provider": "alipay",
                "merchant_order_id": "order_gateway_refund",
                "merchant_refund_id": "refund_gateway_refund_1",
                "amount": {
                    "currency": "CNY",
                    "amount_minor": 100
                }
            }),
            true,
        ))
        .await
        .expect("route should respond");

    let (status, body) = json_response(response).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["error"]["code"], "PAYMENT_STATE_CONFLICT");
}

#[tokio::test]
async fn webhook_route_is_public_but_fails_closed_without_signature_verifier() {
    let app = app();
    let response = app
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/v1/webhooks/wechat/payments")
                .header(header::CONTENT_TYPE, "application/json")
                .header("wechatpay-nonce", "same-nonce")
                .body(Body::from(r#"{"id":"evt_1"}"#))
                .expect("request should build"),
        )
        .await
        .expect("route should respond");

    let (status, body) = json_response(response).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "SIGNATURE_VERIFY_FAILED");
}
