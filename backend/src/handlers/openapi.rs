use axum::{
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};

/// OpenAPI schema endpoint for Anthropic connector integration
/// Returns a complete OpenAPI spec that can be imported as tools in Anthropic
pub async fn openapi_schema() -> (StatusCode, Json<Value>) {
    let schema = json!({
        "openapi": "3.0.0",
        "info": {
            "title": "Panicless Library API",
            "description": "REST API for managing your personal library, readings, and notes. Connect this to Anthropic to enable Claude to access your library data.",
            "version": "1.0.0",
            "contact": {
                "name": "Panicless Library"
            }
        },
        "servers": [
            {
                "url": "{protocol}://{host}:{port}",
                "variables": {
                    "protocol": {
                        "default": "http"
                    },
                    "host": {
                        "default": "localhost"
                    },
                    "port": {
                        "default": "8080"
                    }
                }
            }
        ],
        "components": {
            "securitySchemes": {
                "bearerAuth": {
                    "type": "http",
                    "scheme": "bearer",
                    "bearerFormat": "JWT"
                }
            }
        },
        "security": [
            {
                "bearerAuth": []
            }
        ],
        "paths": {
            "/api/books": {
                "get": {
                    "summary": "List all books in your library",
                    "description": "Retrieves a paginated list of all books added to your personal library",
                    "tags": ["Books"],
                    "parameters": [
                        {
                            "name": "page",
                            "in": "query",
                            "description": "Page number (default: 1)",
                            "schema": { "type": "integer", "default": 1 }
                        },
                        {
                            "name": "limit",
                            "in": "query",
                            "description": "Number of results per page (default: 50)",
                            "schema": { "type": "integer", "default": 50 }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "List of books",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "array",
                                        "items": {
                                            "type": "object",
                                            "properties": {
                                                "id": { "type": "integer" },
                                                "title": { "type": "string" },
                                                "author": { "type": "string" },
                                                "isbn": { "type": "string", "nullable": true },
                                                "pages": { "type": "integer", "nullable": true },
                                                "language": { "type": "string" },
                                                "created_at": { "type": "string", "format": "date-time" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                },
                "post": {
                    "summary": "Create a new book",
                    "description": "Add a new book to your personal library",
                    "tags": ["Books"],
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "required": ["title", "author"],
                                    "properties": {
                                        "title": { "type": "string", "description": "Book title" },
                                        "author": { "type": "string", "description": "Author name" },
                                        "isbn": { "type": "string", "nullable": true },
                                        "pages": { "type": "integer", "nullable": true },
                                        "language": { "type": "string", "default": "en" }
                                    }
                                }
                            }
                        }
                    },
                    "responses": {
                        "201": {
                            "description": "Book created successfully",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "id": { "type": "integer" },
                                            "title": { "type": "string" },
                                            "author": { "type": "string" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/api/books/{id}": {
                "get": {
                    "summary": "Get a book by ID",
                    "description": "Retrieve details of a specific book",
                    "tags": ["Books"],
                    "parameters": [
                        {
                            "name": "id",
                            "in": "path",
                            "required": true,
                            "schema": { "type": "integer" }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Book details",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "id": { "type": "integer" },
                                            "title": { "type": "string" },
                                            "author": { "type": "string" },
                                            "isbn": { "type": "string" },
                                            "pages": { "type": "integer" },
                                            "language": { "type": "string" },
                                            "created_at": { "type": "string", "format": "date-time" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/api/readings": {
                "get": {
                    "summary": "List all reading sessions",
                    "description": "Retrieve all your reading sessions with dates and progress",
                    "tags": ["Readings"],
                    "parameters": [
                        {
                            "name": "page",
                            "in": "query",
                            "schema": { "type": "integer", "default": 1 }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "List of reading sessions",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "array",
                                        "items": {
                                            "type": "object",
                                            "properties": {
                                                "id": { "type": "integer" },
                                                "book_id": { "type": "integer" },
                                                "book_title": { "type": "string" },
                                                "start_date": { "type": "string", "format": "date", "nullable": true },
                                                "end_date": { "type": "string", "format": "date", "nullable": true },
                                                "status": { "type": "string", "enum": ["reading", "completed", "paused"] }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/api/readings/stats": {
                "get": {
                    "summary": "Get reading statistics",
                    "description": "Get your reading statistics including books completed, total pages, current streak",
                    "tags": ["Readings"],
                    "responses": {
                        "200": {
                            "description": "Reading statistics",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "total_books": { "type": "integer" },
                                            "completed_books": { "type": "integer" },
                                            "total_pages": { "type": "integer" },
                                            "average_pages_per_book": { "type": "number" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/api/connectors": {
                "get": {
                    "summary": "List your AI connectors",
                    "description": "Get all configured AI provider connectors (Anthropic, Gemini, ChatGPT)",
                    "tags": ["Connectors"],
                    "responses": {
                        "200": {
                            "description": "List of connectors",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "array",
                                        "items": {
                                            "type": "object",
                                            "properties": {
                                                "id": { "type": "integer" },
                                                "provider": { "type": "string", "enum": ["anthropic", "gemini", "chatgpt"] },
                                                "is_active": { "type": "boolean" },
                                                "last_used_at": { "type": "string", "format": "date-time", "nullable": true },
                                                "created_at": { "type": "string", "format": "date-time" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    (StatusCode::OK, Json(schema))
}
