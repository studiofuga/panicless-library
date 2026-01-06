use panicless_backend::{create_pool, create_router, Config};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing/logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,panicless_backend=debug,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Panicless Library Backend");

    // Load configuration
    let config = Config::from_env()?;
    tracing::info!("Configuration loaded");
    tracing::info!("Environment: {}", config.environment);
    tracing::info!("Server address: {}", config.server_address());

    // Create database connection pool
    let pool = create_pool(&config.database_url).await?;
    tracing::info!("Database connection pool created");

    // Test database connection
    panicless_backend::db::test_connection(&pool).await?;
    tracing::info!("Database connection test successful");

    // Create router
    let app = create_router(pool, config.clone());
    tracing::info!("Router created with all endpoints");

    // Start server
    let listener = tokio::net::TcpListener::bind(&config.server_address()).await?;
    tracing::info!("Server listening on http://{}", config.server_address());

    tracing::info!("🚀 Panicless Library Backend is running!");
    tracing::info!("📚 API available at: http://{}/api", config.server_address());
    tracing::info!("💚 Health check at: http://{}/health", config.server_address());

    axum::serve(listener, app)
        .await?;

    Ok(())
}
