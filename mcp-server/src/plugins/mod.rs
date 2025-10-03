use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

pub mod system_info;
pub mod home_assistant;
pub mod http;
pub mod neo4j;

/// Represents the capability of a plugin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ParameterDefinition>,
}

/// Defines a parameter for a plugin capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDefinition {
    pub name: String,
    pub description: String,
    pub parameter_type: ParameterType,
    pub required: bool,
}

/// Supported parameter types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    Object,
    Array,
}

/// Plugin execution context
#[derive(Debug, Clone)]
pub struct Context {
    pub correlation_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Plugin execution result
#[derive(Debug, Clone, Serialize)]
pub struct PluginResult {
    pub success: bool,
    pub data: serde_json::Value,
    pub metrics: Option<HashMap<String, f64>>,
    pub context_updates: Option<HashMap<String, serde_json::Value>>,
}

/// Core plugin trait that all plugins must implement
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Returns the name of the plugin
    fn name(&self) -> &str;
    
    /// Returns the version of the plugin
    fn version(&self) -> &str;
    
    /// Returns the list of capabilities provided by this plugin
    fn capabilities(&self) -> Vec<Capability>;
    
    /// Executes a capability with the given context and parameters
    async fn execute(
        &self,
        capability: &str,
        context: Context,
        params: HashMap<String, serde_json::Value>,
    ) -> Result<PluginResult, Box<dyn Error + Send + Sync>>;
    
    /// Called when the plugin is loaded
    #[allow(unused_variables)]
    async fn initialize(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }
    
    /// Called when the plugin is being unloaded
    #[allow(unused_variables)]
    async fn shutdown(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }
}

/// Plugin manager to handle plugin lifecycle
pub struct PluginManager {
    plugins: HashMap<String, Arc<dyn Plugin>>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    /// Register a new plugin
    pub async fn register_plugin(&mut self, plugin: Arc<dyn Plugin>) -> Result<(), Box<dyn Error + Send + Sync>> {
        let name = plugin.name().to_string();
        plugin.initialize().await?;
        self.plugins.insert(name, plugin);
        Ok(())
    }

    /// Get a reference to a plugin by name
    pub fn get_plugin(&self, name: &str) -> Option<Arc<dyn Plugin>> {
        self.plugins.get(name).cloned()
    }

    /// List all registered plugins
    pub fn list_plugins(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }
}