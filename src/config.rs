use crate::error::ReporteerError;
use serde::{Deserialize, Serialize};
use std::env;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The endpoint URL for fetching the derived key
    pub endpoint_url: Url,
    /// The port for the web server
    pub server_port: u16,
    /// Log level configuration
    pub log_level: String,
    /// Verify the attestation report before starting
    pub verify_at_start: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            endpoint_url: Url::parse("http://127.0.0.1:8006/derived_key").unwrap(),
            server_port: 3000,
            log_level: "info".to_string(),
            verify_at_start: false,
        }
    }
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, ReporteerError> {
        let endpoint_url = env::var("REPORTEER_ENDPOINT_URL")
            .map(|url| {
                Url::parse(&url).map_err(|e| {
                    ReporteerError::ConfigError(format!("Invalid endpoint URL: {}", e))
                })
            })
            .unwrap_or_else(|_| {
                Ok(Url::parse("http://127.0.0.1:8006/derived_key").expect("Invalid default URL"))
            })?;

        let server_port = env::var("REPORTEER_SERVER_PORT")
            .map(|port| {
                port.parse::<u16>()
                    .map_err(|e| ReporteerError::ConfigError(format!("Invalid server port: {}", e)))
            })
            .unwrap_or_else(|_| Ok(3000))?;

        let log_level = env::var("REPORTEER_LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

        let verify_at_start = env::var("VERIFY_AT_START")
            .map(|v| v.parse::<bool>().unwrap_or(false))
            .unwrap_or(false);

        Ok(Self {
            endpoint_url,
            server_port,
            log_level,
            verify_at_start,
        })
    }
}
