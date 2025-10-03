use anyhow::Result;
use axum::{
    extract::State,
    http::{HeaderValue, Method, StatusCode},
    response::Json,
    routing::{get, post},
    Router,
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing::{error, info};

mod mcp_client;
use mcp_client::McpClient;

#[derive(Parser)]
#[command(name = "mcp-http-bridge")]
#[command(about = "HTTP bridge for MCP server")]
struct Cli {
    #[arg(long, default_value = "3001")]
    port: u16,
    
    #[arg(long, default_value = "info")]
    log_level: String,
    
    #[arg(long, value_name = "MCP_SERVER_URL", default_value = "http://mcp-server:3002")]
    mcp_server_path: String,
}

#[derive(Clone)]
struct AppState {
    mcp_client: Arc<McpClient>,
}

// API Types
#[derive(Debug, Deserialize)]
struct ToolCallRequest {
    tool_name: String,
    arguments: serde_json::Map<String, Value>,
}

#[derive(Debug, Serialize)]
struct ToolCallResponse {
    success: bool,
    content: Option<Vec<ContentBlock>>,
    error: Option<String>,
}

#[derive(Debug, Serialize)]
struct ToolListResponse {
    tools: Vec<ToolInfo>,
}

#[derive(Debug, Serialize)]
struct ToolInfo {
    name: String,
    description: String,
    input_schema: Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(&cli.log_level)
        .init();

    info!("Starting MCP HTTP Bridge v{}", env!("CARGO_PKG_VERSION"));
    
    // Initialize MCP client
    let mcp_client = Arc::new(McpClient::new(&cli.mcp_server_path));
    
    // Initialize the MCP server
    match mcp_client.initialize().await {
        Ok(_) => info!("MCP server initialized successfully"),
        Err(e) => {
            error!("Failed to initialize MCP server: {}", e);
            return Err(e);
        }
    }
    
    let state = AppState { mcp_client };
    
    // Setup CORS
    let cors = CorsLayer::new()
        .allow_origin("*".parse::<HeaderValue>()?)
        .allow_methods([Method::GET, Method::POST])
        .allow_headers(tower_http::cors::Any);
    
    // Build our application with routes
    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/tools", get(list_tools_handler))
        .route("/tools/call", post(call_tool_handler))
        .layer(cors)
        .with_state(state);
    
    // Run the server
    let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{}", cli.port)).await?;
    info!("MCP HTTP Bridge listening on port {}", cli.port);
    
    axum::serve(listener, app).await?;
    
    Ok(())
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