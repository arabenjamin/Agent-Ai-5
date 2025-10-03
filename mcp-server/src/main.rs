use anyhow::Result;
use axum::{
    extract::{Json, State},
    http::{HeaderValue, Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use clap::Parser;
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};
use tower_http::cors::CorsLayer;
use tracing::{info, error};

mod mcp;
mod tools;
mod plugins;
mod context;

use mcp::McpServer;

#[derive(Parser)]
#[command(name = "mcp-server")]
#[command(about = "A Model Context Protocol (MCP) server")]
struct Cli {
    #[arg(long, default_value = "8080")]
    port: u16,
    
    #[arg(long, default_value = "debug")]
    log_level: String,
    
    #[arg(long)]
    stdio: bool,
    
    #[arg(long)]
    quiet: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Initialize tracing (only if not quiet)
    if !cli.quiet {
        tracing_subscriber::fmt()
            .with_env_filter(&cli.log_level)
            .init();
    }

    info!("Starting MCP Server v{}", env!("CARGO_PKG_VERSION"));

    // Test Neo4j connection at startup
    match context::get_neo4j_context().await {
        Ok(_ctx) => info!("Successfully connected to Neo4j"),
        Err(e) => error!("Failed to connect to Neo4j: {}", e),
    }
    
    let server = Arc::new(McpServer::new());
    server.initialize().await?;
    info!("MCP Server initialized successfully");
    
    if cli.stdio {
        run_stdio_mode(server).await?;
    } else {
        run_http_mode(server, cli.port).await?;
    }
    
    Ok(())
}

async fn run_stdio_mode(server: Arc<McpServer>) -> Result<()> {
    info!("Running in STDIO mode");
    
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut reader = BufReader::new(stdin);
    let mut line = String::new();
    
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => break, // EOF
            Ok(_) => {
                if let Ok(response) = server.handle_message(&line).await {
                    stdout.write_all(response.as_bytes()).await?;
                    stdout.write_all(b"\n").await?;
                    stdout.flush().await?;
                }
            }
            Err(e) => {
                error!("Error reading from stdin: {}", e);
                break;
            }
        }
    }
    
    Ok(())
}

async fn run_http_mode(server: Arc<McpServer>, port: u16) -> Result<()> {
    info!("Running in HTTP mode on port {}", port);
    
    let app = Router::new()
        .route("/version", get(|| async { "1.0.0" }))
        .route("/tools/list", get(get_tools))
        .route("/tools/call", post(tool_call))
        .with_state(server)
        .layer(
            CorsLayer::new()
                .allow_origin("*".parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET, Method::POST])
        );

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
    info!("Listening on {}", addr);
    
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    
    Ok(())
}

async fn get_tools(
    State(server): State<Arc<McpServer>>,
) -> impl IntoResponse {
    // Create a tools/list JSON-RPC request
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/list"
    });

    match server.handle_message(&request.to_string()).await {
        Ok(response) => {
            match serde_json::from_str::<serde_json::Value>(&response) {
                Ok(json) => {
                    if let Some(result) = json.as_object().and_then(|obj| obj.get("result")) {
                        // Return the tools array directly without nesting
                        Json(result.clone()).into_response()
                    } else {
                        Json(json).into_response()
                    }
                },
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to parse response: {}", e),
                ).into_response(),
            }
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to get tools: {}", e),
        ).into_response(),
    }
}

async fn tool_call(
    State(server): State<Arc<McpServer>>,
    Json(request): Json<serde_json::Value>,
) -> impl IntoResponse {
    match server.handle_message(&serde_json::to_string(&request).unwrap()).await {
        Ok(response) => {
            match serde_json::from_str::<serde_json::Value>(&response) {
                Ok(json) => Json(json).into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to parse response: {}", e),
                ).into_response(),
            }
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to handle tool call: {}", e),
        ).into_response(),
    }
}