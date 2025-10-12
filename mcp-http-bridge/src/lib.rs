pub mod mcp_client;
pub mod openapi;

pub use mcp_client::McpClient;

use anyhow::Result;
use axum::{
    extract::State,
    http::{HeaderValue, Method, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::{error, info};
use utoipa::ToSchema;

use openapi::openapi_handler;

#[derive(Clone)]
pub struct AppState {
    pub mcp_client: Arc<McpClient>,
}

// API Types
/// Request to call a specific tool
#[derive(Debug, Deserialize, ToSchema)]
pub struct ToolCallRequest {
    /// Name of the tool to call
    pub tool_name: String,
    /// Arguments to pass to the tool
    pub arguments: serde_json::Map<String, Value>,
}

/// Response from a tool call
#[derive(Debug, Serialize, ToSchema)]
pub struct ToolCallResponse {
    /// Whether the tool call was successful
    pub success: bool,
    /// Content returned by the tool (if successful)
    pub content: Option<Vec<ContentBlock>>,
    /// Error message (if unsuccessful)
    pub error: Option<String>,
}

/// List of available tools
#[derive(Debug, Serialize, ToSchema)]
pub struct ToolListResponse {
    /// Array of available tools
    pub tools: Vec<ToolInfo>,
}

/// Information about a tool
#[derive(Debug, Serialize, ToSchema)]
pub struct ToolInfo {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// JSON schema for tool input
    pub input_schema: Value,
}

/// Content block returned by tools
#[derive(Debug, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type")]
pub enum ContentBlock {
    /// Text content
    #[serde(rename = "text")]
    Text { 
        /// The text content
        text: String 
    },
}

/// Health check response
#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    /// Service status
    pub status: String,
    /// Service version
    pub version: String,
}

/// Create the application router with the given state
pub fn create_app_with_state(state: AppState) -> Router {
    // Setup CORS
    let cors = CorsLayer::new()
        .allow_origin("*".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(tower_http::cors::Any);
    
    // Build our application with routes
    Router::new()
        .route("/health", get(health_handler))
        .route("/tools", get(list_tools_handler))
        .route("/tools/call", post(call_tool_handler))
        .route("/openapi.json", get(openapi_handler))
        .layer(cors)
        .with_state(state)
}

/// Create the application router for testing (without real MCP client)
#[cfg(test)]
pub fn create_app() -> Router {
    // Create a mock MCP client for testing
    let mcp_client = Arc::new(McpClient::new("http://mock-server:3002"));
    let state = AppState { mcp_client };
    create_app_with_state(state)
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn list_tools_handler(State(state): State<AppState>) -> Result<Json<ToolListResponse>, StatusCode> {
    match state.mcp_client.list_tools().await {
        Ok(tools) => {
            let tool_infos = tools.into_iter().map(|tool| ToolInfo {
                name: tool.name,
                description: tool.description,
                input_schema: tool.input_schema,
            }).collect();
            
            info!("Successfully listed tools");
            Ok(Json(ToolListResponse { tools: tool_infos }))
        }
        Err(e) => {
            error!("Failed to list tools: {:#}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn call_tool_handler(
    State(state): State<AppState>, 
    Json(request): Json<ToolCallRequest>
) -> Result<Json<ToolCallResponse>, StatusCode> {
    
    info!("Calling tool: {} with args: {:?}", request.tool_name, request.arguments);
    info!("Converting request to JSON-RPC call with params: {}", serde_json::json!({
        "name": request.tool_name,
        "arguments": request.arguments
    }));
    
    match state.mcp_client.call_tool(&request.tool_name, request.arguments).await {
        Ok(content) => {
            Ok(Json(ToolCallResponse {
                success: true,
                content: Some(content),
                error: None,
            }))
        }
        Err(e) => {
            error!("Tool call failed: {}", e);
            Ok(Json(ToolCallResponse {
                success: false,
                content: None,
                error: Some(e.to_string()),
            }))
        }
    }
}

#[cfg(test)]
mod tests;