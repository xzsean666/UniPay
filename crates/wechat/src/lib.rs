//! WeChat Pay v3 provider adapter for UniPay.
//!
//! Provider-specific mapping and webhook entry points stay in this crate. The
//! public adapter implements `unipay_core::PaymentProvider` so SDK and Gateway
//! callers can remain provider-neutral.

pub mod adapter;
pub mod config;
pub mod error;
pub mod http;
pub mod mapping;
pub mod models;
pub mod signing;

pub use adapter::{WechatPayAdapter, WechatRequestContext};
pub use config::{WechatEnvironment, WechatPayConfig};
pub use error::{Operation, ProviderError, ProviderResult, UniPayErrorCode};
pub use http::{HttpTransport, ProviderHttpRequest, ProviderHttpResponse};
pub use mapping::{map_wechat_refund_status, map_wechat_trade_state, wechat_amount_from_money};
pub use models::*;
pub use signing::{
    WechatEncryptedResource, WechatRequestSigner, WechatResourceDecryptor,
    WechatResponseVerificationInput, WechatResponseVerifier, WechatSignRequest,
    WechatWebhookVerificationInput, WechatWebhookVerifier,
};
