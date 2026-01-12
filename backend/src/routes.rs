use axum::{
    extract::FromRef,
    middleware,
    routing::{delete, get, patch, post, put},
    Router,
};
use tower_http::cors::CorsLayer;

use crate::{
    config::Config,
    db::DbPool,
    handlers,
    middleware::auth::auth_middleware,
};

#[derive(Clone)]
pub struct AppState {
    pub pool: DbPool,
    pub config: Config,
}

impl FromRef<AppState> for DbPool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

impl FromRef<AppState> for Config {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}

pub fn create_router(pool: DbPool, config: Config) -> Router {
    let state = AppState {
        pool,
        config: config.clone(),
    };

    // Configure CORS
    let cors = CorsLayer::permissive(); // TODO: Restrict in production using config.cors_allowed_origins

    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/api/auth/register", post(handlers::register))
        .route("/api/auth/login", post(handlers::login))
        .route("/api/auth/refresh", post(handlers::refresh))
        // OAuth2 token endpoint (no auth required, uses client credentials)
        .route("/oauth/token", post(handlers::token));

    // Protected routes (authentication required)
    let protected_routes = Router::new()
        // Auth
        .route("/api/auth/me", get(handlers::get_current_user))
        // OAuth2 authorize endpoint (requires authentication)
        .route("/oauth/authorize", post(handlers::authorize))
        // Users
        .route("/api/users/:id", get(handlers::get_user))
        .route("/api/users/:id", put(handlers::update_user))
        .route("/api/users/:id", delete(handlers::delete_user))
        // Books
        .route("/api/books", get(handlers::list_books))
        .route("/api/books", post(handlers::create_book))
        .route("/api/books/:id", get(handlers::get_book))
        .route("/api/books/:id", put(handlers::update_book))
        .route("/api/books/:id", delete(handlers::delete_book))
        .route("/api/books/:id/readings", get(handlers::get_book_readings))
        // Readings
        .route("/api/readings", get(handlers::list_readings))
        .route("/api/readings", post(handlers::create_reading))
        .route("/api/readings/:id", get(handlers::get_reading))
        .route("/api/readings/:id", put(handlers::update_reading))
        .route("/api/readings/:id", delete(handlers::delete_reading))
        .route("/api/readings/:id/complete", patch(handlers::complete_reading))
        .route("/api/readings/stats", get(handlers::get_reading_stats))
        // Connectors
        .route("/api/connectors", get(handlers::list_connectors))
        .route("/api/connectors", post(handlers::create_or_update_connector))
        .route("/api/connectors/:provider", get(handlers::get_connector))
        .route("/api/connectors/:provider", delete(handlers::delete_connector))
        .route("/api/connectors/:provider/toggle", patch(handlers::toggle_connector))
        // Import
        .route("/api/import/goodreads/csv", post(handlers::import_goodreads_csv))
        // Apply authentication middleware to all protected routes
        .layer(middleware::from_fn_with_state(
            config.clone(),
            auth_middleware,
        ));

    // Public metadata endpoints (no auth required)
    let public_metadata_routes = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/openapi.json", get(handlers::openapi_schema));

    // Combine all routes
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .merge(public_metadata_routes)
        .layer(cors)
        .with_state(state)
}
