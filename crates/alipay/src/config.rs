#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlipayEnvironment {
    Production,
    Sandbox,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlipayConfig {
    pub environment: AlipayEnvironment,
    pub app_id: String,
    pub api_base_url: String,
    pub default_notify_url: Option<String>,
    pub default_return_url: Option<String>,
}

impl AlipayConfig {
    pub fn sandbox(app_id: impl Into<String>) -> Self {
        Self {
            environment: AlipayEnvironment::Sandbox,
            app_id: app_id.into(),
            api_base_url: "https://openapi-sandbox.dl.alipaydev.com/gateway.do".to_owned(),
            default_notify_url: None,
            default_return_url: None,
        }
    }

    pub fn production(app_id: impl Into<String>) -> Self {
        Self {
            environment: AlipayEnvironment::Production,
            app_id: app_id.into(),
            api_base_url: "https://openapi.alipay.com/gateway.do".to_owned(),
            default_notify_url: None,
            default_return_url: None,
        }
    }
}
