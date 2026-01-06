use panicless_mcp_server::MCPServer;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::time::Duration;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing/logging (to stderr to not interfere with stdio protocol)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,panicless_mcp_server=debug,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    tracing::info!("Starting Panicless Library MCP Server");

    // Load environment
    dotenvy::dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    // Create database connection pool (read-only)
    tracing::info!("Connecting to database...");
    let pool = PgPoolOptions::new()
        .max_connections(3)
        .acquire_timeout(Duration::from_secs(3))
        .connect(&database_url)
        .await?;

    tracing::info!("Database connection established");

    // Test connection
    sqlx::query("SELECT 1")
        .fetch_one(&pool)
        .await?;

    tracing::info!("Database connection test successful");

    // Create and run MCP server
    let server = MCPServer::new(pool);

    tracing::info!("MCP Server ready - listening on stdin/stdout");
    tracing::info!("Protocol: JSON-RPC 2.0 over stdio");
    tracing::info!("Available tools: search_books, get_book_details, list_readings, get_reading_statistics, find_similar_books");

    server.run().await?;

    Ok(())
}
