use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use askama::Template;
use log::{debug, info, warn};
use sail_sdk;
use sail_sdk::AmdSevSnpAttestation;
use sail_sdk::EnclaveKeys;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tokio::sync::RwLock;

mod config;
mod error;

use config::Config;
use error::{ReporteerError, Result};

// Struct to hold the derived key hash and attestation report
#[derive(Clone)]
struct AppState {
    derived_key_hash: Arc<RwLock<String>>,
    attestation_report: Arc<RwLock<String>>,
    // Store the raw report object for API access
    raw_report: Arc<RwLock<Option<serde_json::Value>>>,
}

// Template for the HTML page
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    derived_key_hash: String,
    attestation_report: String,
}

// JSON response structures
#[derive(serde::Serialize)]
struct HashResponse {
    derived_key_hash: String,
}

#[derive(serde::Serialize)]
struct ReportResponse {
    attestation_report: String,
}

// Handler for the HTML endpoint
async fn index(state: web::Data<AppState>) -> impl Responder {
    let hash = state.derived_key_hash.read().await;
    let report = state.attestation_report.read().await;

    let template = IndexTemplate {
        derived_key_hash: hash.clone(),
        attestation_report: report.clone(),
    };

    match template.render() {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// Handler for the hash JSON endpoint
async fn get_hash(state: web::Data<AppState>) -> impl Responder {
    let hash = state.derived_key_hash.read().await;
    let response = HashResponse {
        derived_key_hash: hash.clone(),
    };

    HttpResponse::Ok().json(response)
}

// Handler for the attestation report JSON endpoint
async fn get_report(state: web::Data<AppState>) -> impl Responder {
    // Check if we have a raw report available
    let raw_report = state.raw_report.read().await;

    if let Some(raw) = raw_report.as_ref() {
        // Return the raw report directly as JSON
        return HttpResponse::Ok().json(raw);
    }

    // Fall back to the string representation
    let report = state.attestation_report.read().await;
    let response = ReportResponse {
        attestation_report: report.clone(),
    };

    HttpResponse::Ok().json(response)
}

// Handler for the health check endpoint
async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy"
    }))
}

// Function to fetch and hash the derived key
async fn fetch_derived_key(endpoint_url: &str) -> Result<String> {
    let client = reqwest::Client::new();
    let response = client
        .get(endpoint_url)
        .send()
        .await
        .map_err(ReporteerError::FetchError)?;

    let derived_key = response.text().await.map_err(ReporteerError::FetchError)?;

    // Create SHA-256 hash of the derived key
    let mut hasher = Sha256::new();
    hasher.update(derived_key.as_bytes());
    let hash = hasher.finalize();

    Ok(format!("{:x}", hash))
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> std::io::Result<()> {
    // Initialize logging
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    // Load configuration
    let config = Config::from_env().unwrap_or_else(|e| {
        warn!("Failed to load configuration: {}. Using defaults.", e);
        Config::default()
    });

    // Fetch and hash the derived key
    let derived_key_hash = match fetch_derived_key(config.endpoint_url().as_str()).await {
        Ok(hash) => hash,
        Err(e) => {
            warn!("Failed to fetch derived key: {}. Using placeholder.", e);
            "ERROR_FETCHING_KEY".to_string()
        }
    };

    // Log the hash at startup
    info!("Initial derived key hash: {}", derived_key_hash);

    // Log configuration settings
    info!(
        "Configuration: Server port={}, Verify on start={}",
        config.server_port(),
        config.verify_on_start()
    );

    // Default empty attestation report
    let default_report = "No attestation report available.".to_string();

    // Create application state
    let app_state = web::Data::new(AppState {
        derived_key_hash: Arc::new(RwLock::new(derived_key_hash)),
        attestation_report: Arc::new(RwLock::new(default_report)),
        raw_report: Arc::new(RwLock::new(None)),
    });

    // Get the derived key
    info!("Fetching Derived key");
    let enclave_key = match EnclaveKeys::get_derived_key() {
        Ok(derived_key) => {
            // Try to access the bytes directly to see what we're working with
            let bytes = derived_key.as_ref();
            debug!("Response content [in bytes]: {:?}", bytes);

            // Let's try to create a new array from the bytes
            let array: [u8; 32] = match bytes.try_into() {
                Ok(arr) => arr,
                Err(e) => {
                    debug!("Failed to convert to [u8; 32]: {:?}", e);
                    return Ok(());
                }
            };

            array
        }
        Err(e) => {
            warn!("Failed to get derived key: {}", e);
            debug!("Error details: {:?}", e);
            return Ok(());
        }
    };

    // Access first byte if vector is not empty
    if !enclave_key.is_empty() {
        info!("Enclave key first byte: {:?}", enclave_key[0]);
    }

    // Only perform verification if VERIFY_ON_START is enabled
    if config.verify_on_start() {
        info!("Verification on start is enabled, performing attestation and verification");
        let test_msg: Option<&str> = Some("hola");

        // Test AMD attestation
        let report = if let Some(msg) = test_msg {
            AmdSevSnpAttestation::attest(msg).await.unwrap()
        } else {
            return Ok(());
        };
        info!("Report: {:?}", report);

        // Store the attestation report in AppState using Debug formatting
        let report_string = format!("{:#?}", report);
        info!("Storing attestation report");

        // Store the pretty-printed version for HTML display
        let mut app_report = app_state.attestation_report.write().await;
        *app_report = report_string;

        // Create a custom JSON structure for the API
        let json_value = serde_json::json!({
            "report_type": "AMD SEV-SNP Attestation",
            "message": test_msg.unwrap_or("none"),
            "status": "verified",
            "details": format!("{:#?}", report)
        });

        // Store the JSON representation for API access
        let mut raw_report = app_state.raw_report.write().await;
        *raw_report = Some(json_value);

        // Test AMD report verification
        let verification = if let Some(msg) = test_msg {
            AmdSevSnpAttestation::verify(&report, Some(msg.as_bytes()))
                .await
                .unwrap()
        } else {
            return Ok(());
        };
        info!("Verification: {:?}", verification);
    } else {
        info!("Verification on start is disabled, but still generating attestation report");

        // Generate attestation report anyway for display purposes
        let test_msg = "reporteer";
        match AmdSevSnpAttestation::attest(test_msg).await {
            Ok(report) => {
                info!("Generated attestation report for display");

                // Store the attestation report in AppState using Debug formatting
                let report_string = format!("{:#?}", report);

                // Store the pretty-printed version for HTML display
                let mut app_report = app_state.attestation_report.write().await;
                *app_report = report_string;

                // Convert the report to a serde_json::Value for the API
                // This involves creating a custom JSON structure since the report doesn't implement Serialize
                let json_value = serde_json::json!({
                    "report_type": "AMD SEV-SNP Attestation",
                    "message": test_msg,
                    "status": "verified",
                    "details": format!("{:#?}", report)
                });

                // Store the JSON representation for API access
                let mut raw_report = app_state.raw_report.write().await;
                *raw_report = Some(json_value);
            }
            Err(e) => {
                warn!("Failed to generate attestation report: {:?}", e);
            }
        }
    }

    // Start web server
    info!("Starting server on port {}", config.server_port());
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/", web::get().to(index))
            .route("/api/hash", web::get().to(get_hash))
            .route("/api/report", web::get().to(get_report))
            .route("/health", web::get().to(health))
    })
    .bind(("0.0.0.0", config.server_port()))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test;

    #[actix_web::test]
    async fn test_get_hash_endpoint() {
        let app_state = web::Data::new(AppState {
            derived_key_hash: Arc::new(RwLock::new("test_hash".to_string())),
            attestation_report: Arc::new(RwLock::new("test_report".to_string())),
            raw_report: Arc::new(RwLock::new(None)),
        });

        let app = test::init_service(
            App::new()
                .app_data(app_state.clone())
                .route("/api/hash", web::get().to(get_hash)),
        )
        .await;

        let req = test::TestRequest::get().uri("/api/hash").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_index_endpoint() {
        let app_state = web::Data::new(AppState {
            derived_key_hash: Arc::new(RwLock::new("test_hash".to_string())),
            attestation_report: Arc::new(RwLock::new("test_report".to_string())),
            raw_report: Arc::new(RwLock::new(None)),
        });

        let app = test::init_service(
            App::new()
                .app_data(app_state.clone())
                .route("/", web::get().to(index)),
        )
        .await;

        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_report_endpoint() {
        let app_state = web::Data::new(AppState {
            derived_key_hash: Arc::new(RwLock::new("test_hash".to_string())),
            attestation_report: Arc::new(RwLock::new("test_report".to_string())),
            raw_report: Arc::new(RwLock::new(Some(serde_json::json!({"test": "value"})))),
        });

        let app = test::init_service(
            App::new()
                .app_data(app_state.clone())
                .route("/api/report", web::get().to(get_report)),
        )
        .await;

        let req = test::TestRequest::get().uri("/api/report").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_health_endpoint() {
        let app = test::init_service(App::new().route("/health", web::get().to(health))).await;

        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[tokio::test]
    async fn test_fetch_derived_key_error() {
        let result = fetch_derived_key("http://invalid-url").await;
        assert!(result.is_err());
    }
}
