use axum::{
    extract::State,
    http::{StatusCode},
    middleware,
    routing::{get, post},
    Json, Router,
};
use serde_json::Value;
use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use panicless_mcp_server::{config::Config, auth::{auth_middleware, Claims}, mcp};

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

    // Create router with protected routes
    let app = Router::new()
        // Health check (no auth required)
        .route("/health", get(health_check))
        // MCP protocol endpoint (auth required)
        .route("/mcp", post(handle_mcp_request))
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
    tracing::info!("Protocol: JSON-RPC 2.0 over HTTP POST /mcp");
    tracing::info!("Available tools: search_books, get_book_details, list_readings, get_reading_statistics, find_similar_books, create_book, create_reading, update_reading_review");

    axum::serve(listener, app).await?;

    Ok(())
}

/// Health check endpoint (no auth required)
async fn health_check() -> &'static str {
    "OK"
}

/// Handle MCP JSON-RPC 2.0 requests
/// Extracts Claims from JWT automatically via middleware
async fn handle_mcp_request(
    State(pool): State<sqlx::PgPool>,
    claims: Claims,
    Json(request): Json<Value>,
) -> Result<Json<Value>, (StatusCode, String)> {
    // Validate JSON-RPC 2.0 format
    let request_obj = request.as_object()
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Invalid JSON-RPC request".to_string()))?;

    let jsonrpc = request_obj.get("jsonrpc")
        .and_then(|v| v.as_str())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing or invalid jsonrpc version".to_string()))?;

    if jsonrpc != "2.0" {
        return Err((StatusCode::BAD_REQUEST, "jsonrpc must be 2.0".to_string()));
    }

    let method = request_obj.get("method")
        .and_then(|v| v.as_str())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing method".to_string()))?;

    let id = request_obj.get("id").cloned();
    let params = request_obj.get("params").cloned();

    // Dispatch to appropriate handler
    let response = match method {
        "initialize" => {
            handle_initialize(id, params)
        }
        "tools/list" => {
            handle_tools_list(id)
        }
        "tools/call" => {
            handle_tools_call(&pool, id, params, claims.user_id()).await
        }
        _ => {
            serde_json::json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32601,
                    "message": "Method not found"
                },
                "id": id
            })
        }
    };

    Ok(Json(response))
}

fn handle_initialize(id: Option<Value>, params: Option<Value>) -> Value {
    let params = match params {
        Some(p) => match serde_json::from_value::<mcp::protocol::InitializeParams>(p) {
            Ok(params) => params,
            Err(e) => {
                return serde_json::json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32602,
                        "message": format!("Invalid params: {}", e)
                    },
                    "id": id
                });
            }
        },
        None => {
            return serde_json::json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32602,
                    "message": "Missing params"
                },
                "id": id
            });
        }
    };

    tracing::info!(
        "Initializing MCP server for client: {} v{}",
        params.client_info.name,
        params.client_info.version
    );

    let result = mcp::protocol::InitializeResult {
        protocol_version: "2024-11-05".to_string(),
        capabilities: mcp::protocol::ServerCapabilities {
            tools: mcp::protocol::ToolsCapability {
                list_changed: false,
            },
        },
        server_info: mcp::protocol::ServerInfo {
            name: "panicless-mcp-server".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
    };

    serde_json::json!({
        "jsonrpc": "2.0",
        "result": result,
        "id": id
    })
}

fn handle_tools_list(id: Option<Value>) -> Value {
    let tools = mcp::tools::get_tool_definitions();

    serde_json::json!({
        "jsonrpc": "2.0",
        "result": {
            "tools": tools
        },
        "id": id
    })
}

async fn handle_tools_call(
    pool: &sqlx::PgPool,
    id: Option<Value>,
    params: Option<Value>,
    user_id: i32,
) -> Value {
    let params = match params {
        Some(p) => match serde_json::from_value::<mcp::protocol::ToolCallParams>(p) {
            Ok(params) => params,
            Err(e) => {
                return serde_json::json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32602,
                        "message": format!("Invalid params: {}", e)
                    },
                    "id": id
                });
            }
        },
        None => {
            return serde_json::json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32602,
                    "message": "Missing params"
                },
                "id": id
            });
        }
    };

    // Execute tool with user_id from JWT claims
    match mcp::tools::execute_tool(pool, &params.name, params.arguments, user_id).await {
        Ok(result) => {
            serde_json::json!({
                "jsonrpc": "2.0",
                "result": result,
                "id": id
            })
        }
        Err(err) => {
            tracing::error!("Tool execution failed: {}", err);
            serde_json::json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32603,
                    "message": err
                },
                "id": id
            })
        }
    }
}
