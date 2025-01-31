use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum ReporteerError {
    #[error("Failed to fetch derived key: {0}")]
    FetchError(#[from] reqwest::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

pub type Result<T> = std::result::Result<T, ReporteerError>;
