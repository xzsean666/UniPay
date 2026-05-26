use std::collections::BTreeMap;

use axum::body::Bytes;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::{Extension, Json};
use serde::Deserialize;

use crate::api::AppState;
use crate::api::models::{
    CreatePaymentRequest, CreateRefundRequest, HealthResponse, WebhookKind, parse_provider,
    parse_provider_query,
};
use crate::api::response::{error_response, success_response};
use crate::api::trace::trace_id_from_headers;
use crate::auth::AuthenticatedCaller;
use crate::error::ApiError;
use crate::services::RequestContext;

#[derive(Debug, Deserialize)]
pub struct ProviderQuery {
    pub provider: Option<String>,
}

pub async fn create_payment(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthenticatedCaller>,
    headers: HeaderMap,
    request: Result<Json<CreatePaymentRequest>, JsonRejection>,
) -> axum::response::Response {
    let trace_id = trace_id_from_headers(&headers);
    let context = request_context(caller, &trace_id, &headers);
    let Json(request) = match request {
        Ok(request) => request,
        Err(rejection) => {
            return error_response(json_rejection_error(rejection, "create_payment"), &trace_id);
        }
    };
    let request = match request.validate() {
        Ok(request) => request,
        Err(error) => return error_response(error, &trace_id),
    };

    match state.service.create_payment(context, request).await {
        Ok(payment) => success_response(StatusCode::OK, payment, &trace_id),
        Err(error) => error_response(error, &trace_id),
    }
}

pub async fn query_payment(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthenticatedCaller>,
    headers: HeaderMap,
    Path(merchant_order_id): Path<String>,
    Query(query): Query<ProviderQuery>,
) -> axum::response::Response {
    let trace_id = trace_id_from_headers(&headers);
    let context = request_context(caller, &trace_id, &headers);
    let provider = match parse_provider_query(query.provider, "query_payment") {
        Ok(provider) => provider,
        Err(error) => return error_response(error, &trace_id),
    };

    match state
        .service
        .query_payment(context, provider, merchant_order_id)
        .await
    {
        Ok(payment) => success_response(StatusCode::OK, payment, &trace_id),
        Err(error) => error_response(error, &trace_id),
    }
}

pub async fn create_refund(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthenticatedCaller>,
    headers: HeaderMap,
    request: Result<Json<CreateRefundRequest>, JsonRejection>,
) -> axum::response::Response {
    let trace_id = trace_id_from_headers(&headers);
    let context = request_context(caller, &trace_id, &headers);
    let Json(request) = match request {
        Ok(request) => request,
        Err(rejection) => {
            return error_response(json_rejection_error(rejection, "create_refund"), &trace_id);
        }
    };
    let request = match request.validate() {
        Ok(request) => request,
        Err(error) => return error_response(error, &trace_id),
    };

    match state.service.create_refund(context, request).await {
        Ok(refund) => success_response(StatusCode::OK, refund, &trace_id),
        Err(error) => error_response(error, &trace_id),
    }
}

pub async fn query_refund(
    State(state): State<AppState>,
    Extension(caller): Extension<AuthenticatedCaller>,
    headers: HeaderMap,
    Path(merchant_refund_id): Path<String>,
    Query(query): Query<ProviderQuery>,
) -> axum::response::Response {
    let trace_id = trace_id_from_headers(&headers);
    let context = request_context(caller, &trace_id, &headers);
    let provider = match parse_provider_query(query.provider, "query_refund") {
        Ok(provider) => provider,
        Err(error) => return error_response(error, &trace_id),
    };

    match state
        .service
        .query_refund(context, provider, merchant_refund_id)
        .await
    {
        Ok(refund) => success_response(StatusCode::OK, refund, &trace_id),
        Err(error) => error_response(error, &trace_id),
    }
}

pub async fn receive_payment_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(provider): Path<String>,
    body: Bytes,
) -> axum::response::Response {
    receive_webhook(state, headers, provider, WebhookKind::Payment, body).await
}

pub async fn receive_refund_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(provider): Path<String>,
    body: Bytes,
) -> axum::response::Response {
    receive_webhook(state, headers, provider, WebhookKind::Refund, body).await
}

pub async fn liveness(headers: HeaderMap) -> axum::response::Response {
    let trace_id = trace_id_from_headers(&headers);
    success_response(
        StatusCode::OK,
        HealthResponse { status: "alive" },
        &trace_id,
    )
}

pub async fn readiness(headers: HeaderMap) -> axum::response::Response {
    let trace_id = trace_id_from_headers(&headers);
    success_response(
        StatusCode::OK,
        HealthResponse { status: "ready" },
        &trace_id,
    )
}

async fn receive_webhook(
    state: AppState,
    headers: HeaderMap,
    provider: String,
    kind: WebhookKind,
    body: Bytes,
) -> axum::response::Response {
    let trace_id = trace_id_from_headers(&headers);
    let provider = match parse_provider(&provider, "receive_webhook") {
        Ok(provider) => provider,
        Err(error) => return error_response(error, &trace_id),
    };

    let headers = header_map_to_strings(&headers);
    match state
        .service
        .receive_webhook(provider, kind, headers, body.to_vec())
        .await
    {
        Ok(webhook) => success_response(StatusCode::OK, webhook_response(webhook), &trace_id),
        Err(error) => error_response(error, &trace_id),
    }
}

fn request_context(
    caller: AuthenticatedCaller,
    trace_id: &str,
    headers: &HeaderMap,
) -> RequestContext {
    RequestContext {
        caller_id: caller.caller_id,
        trace_id: trace_id.to_owned(),
        idempotency_key: headers
            .get("idempotency-key")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned),
    }
}

fn header_map_to_strings(headers: &HeaderMap) -> BTreeMap<String, String> {
    headers
        .iter()
        .filter_map(|(name, value)| {
            value
                .to_str()
                .ok()
                .map(|value| (name.as_str().to_ascii_lowercase(), value.to_owned()))
        })
        .collect()
}

fn json_rejection_error(rejection: JsonRejection, operation: &'static str) -> ApiError {
    ApiError::invalid_request(
        format!("JSON request body is invalid: {rejection}"),
        operation,
    )
}

fn webhook_response(webhook: crate::services::VerifiedWebhook) -> serde_json::Value {
    serde_json::json!({
        "provider": webhook.provider,
        "kind": match webhook.kind {
            WebhookKind::Payment => "payment",
            WebhookKind::Refund => "refund",
        },
        "deduplication_key": webhook.deduplication_key,
        "duplicate": webhook.duplicate,
    })
}

#[allow(dead_code)]
fn _api_error_type_check(error: ApiError) -> ApiError {
    error
}
