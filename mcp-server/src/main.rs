use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use panicless_mcp_server::config::Config;
use panicless_mcp_server::mcp::MCPServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging (to stderr to avoid interfering with stdin/stdout)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,panicless_mcp_server=debug,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    tracing::info!("Starting Panicless Library MCP Server (stdio)");

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

    // Create and run MCP server
    let server = MCPServer::new(pool, config.user_id);
    tracing::info!("MCP Server ready for user_id: {}", config.user_id);
    tracing::info!("MCP Server ready to receive requests on stdin");

    server.run().await?;

    Ok(())
}
