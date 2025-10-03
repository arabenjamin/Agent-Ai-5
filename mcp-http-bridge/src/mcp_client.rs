use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

use crate::ContentBlock;

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: i32,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

pub struct McpClient {
    mcp_server_path: String,
    request_id: Arc<Mutex<i32>>,
}

impl McpClient {
    pub fn new(mcp_server_path: &str) -> Self {
        Self {
            mcp_server_path: mcp_server_path.to_string(),
            request_id: Arc::new(Mutex::new(1)),
        }
    }

    async fn get_next_id(&self) -> i32 {
        let mut id = self.request_id.lock().await;
        let current = *id;
        *id += 1;
        current
    }

    async fn execute_mcp_command(&self, request: JsonRpcRequest) -> Result<JsonRpcResponse> {
        debug!("Executing MCP command: {} to {}", request.method, self.mcp_server_path);
        
        let client = reqwest::Client::new();
        let base_url = self.mcp_server_path.trim_end_matches('/').to_string();
        let url = if request.method == "tools/list" {
            format!("{}/tools/list", base_url)
        } else {
            format!("{}/tools/call", base_url)
        };
        debug!("Making request to {}", url);

        // Create proper JSON-RPC envelope
        let json_rpc = if request.method == "tools/list" {
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "method": request.method
            })
        } else {
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": request.id,
                "method": request.method,
                "params": request.params
            })
        };
        
        debug!("Sending JSON-RPC request: {}", json_rpc);
        
        let response = if request.method == "tools/list" {
            client.get(&url)
                .header("Content-Type", "application/json")
                .header("Accept", "application/json")
                .send()
                .await?
        } else {
            client.post(&url)
                .header("Content-Type", "application/json")
                .header("Accept", "application/json")
                .json(&json_rpc)
                .send()
                .await?
        };
            
        let status = response.status();
        debug!("Response status: {}", status);
        debug!("Response headers: {:?}", response.headers());
        
        let response_text = response.text().await?;
        debug!("Raw response from MCP server: {}", response_text);
        
        if !status.is_success() {
            error!("MCP server returned error status: {} with body: {}", status, response_text);
            return Err(anyhow!("MCP server error: {} - {}", status, response_text));
        }
        
        // For tools/list, try to parse the raw response first
        if request.method == "tools/list" {
            if let Ok(tools_response) = serde_json::from_str::<serde_json::Value>(&response_text) {
                debug!("Got raw tools response: {}", tools_response);
                if let Some(tools) = tools_response.get("tools") {
                    return Ok(JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: Some(tools.clone()),
                        error: None,
                    });
                }
            }
        }

        // Try to parse as JSON-RPC response
        serde_json::from_str(&response_text)
            .map_err(|e| {
                error!("Failed to parse JSON-RPC response: {} - Response text: {}", e, response_text);
                anyhow!("JSON-RPC parse error: {} - Response: {}", e, response_text)
            })
    }

    pub async fn initialize(&self) -> Result<()> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: self.get_next_id().await,
            method: "tools/list".to_string(),
            params: None,
        };

        match self.execute_mcp_command(request).await {
            Ok(_) => {
                info!("MCP server initialized successfully");
                Ok(())
            }
            Err(e) => {
                error!("Failed to initialize MCP server: {}", e);
                Err(e)
            }
        }
    }

    pub async fn list_tools(&self) -> Result<Vec<ToolDefinition>> {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: self.get_next_id().await,
            method: "tools/list".to_string(),
            params: None,
        };

        let response = self.execute_mcp_command(request).await?;
        
        if let Some(result) = response.result {
            debug!("Got tools list response: {}", serde_json::to_string_pretty(&result)?);
            
            // Try to parse as direct array first
            if let Ok(tools) = serde_json::from_value::<Vec<ToolDefinition>>(result.clone()) {
                return Ok(tools);
            }
            
            // Try to parse from result.tools field
            if let Some(tools_obj) = result.as_object().and_then(|obj| obj.get("tools")) {
                if let Ok(tools) = serde_json::from_value::<Vec<ToolDefinition>>(tools_obj.clone()) {
                    return Ok(tools);
                }
            }
            
            error!("Failed to parse tools list response: {}", result);
        }
        
        Err(anyhow!("Invalid tools/list response format"))
    }

    pub async fn call_tool(&self, tool_name: &str, arguments: serde_json::Map<String, Value>) -> Result<Vec<ContentBlock>> {
        let id = self.get_next_id().await;
        debug!("Making tool call request {} for tool {} with arguments {:?}", id, tool_name, arguments);
        
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id,
            method: "tools/call".to_string(),
            params: Some(serde_json::json!({
                "name": tool_name,
                "arguments": arguments
            })),
        };

        let response = self.execute_mcp_command(request).await?;
        
        if let Some(result) = response.result {
            debug!("Got result from MCP server: {:?}", result);
            
            // Try to parse from the result.content field
            if let Some(content_obj) = result.as_object().and_then(|obj| obj.get("content")) {
                match serde_json::from_value::<Vec<ContentBlock>>(content_obj.clone()) {
                    Ok(content) => {
                        debug!("Successfully parsed content blocks: {:?}", content);
                        return Ok(content);
                    }
                    Err(e) => {
                        error!("Failed to parse content blocks: {}", e);
                        error!("Raw result was: {:?}", result);
                        return Err(anyhow!("Invalid tools/call response format: {}", e));
                    }
                }
            }
            
            // Try to parse directly if no content field
            match serde_json::from_value::<Vec<ContentBlock>>(result.clone()) {
                Ok(content) => {
                    debug!("Successfully parsed content blocks directly: {:?}", content);
                    return Ok(content);
                }
                Err(e) => {
                    error!("Failed to parse content blocks directly: {}", e);
                    error!("Raw result was: {:?}", result);
                    return Err(anyhow!("Invalid tools/call response format: {}", e));
                }
            }
        }
        
        error!("No result field in response");
        Err(anyhow!("Invalid tools/call response format: no result field"))
    }
}
