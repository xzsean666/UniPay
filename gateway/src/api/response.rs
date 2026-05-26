use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::api::models::{ErrorEnvelope, SuccessEnvelope};
use crate::error::ApiError;

pub(crate) fn success_response<T: Serialize>(
    status: StatusCode,
    data: T,
    trace_id: &str,
) -> Response {
    let envelope = SuccessEnvelope {
        success: true,
        data,
        trace_id: trace_id.to_owned(),
    };

    (status, Json(envelope)).into_response()
}

pub(crate) fn error_response(error: ApiError, trace_id: &str) -> Response {
    let status = error.status();
    let envelope = ErrorEnvelope {
        success: false,
        error: error.to_detail(),
        trace_id: trace_id.to_owned(),
    };

    (status, Json(envelope)).into_response()
}
