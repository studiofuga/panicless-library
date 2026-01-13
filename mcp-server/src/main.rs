use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use panicless_mcp_server::{config::Config, auth::auth_middleware, sse, openapi};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging (to stderr to avoid interfering with HTTP output)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,panicless_mcp_server=debug,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    tracing::info!("Starting Panicless Library MCP Server (HTTP)");

    // Load configuration
    let config = Config::from_env()?;
    tracing::info!("Configuration loaded");

    // Create database connection pool
    tracing::info!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&config.database_url)
        .await?;

    tracing::info!("Database connection established");

    // Test connection
    sqlx::query("SELECT 1")
        .fetch_one(&pool)
        .await?;

    tracing::info!("Database connection test successful");

    // Create router with protected and unprotected routes
    let app = Router::new()
        // Unprotected routes
        .route("/health", get(health_check))
        .route("/openapi.json", get(openapi::get_openapi_schema))
        // Protected routes (require JWT authentication)
        .route("/mcp", get(sse::handle_mcp_sse))
        .route("/mcp", post(sse::handle_mcp_sse_post))
        // Apply authentication middleware to protected routes
        .layer(middleware::from_fn_with_state(
            config.clone(),
            auth_middleware,
        ))
        // Add CORS layer
        .layer(CorsLayer::permissive())
        // Add state
        .with_state(pool);

    // Start HTTP server
    let listener = tokio::net::TcpListener::bind(&config.server_address())
        .await?;

    tracing::info!("MCP Server listening on http://{}", config.server_address());
    tracing::info!("Endpoints:");
    tracing::info!("  GET  /health              - Health check (no auth)");
    tracing::info!("  GET  /openapi.json        - OpenAPI schema (no auth)");
    tracing::info!("  GET  /mcp                 - MCP SSE stream (JWT auth)");
    tracing::info!("  POST /mcp                 - MCP JSON-RPC over SSE (JWT auth)");

    axum::serve(listener, app).await?;

    Ok(())
}

/// Health check endpoint (no auth required)
async fn health_check() -> &'static str {
    "OK"
}
