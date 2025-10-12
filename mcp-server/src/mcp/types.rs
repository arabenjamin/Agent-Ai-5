use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeParams {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: ClientCapabilities,
    #[serde(rename = "clientInfo")]
    pub client_info: ClientInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolCapabilities>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    pub capabilities: Capabilities,
    #[serde(rename = "serverInfo")]
    pub server_info: ServerInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Capabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<ToolCapabilities>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCapabilities {
    #[serde(rename = "listChanged", skip_serializing_if = "Option::is_none")]
    pub list_changed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolsListResult {
    pub tools: Vec<ToolDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallParams {
    pub name: String,
    #[serde(default)]
    pub arguments: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    pub content: Vec<ContentBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
}

impl ContentBlock {
    pub fn text(content: &str) -> Self {
        Self::Text {
            text: content.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_json_rpc_request_serialization() {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "tools/list".to_string(),
            params: Some(json!({})),
        };

        let serialized = serde_json::to_string(&request).unwrap();
        let expected = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#;
        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_json_rpc_request_deserialization() {
        let json_str = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}"#;
        let request: JsonRpcRequest = serde_json::from_str(json_str).unwrap();
        
        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.id, Some(json!(1)));
        assert_eq!(request.method, "tools/list");
        assert!(request.params.is_some());
    }

    #[test]
    fn test_json_rpc_request_without_params() {
        let json_str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize"}"#;
        let request: JsonRpcRequest = serde_json::from_str(json_str).unwrap();
        
        assert_eq!(request.jsonrpc, "2.0");
        assert_eq!(request.id, Some(json!(1)));
        assert_eq!(request.method, "initialize");
        assert!(request.params.is_none());
    }

    #[test]
    fn test_json_rpc_response_success() {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            result: Some(json!({"success": true})),
            error: None,
        };

        let serialized = serde_json::to_string(&response).unwrap();
        let expected = r#"{"jsonrpc":"2.0","id":1,"result":{"success":true}}"#;
        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_json_rpc_response_error() {
        let error = JsonRpcError {
            code: -32600,
            message: "Invalid Request".to_string(),
            data: Some(json!({"details": "Missing required field"})),
        };

        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            result: None,
            error: Some(error),
        };

        let serialized = serde_json::to_string(&response).unwrap();
        assert!(serialized.contains("Invalid Request"));
        assert!(serialized.contains("-32600"));
    }

    #[test]
    fn test_initialize_params() {
        let params = InitializeParams {
            protocol_version: "2024-11-05".to_string(),
            capabilities: ClientCapabilities {
                tools: Some(ToolCapabilities {
                    list_changed: Some(true),
                }),
            },
            client_info: ClientInfo {
                name: "test-client".to_string(),
                version: "1.0.0".to_string(),
            },
        };

        let serialized = serde_json::to_string(&params).unwrap();
        assert!(serialized.contains("protocolVersion"));
        assert!(serialized.contains("clientInfo"));
        assert!(serialized.contains("test-client"));
    }

    #[test]
    fn test_initialize_result() {
        let result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: Capabilities {
                tools: Some(ToolCapabilities {
                    list_changed: Some(false),
                }),
            },
            server_info: ServerInfo {
                name: "mcp-server".to_string(),
                version: "0.1.0".to_string(),
            },
        };

        let serialized = serde_json::to_string(&result).unwrap();
        assert!(serialized.contains("protocolVersion"));
        assert!(serialized.contains("serverInfo"));
        assert!(serialized.contains("mcp-server"));
    }

    #[test]
    fn test_tool_definition() {
        let tool = ToolDefinition {
            name: "test_tool".to_string(),
            description: "A test tool".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "param1": {"type": "string"}
                }
            }),
        };

        let serialized = serde_json::to_string(&tool).unwrap();
        assert!(serialized.contains("inputSchema"));
        assert!(serialized.contains("test_tool"));
        assert!(serialized.contains("A test tool"));
    }

    #[test]
    fn test_tools_list_result() {
        let tools = vec![
            ToolDefinition {
                name: "tool1".to_string(),
                description: "First tool".to_string(),
                input_schema: json!({"type": "object"}),
            },
            ToolDefinition {
                name: "tool2".to_string(),
                description: "Second tool".to_string(),
                input_schema: json!({"type": "object"}),
            },
        ];

        let result = ToolsListResult { tools };
        let serialized = serde_json::to_string(&result).unwrap();
        assert!(serialized.contains("tool1"));
        assert!(serialized.contains("tool2"));
    }

    #[test]
    fn test_tool_call_params() {
        let mut arguments = HashMap::new();
        arguments.insert("param1".to_string(), json!("value1"));
        arguments.insert("param2".to_string(), json!(42));

        let params = ToolCallParams {
            name: "test_tool".to_string(),
            arguments,
        };

        let serialized = serde_json::to_string(&params).unwrap();
        assert!(serialized.contains("test_tool"));
        assert!(serialized.contains("value1"));
        assert!(serialized.contains("42"));
    }

    #[test]
    fn test_tool_call_params_empty_arguments() {
        let params = ToolCallParams {
            name: "simple_tool".to_string(),
            arguments: HashMap::new(),
        };

        let serialized = serde_json::to_string(&params).unwrap();
        assert!(serialized.contains("simple_tool"));
        assert!(serialized.contains("arguments"));
    }

    #[test]
    fn test_content_block_text() {
        let block = ContentBlock::text("Hello, world!");
        
        match block {
            ContentBlock::Text { text } => assert_eq!(text, "Hello, world!"),
        }
    }

    #[test]
    fn test_content_block_serialization() {
        let block = ContentBlock::Text {
            text: "Test content".to_string(),
        };

        let serialized = serde_json::to_string(&block).unwrap();
        let expected = r#"{"type":"text","text":"Test content"}"#;
        assert_eq!(serialized, expected);
    }

    #[test]
    fn test_content_block_deserialization() {
        let json_str = r#"{"type":"text","text":"Deserialized content"}"#;
        let block: ContentBlock = serde_json::from_str(json_str).unwrap();
        
        match block {
            ContentBlock::Text { text } => assert_eq!(text, "Deserialized content"),
        }
    }

    #[test]
    fn test_tool_call_result() {
        let result = ToolCallResult {
            content: vec![
                ContentBlock::text("First result"),
                ContentBlock::text("Second result"),
            ],
        };

        let serialized = serde_json::to_string(&result).unwrap();
        assert!(serialized.contains("First result"));
        assert!(serialized.contains("Second result"));
    }

    #[test]
    fn test_capabilities_default() {
        let caps = Capabilities::default();
        assert!(caps.tools.is_none());
        
        let serialized = serde_json::to_string(&caps).unwrap();
        assert_eq!(serialized, "{}");
    }

    #[test]
    fn test_capabilities_with_tools() {
        let caps = Capabilities {
            tools: Some(ToolCapabilities {
                list_changed: Some(true),
            }),
        };

        let serialized = serde_json::to_string(&caps).unwrap();
        assert!(serialized.contains("tools"));
        assert!(serialized.contains("listChanged"));
    }

    #[test]
    fn test_json_rpc_error_codes() {
        // Test standard JSON-RPC error codes
        let parse_error = JsonRpcError {
            code: -32700,
            message: "Parse error".to_string(),
            data: None,
        };

        let invalid_request = JsonRpcError {
            code: -32600,
            message: "Invalid Request".to_string(),
            data: None,
        };

        let method_not_found = JsonRpcError {
            code: -32601,
            message: "Method not found".to_string(),
            data: None,
        };

        assert_eq!(parse_error.code, -32700);
        assert_eq!(invalid_request.code, -32600);
        assert_eq!(method_not_found.code, -32601);
    }

    #[test]
    fn test_round_trip_serialization() {
        // Test that we can serialize and deserialize without loss
        let original_request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!("test-id")),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "test_tool",
                "arguments": {
                    "param1": "value1",
                    "param2": 42
                }
            })),
        };

        let serialized = serde_json::to_string(&original_request).unwrap();
        let deserialized: JsonRpcRequest = serde_json::from_str(&serialized).unwrap();

        assert_eq!(original_request.jsonrpc, deserialized.jsonrpc);
        assert_eq!(original_request.id, deserialized.id);
        assert_eq!(original_request.method, deserialized.method);
        assert_eq!(original_request.params, deserialized.params);
    }
}