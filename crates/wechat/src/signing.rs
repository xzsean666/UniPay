use crate::error::ProviderResult;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WechatSignRequest<'a> {
    pub method: &'a str,
    pub path_and_query: &'a str,
    pub timestamp: i64,
    pub nonce: &'a str,
    pub body: &'a [u8],
}

impl WechatSignRequest<'_> {
    pub fn canonical_message(&self) -> Vec<u8> {
        let body = String::from_utf8_lossy(self.body);
        format!(
            "{}\n{}\n{}\n{}\n{}\n",
            self.method, self.path_and_query, self.timestamp, self.nonce, body
        )
        .into_bytes()
    }
}

pub trait WechatRequestSigner: Send + Sync {
    fn sign(&self, message: &[u8]) -> ProviderResult<String>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WechatResponseVerificationInput<'a> {
    pub timestamp: &'a str,
    pub nonce: &'a str,
    pub serial: &'a str,
    pub signature: &'a str,
    pub body: &'a [u8],
}

impl WechatResponseVerificationInput<'_> {
    pub fn canonical_message(&self) -> Vec<u8> {
        let body = String::from_utf8_lossy(self.body);
        format!("{}\n{}\n{}\n", self.timestamp, self.nonce, body).into_bytes()
    }
}

pub trait WechatResponseVerifier: Send + Sync {
    fn verify_response(&self, input: &WechatResponseVerificationInput<'_>) -> ProviderResult<()>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WechatWebhookVerificationInput<'a> {
    pub timestamp: &'a str,
    pub nonce: &'a str,
    pub serial: &'a str,
    pub signature: &'a str,
    pub body: &'a [u8],
}

impl WechatWebhookVerificationInput<'_> {
    pub fn canonical_message(&self) -> Vec<u8> {
        let body = String::from_utf8_lossy(self.body);
        format!("{}\n{}\n{}\n", self.timestamp, self.nonce, body).into_bytes()
    }
}

pub trait WechatWebhookVerifier: Send + Sync {
    fn verify_webhook(&self, input: &WechatWebhookVerificationInput<'_>) -> ProviderResult<()>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WechatEncryptedResource {
    pub algorithm: String,
    pub ciphertext: String,
    pub associated_data: Option<String>,
    pub nonce: String,
    pub original_type: Option<String>,
}

pub trait WechatResourceDecryptor: Send + Sync {
    fn decrypt(&self, resource: &WechatEncryptedResource) -> ProviderResult<Vec<u8>>;
}
