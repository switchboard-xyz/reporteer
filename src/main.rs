use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use askama::Template;
use log::{info, warn};
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

// Struct to hold the derived key hash
#[derive(Clone)]
struct AppState {
    derived_key_hash: Arc<RwLock<String>>,
}

// Template for the HTML page
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    derived_key_hash: String,
}

// JSON response structure
#[derive(serde::Serialize)]
struct HashResponse {
    derived_key_hash: String,
}

// Handler for the HTML endpoint
async fn index(state: web::Data<AppState>) -> impl Responder {
    let hash = state.derived_key_hash.read().await;
    let template = IndexTemplate {
        derived_key_hash: hash.clone(),
    };

    match template.render() {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

// Handler for the JSON endpoint
async fn get_hash(state: web::Data<AppState>) -> impl Responder {
    let hash = state.derived_key_hash.read().await;
    let response = HashResponse {
        derived_key_hash: hash.clone(),
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

    // Create application state
    let app_state = web::Data::new(AppState {
        derived_key_hash: Arc::new(RwLock::new(derived_key_hash)),
    });

    // Start the web server
    info!("Starting server on port {}", config.server_port());
    let enclave_key = match EnclaveKeys::get_derived_key() {
        Ok(derived_key) => {
            let key_vec: [u8; 32] = derived_key.as_ref().to_vec();
            if key_vec.len() < 32 {
                warn!("Derived key too short: {} bytes", key_vec.len());
                return Ok(());
            }
            key_vec
        }
        Err(e) => {
            warn!("Failed to get derived key: {}", e);
            return Ok(());
        }
    };

    // Access first byte if vector is not empty
    if !enclave_key.is_empty() {
        println!("Enclave key first byte: {:?}", enclave_key[0]);
    }

    let report = AmdSevSnpAttestation::attest(b"hola").await.unwrap();
    println!("Report: {:?}", report);

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/", web::get().to(index))
            .route("/api/hash", web::get().to(get_hash))
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
