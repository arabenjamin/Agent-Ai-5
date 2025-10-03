use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use anyhow::Result;
use tracing::{debug, error, info};

use crate::mcp::{ContentBlock, ToolDefinition};

mod plugin_tools;
pub use plugin_tools::{SystemInfoTool, HomeAssistantTool, HttpTool, Neo4jTool};

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn input_schema(&self) -> Value;
    async fn call(&self, args: HashMap<String, Value>) -> Result<Vec<ContentBlock>>;
}

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register(&mut self, tool: Box<dyn Tool>) {
        let name = tool.name().to_string();
        self.tools.insert(name, tool);
    }

    pub async fn list_tools(&self) -> Vec<ToolDefinition> {
        debug!("Listing available tools: {:?}", self.tools.keys().collect::<Vec<_>>());
        self.tools
            .values()
            .map(|tool| ToolDefinition {
                name: tool.name().to_string(),
                description: tool.description().to_string(),
                input_schema: tool.input_schema(),
            })
            .collect()
    }

    pub async fn call_tool(
        &self,
        name: &str,
        args: HashMap<String, Value>,
    ) -> Result<Vec<ContentBlock>> {
        debug!("Attempting to call tool '{}' with args: {:?}", name, args);
        match self.tools.get(name) {
            Some(tool) => {
                debug!("Found tool '{}', executing...", name);
                let result = tool.call(args).await;
                match &result {
                    Ok(blocks) => debug!("Tool '{}' executed successfully with {} content blocks", name, blocks.len()),
                    Err(e) => error!("Tool '{}' execution failed: {}", name, e),
                }
                result
            },
            None => {
                error!("Tool '{}' not found. Available tools: {:?}", name, self.tools.keys().collect::<Vec<_>>());
                Err(anyhow::anyhow!("Tool '{}' not found", name))
            },
        }
    }
}