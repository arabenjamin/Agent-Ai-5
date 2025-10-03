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
        let url = match request.method.as_str() {
            "tools/list" => format!("{}/tools/list", &self.mcp_server_path),
            "tools/call" => format!("{}/tools/call", &self.mcp_server_path),
            _ => return Err(anyhow!("Unknown method: {}", request.method)),
        };
        debug!("Making request to {}", url);
        
        let response = match request.method.as_str() {
            "tools/list" => client.get(&url).send().await?,
            "tools/call" => client.post(&url).json(&request.params).send().await?,
            _ => return Err(anyhow!("Unknown method: {}", request.method)),
        };
            
        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            error!("MCP server returned error status: {} with body: {}", status, error_text);
            return Err(anyhow!(
                "MCP server returned error status: {} with body: {}",
                status,
                error_text
            ));
        }
        
        let response_text = response.text().await?;
        debug!("Received response: {}", response_text);
        
        // Parse the raw response into a Value first
        let response_value: serde_json::Value = serde_json::from_str(&response_text)?;
        
        // Convert the response into our expected JsonRpcResponse format
        let jsonrpc_response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: request.id,
            result: Some(response_value),
            error: None,
        };
        
        Ok(jsonrpc_response)
    }

    pub async fn initialize(&self) -> Result<()> {
        // Verify the server is up by fetching tools list
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
                
                // If tools is another object with a tools field
                if let Some(nested_tools) = tools_obj.as_object().and_then(|obj| obj.get("tools")) {
                    if let Ok(tools) = serde_json::from_value::<Vec<ToolDefinition>>(nested_tools.clone()) {
                        return Ok(tools);
                    }
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
            let call_result: serde_json::Map<String, Value> = serde_json::from_value(result)?;
            if let Some(content_array) = call_result.get("content") {
                let content: Vec<ContentBlock> = serde_json::from_value(content_array.clone())?;
                return Ok(content);
            }
        }
        
        Err(anyhow!("Invalid tools/call response format"))
    }
}