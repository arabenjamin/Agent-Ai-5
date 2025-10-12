use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tracing::{error, info};

use mcp_http_bridge::{AppState, McpClient, create_app_with_state};

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
    
    let app = create_app_with_state(state);

    // Run the server
    let listener = tokio::net::TcpListener::bind(&format!("0.0.0.0:{}", cli.port)).await?;
    info!("MCP HTTP Bridge listening on port {}", cli.port);
    info!("OpenAPI documentation available at http://localhost:{}/openapi.json", cli.port);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}