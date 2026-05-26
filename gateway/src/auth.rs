use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;

use crate::api::AppState;
use crate::api::response::error_response;
use crate::api::trace::trace_id_from_headers;
use crate::error::ApiError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthenticatedCaller {
    pub caller_id: String,
}

pub async fn require_api_key(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Response {
    let trace_id = trace_id_from_headers(request.headers());
    let supplied_key = match bearer_token(request.headers().get(axum::http::header::AUTHORIZATION))
    {
        Some(key) => key,
        None => return error_response(ApiError::unauthorized(), &trace_id),
    };

    let Some(api_key) = state.config.authenticate_api_key(supplied_key) else {
        return error_response(ApiError::unauthorized(), &trace_id);
    };

    request.extensions_mut().insert(AuthenticatedCaller {
        caller_id: api_key.caller_id().to_owned(),
    });

    next.run(request).await
}

fn bearer_token(header: Option<&axum::http::HeaderValue>) -> Option<&str> {
    let value = header?.to_str().ok()?.trim();
    let (scheme, token) = value.split_once(' ')?;

    if scheme.eq_ignore_ascii_case("Bearer") && !token.trim().is_empty() {
        Some(token.trim())
    } else {
        None
    }
}
