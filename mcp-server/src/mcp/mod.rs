use serde::Serialize;
use serde_json::Value;
use tracing::{debug, error, info};
use std::sync::Arc;
use std::collections::HashMap;

use crate::tools::{ToolRegistry, SystemInfoTool, HomeAssistantTool, HttpTool, Neo4jTool};
use crate::plugins::system_info::SystemInfoPlugin;
use crate::plugins::home_assistant::HomeAssistantPlugin;
use crate::plugins::http::HttpPlugin;

pub mod types;
pub mod plugin_registry;
pub mod plugin_params;
pub use types::*;
use plugin_registry::PluginRegistry;
use plugin_params::PluginCallParams;

use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::Mutex;

pub struct McpServer {
    tool_registry: Mutex<ToolRegistry>,
    plugin_registry: Mutex<PluginRegistry>,
    initialized: AtomicBool,
}

impl McpServer {
    pub fn new() -> Self {
        Self {
            tool_registry: Mutex::new(ToolRegistry::new()),
            plugin_registry: Mutex::new(PluginRegistry::new()),
            initialized: AtomicBool::new(false),
        }
    }

    pub async fn initialize(&self) -> anyhow::Result<()> {
        // Register built-in plugins
        let system_info = Arc::new(SystemInfoPlugin::new());
        let home_assistant = Arc::new(HomeAssistantPlugin::new());
        let http = Arc::new(HttpPlugin::new());
        
        // Initialize Neo4j plugin
        let neo4j = Arc::new(
            crate::plugins::neo4j::Neo4jPlugin::new(
                &std::env::var("NEO4J_URI").unwrap_or_else(|_| "bolt://neo4j:7687".to_string()),
                &std::env::var("NEO4J_USER").unwrap_or_else(|_| "neo4j".to_string()),
                &std::env::var("NEO4J_PASSWORD").expect("NEO4J_PASSWORD must be set")
            ).await.map_err(|e| anyhow::anyhow!("Failed to create Neo4j plugin: {}", e))?
        );
        
        // Register plugins
        let mut registry = self.plugin_registry.lock().await;
        registry.register_plugin(system_info.clone()).await?;
        registry.register_plugin(home_assistant.clone()).await?;
        registry.register_plugin(http.clone()).await?;
        registry.register_plugin(neo4j.clone()).await?;
        drop(registry);
        
        // Register tools for each plugin capability
        let mut tool_registry = self.tool_registry.lock().await;
        
        let system_info_tool = SystemInfoTool::new(system_info);
        tool_registry.register(Box::new(system_info_tool));
        
        let home_assistant_tool = HomeAssistantTool::new(home_assistant);
        tool_registry.register(Box::new(home_assistant_tool));
        
        let http_tool = HttpTool::new(http);
        tool_registry.register(Box::new(http_tool));
        
        let neo4j_tool = Neo4jTool::new(neo4j);
        tool_registry.register(Box::new(neo4j_tool));
        
        drop(tool_registry);
        
        self.initialized.store(true, Ordering::SeqCst);
        Ok(())
    }

    async fn call_plugin_as_tool(&self, name: &str, args: HashMap<String, Value>) -> anyhow::Result<Vec<ContentBlock>> {
        debug!("Mapping tool call to plugin: {} with args: {:?}", name, args);
        let registry = self.plugin_registry.lock().await;
        let plugin_name = match name {
            "system_info" => "system_info",
            "homeassistant" => "home_assistant",
            "http_request" => "http",
            "neo4j_query" => "neo4j",
            _ => return Err(anyhow::anyhow!("Tool not found: {}", name))
        };

        let plugin = registry.get_plugin(plugin_name).ok_or_else(|| {
            anyhow::anyhow!("Plugin not found: {}", plugin_name)
        })?;

        // Map tool names to plugin capabilities
        let (capability, mapped_args) = match name {
            "system_info" => {
                let action = args.get("action")
                    .and_then(|v| v.as_str())
                    .unwrap_or("get_system_info");
                debug!("Mapping system_info action '{}' to capability", action);
                ("get_system_info", args)
            },
            "homeassistant" => {
                let action = args.get("action")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("action is required for homeassistant"))?;
                debug!("Mapping homeassistant action '{}' to capability", action);
                match action {
                    "get_states" => ("get_states", args),
                    "get_state" => ("get_state", args),
                    "call_service" => ("call_service", args),
                    "get_services" => ("get_services", args),
                    _ => return Err(anyhow::anyhow!("Unknown homeassistant action: {}", action))
                }
            },
            "http_request" => {
                debug!("Mapping http_request tool to http plugin 'request' capability");
                ("request", args)
            },
            _ => return Err(anyhow::anyhow!("Unknown tool: {}", name))
        };

        let context = crate::plugins::Context {
            correlation_id: "tool_call".to_string(),
            timestamp: chrono::Utc::now(),
            parameters: mapped_args.clone(),
        };

        debug!("Executing plugin {} with capability {} and args {:?}", plugin_name, capability, mapped_args);
        let result = plugin.execute(capability, context, mapped_args).await
            .map_err(|e| anyhow::anyhow!("Plugin execution failed: {}", e))?;

        // Convert plugin result to ContentBlock with proper formatting
        let result_text = serde_json::to_string_pretty(&result.data)
            .map_err(|e| anyhow::anyhow!("Failed to serialize plugin result: {}", e))?;
            
        let content_block = ContentBlock::text(&result_text);
        Ok(vec![content_block])
    }

    async fn handle_plugins_list(&self, request: &JsonRpcRequest) -> String {
        let registry = self.plugin_registry.lock().await;
        let plugins = registry.list_plugins();
        
        self.create_success_response(
            request.id.clone(),
            serde_json::json!({
                "plugins": plugins
            }),
        )
    }

    async fn handle_plugins_call(&self, request: &JsonRpcRequest) -> String {
        let params: Result<PluginCallParams, _> = serde_json::from_value(request.params.clone().unwrap_or(Value::Null));
        
        let params = match params {
            Ok(p) => p,
            Err(e) => {
                return self.create_error_response(
                    request.id.clone(),
                    -32602,
                    "Invalid params",
                    Some(Value::String(e.to_string())),
                )
            }
        };

        let registry = self.plugin_registry.lock().await;
        let plugin = match registry.get_plugin(&params.name) {
            Some(p) => p,
            None => {
                return self.create_error_response(
                    request.id.clone(),
                    -32601,
                    "Plugin not found",
                    None,
                )
            }
        };

        let context = crate::plugins::Context {
            correlation_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            parameters: params.args.clone(),
        };

        match plugin.execute(&params.action, context, params.args).await {
            Ok(result) => self.create_success_response(request.id.clone(), serde_json::json!(result)),
            Err(e) => self.create_error_response(
                request.id.clone(),
                -32603,
                "Plugin execution failed",
                Some(Value::String(e.to_string())),
            ),
        }
    }

    pub async fn handle_message(&self, message: &str) -> anyhow::Result<String> {
        let message = message.trim();
        if message.is_empty() {
            return Ok(String::new());
        }

        debug!("Received message: {}", message);

        let request: JsonRpcRequest = match serde_json::from_str(message) {
            Ok(req) => req,
            Err(e) => {
                error!("Failed to parse JSON-RPC request: {}", e);
                return Ok(self.create_error_response(None, -32700, "Parse error", None));
            }
        };

        // Only allow initialize method if not initialized
        if !self.initialized.load(Ordering::SeqCst) && request.method != "initialize" {
            return Ok(self.create_error_response(
                request.id.clone(),
                -32002,
                "Server not initialized",
                None,
            ));
        }

        let response = match request.method.as_str() {
            "initialize" => self.handle_initialize(&request).await,
            "tools/list" => self.handle_tools_list(&request).await,
            "tools/call" => self.handle_tool_call(&request).await,
            "plugins/list" => self.handle_plugins_list(&request).await,
            "plugins/call" => self.handle_plugins_call(&request).await,
            _ => self.create_error_response(
                request.id.clone(),
                -32601,
                "Method not found",
                None,
            ),
        };

        Ok(response)
    }

    async fn handle_initialize(&self, request: &JsonRpcRequest) -> String {
        info!("Handling initialize request");

        // Check if already initialized
        if self.initialized.load(Ordering::SeqCst) {
            return self.create_error_response(
                request.id.clone(),
                -32002,
                "Server already initialized",
                None,
            );
        }
        
        let init_result = InitializeResult {
            protocol_version: "2024-11-05".to_string(),
            capabilities: Capabilities {
                tools: Some(ToolCapabilities { list_changed: Some(false) }),
                ..Default::default()
            },
            server_info: ServerInfo {
                name: "ollama-n8n-mcp-server".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        // Set initialized flag atomically
        self.initialized.store(true, Ordering::SeqCst);

        self.create_success_response(request.id.clone(), init_result)
    }

    async fn handle_tools_list(&self, request: &JsonRpcRequest) -> String {
        debug!("Handling tools/list request");
        
        let tool_registry = self.tool_registry.lock().await;
        let tools = tool_registry.list_tools().await;
        drop(tool_registry);
        
        let result = ToolsListResult { tools };
        
        self.create_success_response(request.id.clone(), result)
    }

    async fn handle_tool_call(&self, request: &JsonRpcRequest) -> String {
        debug!("Received tool call request: {:?}", request);
        
        let params = match request.params.as_ref() {
            Some(value) => match serde_json::from_value::<ToolCallParams>(value.clone()) {
                Ok(p) => p,
                Err(e) => {
                    error!("Invalid tool call parameters: {}", e);
                    return self.create_error_response(
                        request.id.clone(),
                        -32602,
                        "Invalid params",
                        None,
                    );
                }
            },
            None => {
                error!("Missing parameters in tool call request");
                return self.create_error_response(
                    request.id.clone(),
                    -32602,
                    "Missing params",
                    None,
                );
            }
        };

        debug!("Handling tool call for {} with arguments {:?}", params.name, params.arguments);
        match self.call_plugin_as_tool(&params.name, params.arguments).await {
            Ok(result) => {
                debug!("Tool call succeeded with result length {}", result.len());
                let response = ToolCallResult { content: result };
                self.create_success_response(request.id.clone(), response)
            }
            Err(e) => {
                error!("Tool call failed: {}", e);
                self.create_error_response(
                    request.id.clone(),
                    -1,
                    "Tool execution failed",
                    Some(Value::String(e.to_string())),
                )
            }
        }
    }

    fn create_success_response<T: Serialize>(&self, id: Option<Value>, result: T) -> String {
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(serde_json::to_value(result).unwrap()),
            error: None,
        };
        serde_json::to_string(&response).unwrap()
    }

    fn create_error_response(
        &self,
        id: Option<Value>,
        code: i32,
        message: &str,
        data: Option<Value>,
    ) -> String {
        let error = JsonRpcError {
            code,
            message: message.to_string(),
            data,
        };
        let response = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(error),
        };
        serde_json::to_string(&response).unwrap()
    }
}