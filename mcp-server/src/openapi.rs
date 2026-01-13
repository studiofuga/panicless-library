use axum::Json;
use serde_json::json;

/// Generate OpenAPI schema for the MCP server
pub async fn get_openapi_schema() -> Json<serde_json::Value> {
    let schema = json!({
        "openapi": "3.1.0",
        "info": {
            "title": "Panicless Library MCP Server",
            "description": "Model Context Protocol server for Panicless Library providing access to books, readings, and library management",
            "version": env!("CARGO_PKG_VERSION")
        },
        "servers": [
            {
                "url": "/",
                "description": "MCP Server"
            }
        ],
        "paths": {
            "/health": {
                "get": {
                    "summary": "Health check",
                    "operationId": "health_check",
                    "tags": ["system"],
                    "responses": {
                        "200": {
                            "description": "Server is healthy",
                            "content": {
                                "text/plain": {
                                    "schema": {
                                        "type": "string"
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/mcp": {
                "post": {
                    "summary": "MCP Protocol endpoint (JSON-RPC 2.0)",
                    "description": "Accept JSON-RPC 2.0 requests for MCP operations",
                    "operationId": "handle_mcp_request",
                    "tags": ["mcp"],
                    "security": [
                        {
                            "bearerAuth": []
                        }
                    ],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "jsonrpc": {
                                            "type": "string",
                                            "enum": ["2.0"]
                                        },
                                        "method": {
                                            "type": "string",
                                            "enum": ["initialize", "tools/list", "tools/call"]
                                        },
                                        "params": {
                                            "type": "object"
                                        },
                                        "id": {
                                            "oneOf": [
                                                {"type": "string"},
                                                {"type": "number"}
                                            ]
                                        }
                                    },
                                    "required": ["jsonrpc", "method"]
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "JSON-RPC response",
                            "content": {
                                "text/event-stream": {
                                    "schema": {
                                        "type": "object"
                                    }
                                }
                            }
                        },
                        "400": {
                            "description": "Invalid request"
                        },
                        "401": {
                            "description": "Unauthorized"
                        }
                    }
                },
                "get": {
                    "summary": "MCP SSE Stream endpoint",
                    "description": "Server-Sent Events stream for MCP protocol",
                    "operationId": "handle_mcp_sse",
                    "tags": ["mcp"],
                    "security": [
                        {
                            "bearerAuth": []
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Server-Sent Events stream",
                            "content": {
                                "text/event-stream": {
                                    "schema": {
                                        "type": "object"
                                    }
                                }
                            }
                        },
                        "401": {
                            "description": "Unauthorized"
                        }
                    }
                }
            }
        },
        "components": {
            "securitySchemes": {
                "bearerAuth": {
                    "type": "http",
                    "scheme": "bearer",
                    "bearerFormat": "JWT",
                    "description": "JWT token for authentication"
                }
            }
        },
        "tags": [
            {
                "name": "system",
                "description": "System endpoints"
            },
            {
                "name": "mcp",
                "description": "Model Context Protocol endpoints"
            }
        ]
    });

    Json(schema)
}
