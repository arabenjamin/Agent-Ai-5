use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
}

pub struct McpClient {
    base_url: String,
    client: reqwest::Client,
}

impl McpClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn list_tools(&self) -> Result<Vec<ToolDefinition>> {
        let response = self.client
            .get(&format!("{}/tools", self.base_url))
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!(
                "MCP server returned error status: {} with body: {}",
                status,
                error_text
            ));
        }

        #[derive(Deserialize)]
        struct ToolListResponse {
            tools: Vec<ToolDefinition>,
        }

        let response_data: ToolListResponse = response.json().await?;
        Ok(response_data.tools)
    }

    pub async fn call_tool(&self, tool_name: &str, arguments: serde_json::Map<String, Value>) -> Result<Vec<ContentBlock>> {
                #[derive(Serialize)]
        struct ToolCallRequest {
            tool_name: String,
            arguments: serde_json::Map<String, Value>,
        }

        let request = ToolCallRequest {
            tool_name: tool_name.to_string(),
            arguments,
        };

        let response = self.client
            .post(&format!("{}/tools/call", self.base_url))
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!(
                "MCP server returned error status: {} with body: {}",
                status,
                error_text
            ));
        }

        #[derive(Deserialize)]
        struct ToolCallResponse {
            success: bool,
            content: Option<Vec<ContentBlock>>,
            error: Option<String>,
        }

        let response_data: ToolCallResponse = response.json().await?;
        
        if !response_data.success {
            return Err(anyhow::anyhow!(
                "Tool call failed: {}",
                response_data.error.unwrap_or_else(|| "Unknown error".to_string())
            ));
        }

        Ok(response_data.content.unwrap_or_default())
    }
}