use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;

pub type DbPool = PgPool;

pub async fn create_pool(database_url: &str) -> Result<DbPool, sqlx::Error> {
    tracing::info!("Creating database connection pool");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(database_url)
        .await?;

    tracing::info!("Database connection pool created successfully");

    Ok(pool)
}

pub async fn run_migrations(pool: &DbPool) -> Result<(), sqlx::Error> {
    tracing::info!("Running database migrations");

    sqlx::migrate!("../database/migrations")
        .run(pool)
        .await?;

    tracing::info!("Migrations completed successfully");

    Ok(())
}

pub async fn test_connection(pool: &DbPool) -> Result<(), sqlx::Error> {
    tracing::info!("Testing database connection");

    sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await?;

    tracing::info!("Database connection test successful");

    Ok(())
}
