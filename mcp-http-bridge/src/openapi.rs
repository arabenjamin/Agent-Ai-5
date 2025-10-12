use axum::Json;
use serde_json::{json, Value};
use utoipa::{OpenApi, ToSchema};

use crate::{ContentBlock, HealthResponse, ToolCallRequest, ToolCallResponse, ToolInfo, ToolListResponse};

#[derive(OpenApi)]
#[openapi(
    paths(
        openapi_handler
    ),
    components(
        schemas(
            HealthResponse,
            ToolListResponse,
            ToolInfo,
            ToolCallRequest,
            ToolCallResponse,
            ContentBlock,
            ApiError
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "tools", description = "MCP tool management and execution"),
        (name = "documentation", description = "API documentation endpoints")
    ),
    info(
        title = "MCP HTTP Bridge API",
        version = "0.1.0",
        description = "HTTP bridge for Model Context Protocol (MCP) server communication",
        contact(
            name = "MCP HTTP Bridge",
            url = "https://github.com/arabenjamin/Agent-Ai-5"
        )
    ),
    servers(
        (url = "http://localhost:3001", description = "Local development server"),
        (url = "http://mcp-http-bridge:3001", description = "Docker container")
    )
)]
pub struct ApiDoc;

/// Error response
#[derive(ToSchema)]
pub struct ApiError {
    /// Error message
    pub error: String,
    /// HTTP status code
    pub status: u16,
}

/// Get OpenAPI specification
#[utoipa::path(
    get,
    path = "/openapi.json",
    tag = "documentation",
    responses(
        (status = 200, description = "OpenAPI specification", content_type = "application/json")
    )
)]
pub async fn openapi_handler() -> Json<Value> {
    // Create a comprehensive OpenAPI spec manually to ensure all endpoints are documented
    let spec = json!({
        "openapi": "3.0.3",
        "info": {
            "title": "MCP HTTP Bridge API",
            "version": "0.1.0",
            "description": "HTTP bridge for Model Context Protocol (MCP) server communication",
            "contact": {
                "name": "MCP HTTP Bridge",
                "url": "https://github.com/arabenjamin/Agent-Ai-5"
            }
        },
        "servers": [
            {
                "url": "http://localhost:3001",
                "description": "Local development server"
            },
            {
                "url": "http://mcp-http-bridge:3001",
                "description": "Docker container"
            }
        ],
        "paths": {
            "/health": {
                "get": {
                    "tags": ["health"],
                    "summary": "Health check",
                    "description": "Returns the health status and version of the service",
                    "responses": {
                        "200": {
                            "description": "Service is healthy",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/HealthResponse"
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/tools": {
                "get": {
                    "tags": ["tools"],
                    "summary": "List tools",
                    "description": "Returns a list of all available MCP tools with their descriptions and input schemas",
                    "responses": {
                        "200": {
                            "description": "List of available tools",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/ToolListResponse"
                                    }
                                }
                            }
                        },
                        "500": {
                            "description": "Internal server error"
                        }
                    }
                }
            },
            "/tools/call": {
                "post": {
                    "tags": ["tools"],
                    "summary": "Call tool",
                    "description": "Execute a specific MCP tool with the provided arguments",
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "$ref": "#/components/schemas/ToolCallRequest"
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Tool execution result",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/ToolCallResponse"
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/openapi.json": {
                "get": {
                    "tags": ["documentation"],
                    "summary": "Get OpenAPI specification",
                    "description": "Returns the OpenAPI 3.0 specification for this API in JSON format",
                    "responses": {
                        "200": {
                            "description": "OpenAPI specification",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        "components": {
            "schemas": {
                "HealthResponse": {
                    "type": "object",
                    "required": ["status", "version"],
                    "properties": {
                        "status": {
                            "type": "string",
                            "description": "Service status",
                            "example": "healthy"
                        },
                        "version": {
                            "type": "string",
                            "description": "Service version",
                            "example": "0.1.0"
                        }
                    }
                },
                "ToolListResponse": {
                    "type": "object",
                    "required": ["tools"],
                    "properties": {
                        "tools": {
                            "type": "array",
                            "description": "Array of available tools",
                            "items": {
                                "$ref": "#/components/schemas/ToolInfo"
                            }
                        }
                    }
                },
                "ToolInfo": {
                    "type": "object",
                    "required": ["name", "description", "input_schema"],
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Tool name",
                            "example": "system_info"
                        },
                        "description": {
                            "type": "string",
                            "description": "Tool description",
                            "example": "Get system information"
                        },
                        "input_schema": {
                            "type": "object",
                            "description": "JSON schema for tool input"
                        }
                    }
                },
                "ToolCallRequest": {
                    "type": "object",
                    "required": ["tool_name", "arguments"],
                    "properties": {
                        "tool_name": {
                            "type": "string",
                            "description": "Name of the tool to call",
                            "example": "system_info"
                        },
                        "arguments": {
                            "type": "object",
                            "description": "Arguments to pass to the tool",
                            "additionalProperties": true
                        }
                    }
                },
                "ToolCallResponse": {
                    "type": "object",
                    "required": ["success"],
                    "properties": {
                        "success": {
                            "type": "boolean",
                            "description": "Whether the tool call was successful"
                        },
                        "content": {
                            "type": "array",
                            "description": "Content returned by the tool (if successful)",
                            "items": {
                                "$ref": "#/components/schemas/ContentBlock"
                            }
                        },
                        "error": {
                            "type": "string",
                            "description": "Error message (if unsuccessful)"
                        }
                    }
                },
                "ContentBlock": {
                    "type": "object",
                    "required": ["type"],
                    "properties": {
                        "type": {
                            "type": "string",
                            "enum": ["text"],
                            "description": "Content block type"
                        },
                        "text": {
                            "type": "string",
                            "description": "The text content"
                        }
                    }
                }
            }
        },
        "tags": [
            {
                "name": "health",
                "description": "Health check endpoints"
            },
            {
                "name": "tools",
                "description": "MCP tool management and execution"
            },
            {
                "name": "documentation",
                "description": "API documentation endpoints"
            }
        ]
    });
    
    Json(spec)
}