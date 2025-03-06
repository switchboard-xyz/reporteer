use actix_web::{web, HttpResponse, Responder};
use askama::Template;
use log::info;
use std::sync::Arc;
use tokio::sync::RwLock;

mod config;
mod error;

use config::Config;
use error::{ReporteerError, Result};

// Template for the HTML page
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    attestation_report: String,
    derived_key_hash: String,
}

#[derive(Clone)]
struct AppState {
    attestation_report: Arc<RwLock<String>>,
    derived_key_hash: Arc<RwLock<String>>,
}

async fn index(state: web::Data<AppState>) -> impl Responder {
    let hash = state.derived_key_hash.read().await;
    let report = state.attestation_report.read().await;
    let template = IndexTemplate {
        attestation_report: report.clone(),
        derived_key_hash: hash.clone(),
    };

    match template.render() {
        Ok(html) => HttpResponse::Ok().content_type("text/html").body(html),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

async fn get_data(state: web::Data<AppState>) -> impl Responder {
    let hash = state.derived_key_hash.read().await;
    HttpResponse::Ok().json(serde_json::json!({
        "derived_key_hash": *hash,
    }))
}

#[actix_web::main]
async fn main() -> Result<()> {
    // Load configuration
    let config = Config::from_env().unwrap_or_default();

    // If verification is enabled, handle it here
    if config.verify_at_start {
        info!("Verification logic should be implemented here.");
    }

    info!("Starting server at {}:{}", "127.0.0.1", config.server_port);

    // Server startup logic remains placeholder

    Ok(())
}
