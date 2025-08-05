use anyhow::Result;
use axum::{
    http::StatusCode,
    response::Html,
    routing::{get, Router},
};
use clap::Parser;
use reqwest::Client;
use std::{net::SocketAddr, sync::Arc};
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    services::ServeDir,
    trace::TraceLayer,
};
use tracing::{info, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod cache;
mod models;

use cache::FileCache;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to run the server on
    #[arg(short, long, default_value = "3000")]
    port: u16,

    /// Host to bind the server to
    #[arg(long, default_value = "0.0.0.0")]
    host: String,

    /// Cache directory path
    #[arg(long, default_value = "cache")]
    cache_dir: String,

    /// Log level
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[derive(Clone)]
pub struct AppState {
    pub client: Client,
    pub cache: Arc<FileCache>,
}

impl AppState {
    pub fn new(cache_dir: &str) -> Result<Self> {
        let client = Client::builder()
            .user_agent("poe-gem-calculator/0.1.0")
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        let cache = Arc::new(FileCache::new(cache_dir)?);

        Ok(Self { client, cache })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize tracing
    let log_level = match args.log_level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("poe_gem_calculator={}", log_level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize application state
    let state = AppState::new(&args.cache_dir)?;

    // Clean up expired cache entries on startup
    if let Err(e) = state.cache.cleanup_expired().await {
        tracing::warn!("Failed to cleanup expired cache entries: {}", e);
    }

    // Build the application router
    let app = create_router(state);

    // Create socket address
    let addr = format!("{}:{}", args.host, args.port)
        .parse::<SocketAddr>()?;

    info!("Server starting on http://{}", addr);
    info!("Cache directory: {}", args.cache_dir);

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn create_router(state: AppState) -> Router {
    // API routes
    let api_routes = Router::new()
        .route("/leagues", get(api::get_leagues))
        .route("/skill-gems", get(api::get_skill_gems))
        .route("/calculate", get(api::calculate_gem_roi));

    // Main application router
    Router::new()
        .nest("/api", api_routes)
        .route("/health", get(health_check))
        .fallback_service(ServeDir::new("public").append_index_html_on_directories(true))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        )
        .with_state(state)
}

async fn health_check() -> Result<Html<&'static str>, StatusCode> {
    Ok(Html("<html><body><h1>POE Gem Calculator - Healthy</h1></body></html>"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::util::ServiceExt;

    #[tokio::test]
    async fn test_health_check() {
        let state = AppState::new("test_cache").unwrap();
        let app = create_router(state);

        let request = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_api_leagues_endpoint() {
        let state = AppState::new("test_cache").unwrap();
        let app = create_router(state);

        let request = Request::builder()
            .uri("/api/leagues")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        // Should either succeed or fail gracefully
        assert!(response.status().is_success() || response.status().is_server_error());
    }
}
