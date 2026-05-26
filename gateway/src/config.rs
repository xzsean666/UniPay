use std::env;
use std::fmt;
use std::net::SocketAddr;

#[derive(Clone, Debug)]
pub struct GatewayConfig {
    bind_addr: SocketAddr,
    api_keys: Vec<ApiKeyConfig>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApiKeyConfig {
    caller_id: String,
    secret: String,
}

#[derive(Debug)]
pub enum ConfigError {
    InvalidBindAddress(String),
    MissingApiKeys,
    InvalidApiKeyEntry(String),
}

impl GatewayConfig {
    pub fn new(bind_addr: SocketAddr, api_keys: Vec<ApiKeyConfig>) -> Result<Self, ConfigError> {
        if api_keys.is_empty() {
            return Err(ConfigError::MissingApiKeys);
        }

        Ok(Self {
            bind_addr,
            api_keys,
        })
    }

    pub fn from_env() -> Result<Self, ConfigError> {
        let bind_addr = env::var("UNIPAY_GATEWAY_BIND_ADDR")
            .unwrap_or_else(|_| "127.0.0.1:8080".to_owned())
            .parse()
            .map_err(|_| ConfigError::InvalidBindAddress("UNIPAY_GATEWAY_BIND_ADDR".to_owned()))?;

        let api_keys = env::var("UNIPAY_GATEWAY_API_KEYS")
            .map_err(|_| ConfigError::MissingApiKeys)
            .and_then(|raw| parse_api_keys(&raw))?;

        Self::new(bind_addr, api_keys)
    }

    pub fn bind_addr(&self) -> SocketAddr {
        self.bind_addr
    }

    pub fn authenticate_api_key(&self, supplied_secret: &str) -> Option<&ApiKeyConfig> {
        self.api_keys.iter().find(|candidate| {
            constant_time_eq(candidate.secret.as_bytes(), supplied_secret.as_bytes())
        })
    }

    pub fn api_keys(&self) -> &[ApiKeyConfig] {
        &self.api_keys
    }

    pub fn for_tests() -> Self {
        Self::new(
            "127.0.0.1:0"
                .parse()
                .expect("test socket address must parse"),
            vec![ApiKeyConfig::new("test-caller", "test-api-key")],
        )
        .expect("test config must include an API key")
    }
}

impl ApiKeyConfig {
    pub fn new(caller_id: impl Into<String>, secret: impl Into<String>) -> Self {
        Self {
            caller_id: caller_id.into(),
            secret: secret.into(),
        }
    }

    pub fn caller_id(&self) -> &str {
        &self.caller_id
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidBindAddress(variable) => {
                write!(formatter, "{variable} must be a valid socket address")
            }
            Self::MissingApiKeys => write!(
                formatter,
                "UNIPAY_GATEWAY_API_KEYS must contain at least one caller_id:api_key entry"
            ),
            Self::InvalidApiKeyEntry(entry) => write!(
                formatter,
                "invalid API key entry {entry:?}; expected caller_id:api_key"
            ),
        }
    }
}

impl std::error::Error for ConfigError {}

fn parse_api_keys(raw: &str) -> Result<Vec<ApiKeyConfig>, ConfigError> {
    raw.split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(|entry| {
            let (caller_id, secret) = entry
                .split_once(':')
                .ok_or_else(|| ConfigError::InvalidApiKeyEntry(entry.to_owned()))?;

            if caller_id.trim().is_empty() || secret.trim().is_empty() {
                return Err(ConfigError::InvalidApiKeyEntry(entry.to_owned()));
            }

            Ok(ApiKeyConfig::new(caller_id.trim(), secret.trim()))
        })
        .collect::<Result<Vec<_>, _>>()
        .and_then(|keys| {
            if keys.is_empty() {
                Err(ConfigError::MissingApiKeys)
            } else {
                Ok(keys)
            }
        })
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    let mut diff = left.len() ^ right.len();
    let max_len = left.len().max(right.len());

    for index in 0..max_len {
        let left_byte = left.get(index).copied().unwrap_or_default();
        let right_byte = right.get(index).copied().unwrap_or_default();
        diff |= usize::from(left_byte ^ right_byte);
    }

    diff == 0
}
