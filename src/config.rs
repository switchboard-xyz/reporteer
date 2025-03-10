use crate::error::{ReporteerError, Result};
use serde::{Deserialize, Serialize};
use std::env;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The endpoint URL for fetching the derived key
    endpoint_url: Url,
    /// The port for the web server
    server_port: u16,
    /// Log level configuration
    log_level: String,
    /// Whether to verify on startup
    verify_on_start: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            endpoint_url: Url::parse("http://127.0.0.1:8006/derived_key")
                .expect("Invalid default URL"),
            server_port: 3000,
            log_level: "info".to_string(),
            verify_on_start: false,
        }
    }
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
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

        let verify_on_start = env::var("REPORTEER_VERIFY_ON_START")
            .map(|val| val.to_lowercase() == "true")
            .unwrap_or(false);

        Ok(Self {
            endpoint_url,
            server_port,
            log_level,
            verify_on_start,
        })
    }

    pub fn endpoint_url(&self) -> &Url {
        &self.endpoint_url
    }

    pub fn server_port(&self) -> u16 {
        self.server_port
    }

    pub fn verify_on_start(&self) -> bool {
        self.verify_on_start
    }

    pub fn log_level(&self) -> &str {
        &self.log_level
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(
            config.endpoint_url().as_str(),
            "http://127.0.0.1:8006/derived_key"
        );
        assert_eq!(config.server_port(), 3000);
        assert_eq!(config.log_level(), "info");
        assert_eq!(config.verify_on_start(), false);
    }

    #[test]
    fn test_config_from_env() {
        env::set_var("REPORTEER_ENDPOINT_URL", "http://localhost:9000/key");
        env::set_var("REPORTEER_SERVER_PORT", "8080");
        env::set_var("REPORTEER_LOG_LEVEL", "debug");
        env::set_var("REPORTEER_VERIFY_ON_START", "true");

        let config = Config::from_env().unwrap();
        assert_eq!(config.endpoint_url().as_str(), "http://localhost:9000/key");
        assert_eq!(config.server_port(), 8080);
        assert_eq!(config.log_level(), "debug");
        assert_eq!(config.verify_on_start(), true);

        // Clean up environment
        env::remove_var("REPORTEER_ENDPOINT_URL");
        env::remove_var("REPORTEER_SERVER_PORT");
        env::remove_var("REPORTEER_LOG_LEVEL");
        env::remove_var("REPORTEER_VERIFY_ON_START");
    }

    #[test]
    fn test_invalid_port() {
        env::set_var("REPORTEER_SERVER_PORT", "invalid");
        assert!(Config::from_env().is_err());
        env::remove_var("REPORTEER_SERVER_PORT");
    }

    #[test]
    fn test_invalid_url() {
        env::set_var("REPORTEER_ENDPOINT_URL", "not-a-url");
        assert!(Config::from_env().is_err());
        env::remove_var("REPORTEER_ENDPOINT_URL");
    }
}
