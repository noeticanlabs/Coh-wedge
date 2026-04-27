mod error;
mod routes;

use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use anyhow::Context;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initializing tracing (simple stdout logging)
    tracing_subscriber::fmt::init();

    let host = std::env::var("COH_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("COH_PORT").unwrap_or_else(|_| "3030".to_string());
    let addr_str = format!("{}:{}", host, port);
    let addr: SocketAddr = addr_str.parse().context("Failed to parse SocketAddr")?;

    let app = Router::new()
        .route("/health", get(routes::health_check))
        .route("/v1/verify-micro", post(routes::verify_micro_handler))
        .route("/v1/verify-chain", post(routes::verify_chain_handler))
        .route(
            "/v1/execute-verified",
            post(routes::execute_verified_handler),
        )
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    println!("🚀 Coh Sidecar active at http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("Failed to bind TCP listener")?;
    axum::serve(listener, app)
        .await
        .context("Axum server error")?;

    Ok(())
}
