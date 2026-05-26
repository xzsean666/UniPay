use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use axum::http::HeaderMap;

static TRACE_COUNTER: AtomicU64 = AtomicU64::new(1);

pub(crate) fn trace_id_from_headers(headers: &HeaderMap) -> String {
    if let Some(value) = headers.get("x-request-id")
        && let Ok(value) = value.to_str()
    {
        let value = value.trim();
        if !value.is_empty() && value.len() <= 128 {
            return value.to_owned();
        }
    }

    let counter = TRACE_COUNTER.fetch_add(1, Ordering::Relaxed);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();

    format!("req_{nanos:032x}{counter:016x}")
}
