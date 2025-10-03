use std::sync::Arc;
use std::collections::HashMap;
use serde_json::Value;
use anyhow::Result;
use async_trait::async_trait;

use crate::mcp::ContentBlock;
use crate::plugins::{
    Plugin,
    system_info::SystemInfoPlugin,
    home_assistant::HomeAssistantPlugin,
    http::HttpPlugin,
    neo4j::Neo4jPlugin,
    Context,
};

use super::Tool;

pub struct SystemInfoTool {
    plugin: Arc<SystemInfoPlugin>,
}

impl SystemInfoTool {
    pub fn new(plugin: Arc<SystemInfoPlugin>) -> Self {
        Self { plugin }
    }
}

#[async_trait]
impl Tool for SystemInfoTool {
    fn name(&self) -> &str {
        "system_info"
    }

    fn description(&self) -> &str {
        "Get system information like memory usage, CPU load, etc."
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["get_system_info"],
                    "default": "get_system_info"
                }
            }
        })
    }

    async fn call(&self, args: HashMap<String, Value>) -> Result<Vec<ContentBlock>> {
        let context = Context {
            correlation_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            parameters: args.clone(),
        };
        let result = self.plugin.execute("get_system_info", context, args).await
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(vec![ContentBlock::text(&serde_json::to_string_pretty(&result.data)?)])
    }
}

pub struct HomeAssistantTool {
    plugin: Arc<HomeAssistantPlugin>,
}

impl HomeAssistantTool {
    pub fn new(plugin: Arc<HomeAssistantPlugin>) -> Self {
        Self { plugin }
    }
}

#[async_trait]
impl Tool for HomeAssistantTool {
    fn name(&self) -> &str {
        "homeassistant"
    }

    fn description(&self) -> &str {
        "Interact with Home Assistant devices and services"
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "required": ["action"],
            "properties": {
                "action": {
                    "type": "string",
                    "enum": ["get_states", "get_state", "call_service", "get_services"]
                },
                "entity_id": {
                    "type": "string"
                },
                "domain": {
                    "type": "string"
                },
                "service": {
                    "type": "string"
                },
                "service_data": {
                    "type": "object"
                }
            }
        })
    }

    async fn call(&self, args: HashMap<String, Value>) -> Result<Vec<ContentBlock>> {
        let action = args.get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing action parameter"))?;
            
        let context = Context {
            correlation_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            parameters: args.clone(),
        };
        let result = self.plugin.execute(action, context, args.clone()).await
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(vec![ContentBlock::text(&serde_json::to_string_pretty(&result.data)?)])
    }
}

pub struct HttpTool {
    plugin: Arc<HttpPlugin>,
}

impl HttpTool {
    pub fn new(plugin: Arc<HttpPlugin>) -> Self {
        Self { plugin }
    }
}

#[async_trait]
impl Tool for HttpTool {
    fn name(&self) -> &str {
        "http_request"
    }

    fn description(&self) -> &str {
        "Make HTTP requests to external services"
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "required": ["method", "url"],
            "properties": {
                "method": {
                    "type": "string",
                    "enum": ["GET", "POST", "PUT", "DELETE", "PATCH"]
                },
                "url": {
                    "type": "string"
                },
                "headers": {
                    "type": "object"
                },
                "body": {
                    "type": "object"
                }
            }
        })
    }

    async fn call(&self, args: HashMap<String, Value>) -> Result<Vec<ContentBlock>> {
        let context = Context {
            correlation_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            parameters: args.clone(),
        };
        let result = self.plugin.execute("request", context, args.clone()).await
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(vec![ContentBlock::text(&serde_json::to_string_pretty(&result.data)?)])
    }
}

pub struct Neo4jTool {
    plugin: Arc<Neo4jPlugin>,
}

impl Neo4jTool {
    pub fn new(plugin: Arc<Neo4jPlugin>) -> Self {
        Self { plugin }
    }
}

#[async_trait]
impl Tool for Neo4jTool {
    fn name(&self) -> &str {
        "neo4j_query"
    }

    fn description(&self) -> &str {
        "Execute Cypher queries against a Neo4j database"
    }

    fn input_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "required": ["query"],
            "properties": {
                "query": {
                    "type": "string",
                    "description": "The Cypher query to execute"
                },
                "params": {
                    "type": "object",
                    "description": "Optional parameters for the query",
                    "additionalProperties": true
                }
            }
        })
    }

    async fn call(&self, args: HashMap<String, Value>) -> Result<Vec<ContentBlock>> {
        let context = Context {
            correlation_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            parameters: args.clone(),
        };
        let result = self.plugin.execute("query", context, args.clone()).await
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(vec![ContentBlock::text(&serde_json::to_string_pretty(&result.data)?)])
    }
}