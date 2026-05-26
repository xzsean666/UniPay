#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WechatEnvironment {
    Production,
    Sandbox,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WechatPayConfig {
    pub environment: WechatEnvironment,
    pub app_id: String,
    pub merchant_id: String,
    pub merchant_serial_no: String,
    pub api_base_url: String,
    pub default_notify_url: Option<String>,
}

impl WechatPayConfig {
    pub fn production(
        app_id: impl Into<String>,
        merchant_id: impl Into<String>,
        merchant_serial_no: impl Into<String>,
    ) -> Self {
        Self {
            environment: WechatEnvironment::Production,
            app_id: app_id.into(),
            merchant_id: merchant_id.into(),
            merchant_serial_no: merchant_serial_no.into(),
            api_base_url: "https://api.mch.weixin.qq.com".to_string(),
            default_notify_url: None,
        }
    }
}
