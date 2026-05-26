use std::collections::BTreeMap;

use async_trait::async_trait;

use crate::error::{ProviderError, ProviderResult};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderHttpRequest {
    pub method: String,
    pub url: String,
    pub path_and_query: String,
    pub headers: BTreeMap<String, String>,
    pub body: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderHttpResponse {
    pub status: u16,
    pub headers: BTreeMap<String, String>,
    pub body: Vec<u8>,
}

#[async_trait]
pub trait HttpTransport: Send + Sync {
    async fn send(&self, request: ProviderHttpRequest) -> ProviderResult<ProviderHttpResponse>;
}

impl From<ProviderError> for ProviderHttpResponse {
    fn from(error: ProviderError) -> Self {
        let body = serde_json::json!({
            "code": error.code.as_str(),
            "message": error.message,
        })
        .to_string()
        .into_bytes();

        Self {
            status: if error.retryable { 503 } else { 422 },
            headers: BTreeMap::new(),
            body,
        }
    }
}
