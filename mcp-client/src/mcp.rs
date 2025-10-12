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

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{
        matchers::{method, path, body_json},
        Mock, MockServer, ResponseTemplate,
    };
    use serde_json::json;
    use assert_json_diff::assert_json_eq;

    #[tokio::test]
    async fn test_mcp_client_new() {
        let client = McpClient::new("http://localhost:3001");
        assert_eq!(client.base_url, "http://localhost:3001");
    }

    #[tokio::test]
    async fn test_list_tools_success() {
        let mock_server = MockServer::start().await;
        
        let mock_response = json!({
            "tools": [
                {
                    "name": "system_info",
                    "description": "Get system information",
                    "input_schema": {
                        "type": "object",
                        "properties": {
                            "detailed": {"type": "boolean"}
                        }
                    }
                },
                {
                    "name": "file_read",
                    "description": "Read file contents",
                    "input_schema": {
                        "type": "object",
                        "properties": {
                            "path": {"type": "string"}
                        },
                        "required": ["path"]
                    }
                }
            ]
        });

        Mock::given(method("GET"))
            .and(path("/tools"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
            .mount(&mock_server)
            .await;

        let client = McpClient::new(&mock_server.uri());
        let tools = client.list_tools().await.unwrap();

        assert_eq!(tools.len(), 2);
        assert_eq!(tools[0].name, "system_info");
        assert_eq!(tools[0].description, "Get system information");
        assert_eq!(tools[1].name, "file_read");
        assert_eq!(tools[1].description, "Read file contents");
        
        // Verify input schemas
        assert!(tools[0].input_schema["properties"]["detailed"].is_object());
        assert!(tools[1].input_schema["required"].is_array());
    }

    #[tokio::test]
    async fn test_list_tools_empty_response() {
        let mock_server = MockServer::start().await;
        
        let mock_response = json!({
            "tools": []
        });

        Mock::given(method("GET"))
            .and(path("/tools"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
            .mount(&mock_server)
            .await;

        let client = McpClient::new(&mock_server.uri());
        let tools = client.list_tools().await.unwrap();

        assert_eq!(tools.len(), 0);
    }

    #[tokio::test]
    async fn test_list_tools_server_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/tools"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal server error"))
            .mount(&mock_server)
            .await;

        let client = McpClient::new(&mock_server.uri());
        let result = client.list_tools().await;

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("500"));
        assert!(error_msg.contains("Internal server error"));
    }

    #[tokio::test]
    async fn test_list_tools_network_error() {
        // Use an invalid URL to simulate network error
        let client = McpClient::new("http://localhost:99999");
        let result = client.list_tools().await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_tools_invalid_json() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/tools"))
            .respond_with(ResponseTemplate::new(200).set_body_string("invalid json"))
            .mount(&mock_server)
            .await;

        let client = McpClient::new(&mock_server.uri());
        let result = client.list_tools().await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_call_tool_success() {
        let mock_server = MockServer::start().await;
        
        let expected_request = json!({
            "tool_name": "system_info",
            "arguments": {
                "detailed": true
            }
        });

        let mock_response = json!({
            "success": true,
            "content": [
                {
                    "type": "text",
                    "text": "System: Ubuntu 22.04, CPU: 8 cores"
                }
            ],
            "error": null
        });

        Mock::given(method("POST"))
            .and(path("/tools/call"))
            .and(body_json(&expected_request))
            .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
            .mount(&mock_server)
            .await;

        let client = McpClient::new(&mock_server.uri());
        let mut args = serde_json::Map::new();
        args.insert("detailed".to_string(), json!(true));
        
        let content = client.call_tool("system_info", args).await.unwrap();

        assert_eq!(content.len(), 1);
        match &content[0] {
            ContentBlock::Text { text } => {
                assert_eq!(text, "System: Ubuntu 22.04, CPU: 8 cores");
            }
        }
    }

    #[tokio::test]
    async fn test_call_tool_success_multiple_content_blocks() {
        let mock_server = MockServer::start().await;
        
        let expected_request = json!({
            "tool_name": "multi_output",
            "arguments": {}
        });

        let mock_response = json!({
            "success": true,
            "content": [
                {
                    "type": "text",
                    "text": "First output"
                },
                {
                    "type": "text", 
                    "text": "Second output"
                }
            ],
            "error": null
        });

        Mock::given(method("POST"))
            .and(path("/tools/call"))
            .and(body_json(&expected_request))
            .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
            .mount(&mock_server)
            .await;

        let client = McpClient::new(&mock_server.uri());
        let args = serde_json::Map::new();
        
        let content = client.call_tool("multi_output", args).await.unwrap();

        assert_eq!(content.len(), 2);
        match &content[0] {
            ContentBlock::Text { text } => assert_eq!(text, "First output"),
        }
        match &content[1] {
            ContentBlock::Text { text } => assert_eq!(text, "Second output"),
        }
    }

    #[tokio::test]
    async fn test_call_tool_success_empty_content() {
        let mock_server = MockServer::start().await;
        
        let expected_request = json!({
            "tool_name": "empty_tool",
            "arguments": {}
        });

        let mock_response = json!({
            "success": true,
            "content": [],
            "error": null
        });

        Mock::given(method("POST"))
            .and(path("/tools/call"))
            .and(body_json(&expected_request))
            .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
            .mount(&mock_server)
            .await;

        let client = McpClient::new(&mock_server.uri());
        let args = serde_json::Map::new();
        
        let content = client.call_tool("empty_tool", args).await.unwrap();

        assert_eq!(content.len(), 0);
    }

    #[tokio::test]
    async fn test_call_tool_success_null_content() {
        let mock_server = MockServer::start().await;
        
        let expected_request = json!({
            "tool_name": "null_content",
            "arguments": {}
        });

        let mock_response = json!({
            "success": true,
            "content": null,
            "error": null
        });

        Mock::given(method("POST"))
            .and(path("/tools/call"))
            .and(body_json(&expected_request))
            .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
            .mount(&mock_server)
            .await;

        let client = McpClient::new(&mock_server.uri());
        let args = serde_json::Map::new();
        
        let content = client.call_tool("null_content", args).await.unwrap();

        assert_eq!(content.len(), 0);
    }

    #[tokio::test]
    async fn test_call_tool_failure_with_error() {
        let mock_server = MockServer::start().await;
        
        let expected_request = json!({
            "tool_name": "failing_tool",
            "arguments": {}
        });

        let mock_response = json!({
            "success": false,
            "content": null,
            "error": "Tool execution failed: Permission denied"
        });

        Mock::given(method("POST"))
            .and(path("/tools/call"))
            .and(body_json(&expected_request))
            .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
            .mount(&mock_server)
            .await;

        let client = McpClient::new(&mock_server.uri());
        let args = serde_json::Map::new();
        
        let result = client.call_tool("failing_tool", args).await;

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Tool call failed"));
        assert!(error_msg.contains("Permission denied"));
    }

    #[tokio::test]
    async fn test_call_tool_failure_no_error_message() {
        let mock_server = MockServer::start().await;
        
        let expected_request = json!({
            "tool_name": "failing_tool",
            "arguments": {}
        });

        let mock_response = json!({
            "success": false,
            "content": null,
            "error": null
        });

        Mock::given(method("POST"))
            .and(path("/tools/call"))
            .and(body_json(&expected_request))
            .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
            .mount(&mock_server)
            .await;

        let client = McpClient::new(&mock_server.uri());
        let args = serde_json::Map::new();
        
        let result = client.call_tool("failing_tool", args).await;

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Unknown error"));
    }

    #[tokio::test]
    async fn test_call_tool_http_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/tools/call"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal server error"))
            .mount(&mock_server)
            .await;

        let client = McpClient::new(&mock_server.uri());
        let args = serde_json::Map::new();
        
        let result = client.call_tool("test_tool", args).await;

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("500"));
        assert!(error_msg.contains("Internal server error"));
    }

    #[tokio::test]
    async fn test_call_tool_network_error() {
        let client = McpClient::new("http://localhost:99999");
        let args = serde_json::Map::new();
        
        let result = client.call_tool("test_tool", args).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_call_tool_invalid_json_response() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/tools/call"))
            .respond_with(ResponseTemplate::new(200).set_body_string("invalid json"))
            .mount(&mock_server)
            .await;

        let client = McpClient::new(&mock_server.uri());
        let args = serde_json::Map::new();
        
        let result = client.call_tool("test_tool", args).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_call_tool_complex_arguments() {
        let mock_server = MockServer::start().await;
        
        let mut complex_args = serde_json::Map::new();
        complex_args.insert("file_path".to_string(), json!("/home/user/test.txt"));
        complex_args.insert("options".to_string(), json!({
            "encoding": "utf-8",
            "buffer_size": 1024,
            "metadata": {
                "include_timestamps": true,
                "format": "json"
            }
        }));
        complex_args.insert("flags".to_string(), json!(["read", "write", "create"]));

        let expected_request = json!({
            "tool_name": "file_processor",
            "arguments": complex_args
        });

        let mock_response = json!({
            "success": true,
            "content": [
                {
                    "type": "text",
                    "text": "File processed successfully"
                }
            ],
            "error": null
        });

        Mock::given(method("POST"))
            .and(path("/tools/call"))
            .and(body_json(&expected_request))
            .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
            .mount(&mock_server)
            .await;

        let client = McpClient::new(&mock_server.uri());
        
        let content = client.call_tool("file_processor", complex_args).await.unwrap();

        assert_eq!(content.len(), 1);
        match &content[0] {
            ContentBlock::Text { text } => {
                assert_eq!(text, "File processed successfully");
            }
        }
    }

    #[tokio::test]
    async fn test_tool_definition_deserialization() {
        let json_data = json!({
            "name": "test_tool",
            "description": "A test tool for demonstration",
            "input_schema": {
                "type": "object",
                "properties": {
                    "param1": {"type": "string"},
                    "param2": {"type": "integer", "minimum": 0}
                },
                "required": ["param1"]
            }
        });

        let tool: ToolDefinition = serde_json::from_value(json_data).unwrap();
        
        assert_eq!(tool.name, "test_tool");
        assert_eq!(tool.description, "A test tool for demonstration");
        assert_eq!(tool.input_schema["type"], "object");
        assert!(tool.input_schema["properties"]["param1"].is_object());
        assert!(tool.input_schema["required"].is_array());
    }

    #[tokio::test]
    async fn test_content_block_deserialization() {
        let json_data = json!({
            "type": "text",
            "text": "This is a text content block"
        });

        let content_block: ContentBlock = serde_json::from_value(json_data).unwrap();
        
        match content_block {
            ContentBlock::Text { text } => {
                assert_eq!(text, "This is a text content block");
            }
        }
    }

    #[tokio::test]
    async fn test_content_block_serialization() {
        let content_block = ContentBlock::Text {
            text: "Test message".to_string(),
        };

        let json_value = serde_json::to_value(&content_block).unwrap();
        let expected = json!({
            "type": "text",
            "text": "Test message"
        });

        assert_json_eq!(json_value, expected);
    }
}