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
    /// Verify the attestation report before starting
    verify_at_start: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
              server_port: 3000,
              log_level: "info".to_string(),
              verify_at_start: false,
            log_level: "info".to_string(),
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
          let verify_at_start = env::var("VERIFY_AT_START")
              .map(|v| v.parse::<bool>().unwrap_or(false))
          Ok(Self {
              endpoint_url,
              server_port,
              log_level,
              verify_at_start,
          })
            endpoint_url,
            server_port,
            log_level,
        })
    }

    pub fn endpoint_url(&self) -> &Url {
        &self.endpoint_url
    }

    pub fn server_port(&self) -> u16 {
        self.server_port
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
    }

    #[test]
    fn test_config_from_env() {
        env::set_var("REPORTEER_ENDPOINT_URL", "http://localhost:9000/key");
        env::set_var("REPORTEER_SERVER_PORT", "8080");
        env::set_var("REPORTEER_LOG_LEVEL", "debug");

        let config = Config::from_env().unwrap();
        assert_eq!(config.endpoint_url().as_str(), "http://localhost:9000/key");
        assert_eq!(config.server_port(), 8080);
        assert_eq!(config.log_level(), "debug");

        // Clean up environment
        env::remove_var("REPORTEER_ENDPOINT_URL");
        env::remove_var("REPORTEER_SERVER_PORT");
        env::remove_var("REPORTEER_LOG_LEVEL");
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
