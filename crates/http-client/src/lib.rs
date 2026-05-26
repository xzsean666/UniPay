use reqwest::{Client, Method};
use serde::Serialize;
use serde_json::Value;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HttpClientError {
    #[error("transport error: {0}")]
    Transport(String),
    #[error("failed to serialize request body: {0}")]
    Serialization(String),
}

pub type HttpClientResult<T> = Result<T, HttpClientError>;

#[derive(Clone)]
pub struct PaymentHttpClient {
    client: Client,
}

impl PaymentHttpClient {
    pub fn new(timeout: Duration) -> HttpClientResult<Self> {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(|error| HttpClientError::Transport(error.to_string()))?;
        Ok(Self { client })
    }

    pub async fn send_json<T: Serialize + ?Sized>(
        &self,
        method: Method,
        url: &str,
        headers: Vec<(String, String)>,
        body: Option<&T>,
    ) -> HttpClientResult<PaymentHttpResponse> {
        let mut request = self.client.request(method, url);
        for (name, value) in headers {
            request = request.header(name, value);
        }
        if let Some(body) = body {
            request = request.json(body);
        }

        let response = request
            .send()
            .await
            .map_err(|error| HttpClientError::Transport(error.to_string()))?;
        let status = response.status().as_u16();
        let body = response
            .json::<Value>()
            .await
            .map_err(|error| HttpClientError::Transport(error.to_string()))?;
        Ok(PaymentHttpResponse { status, body })
    }
}

#[derive(Debug, Clone)]
pub struct PaymentHttpResponse {
    pub status: u16,
    pub body: Value,
}
