use axum::{
    extract::State,
    http::StatusCode,
    response::sse::{Event, Sse},
};
use futures::stream::{self, Stream, StreamExt};
use serde_json::{json, Value};
use std::convert::Infallible;

use panicless_mcp_lib as mcp;

use crate::{db::DbPool, middleware::auth::Claims};

/// Handle SSE stream for MCP protocol
/// Claude Desktop remote MCP connectors use SSE for bidirectional communication
pub async fn handle_mcp_sse(
    State(_pool): State<DbPool>,
    _claims: Claims,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {

    // Create initial response stream with server initialization
    let stream = stream::once(async move {
        // Send initialization info
        let init_event = json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {
                "server_info": {
                    "name": "panicless-backend-mcp",
                    "version": env!("CARGO_PKG_VERSION")
                }
            }
        });

        Ok::<_, Infallible>(Event::default().data(init_event.to_string()))
    })
    .chain(stream::once(async move {
        // Send available tools list
        let tools = mcp::tools::get_tool_definitions();
        let tools_event = json!({
            "jsonrpc": "2.0",
            "method": "tools/list",
            "params": {
                "tools": tools
            }
        });

        Ok::<_, Infallible>(Event::default().data(tools_event.to_string()))
    }));

    Sse::new(stream)
}

/// Handle MCP request via POST and return SSE-formatted response
pub async fn handle_mcp_sse_post(
    State(pool): State<DbPool>,
    claims: Claims,
    axum::Json(request): axum::Json<Value>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, (StatusCode, String)> {
    let user_id = claims.sub;

    // Validate JSON-RPC 2.0 format
    let request_obj = request
        .as_object()
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Invalid JSON-RPC request".to_string()))?;

    let jsonrpc = request_obj
        .get("jsonrpc")
        .and_then(|v| v.as_str())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing or invalid jsonrpc version".to_string()))?;

    if jsonrpc != "2.0" {
        return Err((StatusCode::BAD_REQUEST, "jsonrpc must be 2.0".to_string()));
    }

    let method = request_obj
        .get("method")
        .and_then(|v| v.as_str())
        .ok_or_else(|| (StatusCode::BAD_REQUEST, "Missing method".to_string()))?;

    let id = request_obj.get("id").cloned();
    let params = request_obj.get("params").cloned();

    // Process the request and create response stream
    let response = match method {
        "initialize" => {
            handle_initialize(id, params)
        }
        "tools/list" => {
            handle_tools_list(id)
        }
        "tools/call" => {
            handle_tools_call(&pool, id, params, user_id).await
        }
        _ => {
            json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32601,
                    "message": "Method not found"
                },
                "id": id
            })
        }
    };

    // Wrap response in SSE event
    let stream = stream::once(async move {
        Ok::<_, Infallible>(Event::default().data(response.to_string()))
    });

    Ok(Sse::new(stream))
}

fn handle_initialize(id: Option<Value>, params: Option<Value>) -> Value {
    let params = match params {
        Some(p) => match serde_json::from_value::<mcp::protocol::InitializeParams>(p) {
            Ok(params) => params,
            Err(e) => {
                return json!({
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
            return json!({
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
            name: "panicless-backend-mcp".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
    };

    json!({
        "jsonrpc": "2.0",
        "result": result,
        "id": id
    })
}

fn handle_tools_list(id: Option<Value>) -> Value {
    let tools = mcp::tools::get_tool_definitions();

    json!({
        "jsonrpc": "2.0",
        "result": {
            "tools": tools
        },
        "id": id
    })
}

async fn handle_tools_call(
    pool: &DbPool,
    id: Option<Value>,
    params: Option<Value>,
    user_id: i32,
) -> Value {
    let params = match params {
        Some(p) => match serde_json::from_value::<mcp::protocol::ToolCallParams>(p) {
            Ok(params) => params,
            Err(e) => {
                return json!({
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
            return json!({
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
            json!({
                "jsonrpc": "2.0",
                "result": result,
                "id": id
            })
        }
        Err(err) => {
            tracing::error!("Tool execution failed: {}", err);
            json!({
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
