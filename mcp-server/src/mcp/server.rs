use serde_json::Value;
use sqlx::PgPool;
use std::io::{self, BufRead, Write};

use super::protocol::*;
use super::tools;

pub struct MCPServer {
    pool: PgPool,
}

impl MCPServer {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("MCP Server starting - ready to receive requests on stdin");

        let stdin = io::stdin();
        let mut stdout = io::stdout();

        for line in stdin.lock().lines() {
            let line = line?;

            if line.trim().is_empty() {
                continue;
            }

            tracing::debug!("Received request: {}", line);

            let response = self.handle_request(&line).await;

            let response_json = serde_json::to_string(&response)?;
            tracing::debug!("Sending response: {}", response_json);

            writeln!(stdout, "{}", response_json)?;
            stdout.flush()?;
        }

        Ok(())
    }

    async fn handle_request(&self, request_str: &str) -> Value {
        // Parse JSON-RPC request
        let request: JsonRpcRequest = match serde_json::from_str(request_str) {
            Ok(req) => req,
            Err(e) => {
                tracing::error!("Failed to parse request: {}", e);
                return serde_json::to_value(JsonRpcError::parse_error(None)).unwrap();
            }
        };

        // Validate JSON-RPC version
        if request.jsonrpc != "2.0" {
            return serde_json::to_value(JsonRpcError::invalid_request(request.id)).unwrap();
        }

        // Handle method
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request.id, request.params),
            "tools/list" => self.handle_tools_list(request.id),
            "tools/call" => self.handle_tools_call(request.id, request.params).await,
            _ => serde_json::to_value(JsonRpcError::method_not_found(
                request.id,
                request.method,
            ))
            .unwrap(),
        }
    }

    fn handle_initialize(&self, id: Option<Value>, params: Option<Value>) -> Value {
        let params: InitializeParams = match params {
            Some(p) => match serde_json::from_value(p) {
                Ok(params) => params,
                Err(e) => {
                    return serde_json::to_value(JsonRpcError::invalid_params(
                        id,
                        format!("Invalid params: {}", e),
                    ))
                    .unwrap();
                }
            },
            None => {
                return serde_json::to_value(JsonRpcError::invalid_params(
                    id,
                    "Missing params".to_string(),
                ))
                .unwrap();
            }
        };

        tracing::info!(
            "Initializing MCP server for client: {} v{}",
            params.client_info.name,
            params.client_info.version
        );

        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ServerCapabilities {
                tools: ToolsCapability {
                    list_changed: false,
                },
            },
            server_info: ServerInfo {
                name: "panicless-mcp-server".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        serde_json::to_value(JsonRpcResponse::success(id, serde_json::to_value(result).unwrap()))
            .unwrap()
    }

    fn handle_tools_list(&self, id: Option<Value>) -> Value {
        let tools = tools::get_tool_definitions();

        serde_json::to_value(JsonRpcResponse::success(
            id,
            serde_json::json!({
                "tools": tools
            }),
        ))
        .unwrap()
    }

    async fn handle_tools_call(&self, id: Option<Value>, params: Option<Value>) -> Value {
        let params: ToolCallParams = match params {
            Some(p) => match serde_json::from_value(p) {
                Ok(params) => params,
                Err(e) => {
                    return serde_json::to_value(JsonRpcError::invalid_params(
                        id,
                        format!("Invalid params: {}", e),
                    ))
                    .unwrap();
                }
            },
            None => {
                return serde_json::to_value(JsonRpcError::invalid_params(
                    id,
                    "Missing params".to_string(),
                ))
                .unwrap();
            }
        };

        // Note: user_id would need to come from somewhere (e.g., JWT claims in HTTP context)
        // For backward compatibility with old stdio-based server, use user_id 0
        match tools::execute_tool(&self.pool, &params.name, params.arguments, 0).await {
            Ok(result) => {
                serde_json::to_value(JsonRpcResponse::success(
                    id,
                    serde_json::to_value(result).unwrap(),
                ))
                .unwrap()
            }
            Err(err) => {
                serde_json::to_value(JsonRpcError::internal_error(id, err)).unwrap()
            }
        }
    }
}
