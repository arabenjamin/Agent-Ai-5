use axum_test::TestServer;
use std::sync::Arc;

/// Create a test server for integration testing
pub async fn create_test_server() -> TestServer {
    // Create a mock MCP client for testing
    let mcp_client = Arc::new(mcp_http_bridge::McpClient::new("http://mock-server:3002"));
    let state = mcp_http_bridge::AppState { mcp_client };
    let app = mcp_http_bridge::create_app_with_state(state);
    
    TestServer::new(app).unwrap()
}

/// Create a test server with a specific MCP server URL
pub async fn create_test_server_with_url(mcp_url: &str) -> TestServer {
    let mcp_client = Arc::new(mcp_http_bridge::McpClient::new(mcp_url));
    let state = mcp_http_bridge::AppState { mcp_client };
    let app = mcp_http_bridge::create_app_with_state(state);
    
    TestServer::new(app).unwrap()
}