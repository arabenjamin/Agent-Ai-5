use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{Result, Error};

use crate::plugins::Plugin;

pub struct PluginRegistry {
    plugins: HashMap<String, Arc<dyn Plugin + Send + Sync>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
        }
    }

    pub async fn register_plugin(&mut self, plugin: Arc<dyn Plugin + Send + Sync>) -> Result<()> {
        // Initialize the plugin
        if let Err(e) = plugin.initialize().await {
            return Err(Error::msg(format!("Failed to initialize plugin: {}", e)));
        }
        
        let name = plugin.name().to_string();
        self.plugins.insert(name, plugin);
        Ok(())
    }

    pub fn get_plugin(&self, name: &str) -> Option<Arc<dyn Plugin + Send + Sync>> {
        self.plugins.get(name).cloned()
    }

    pub fn list_plugins(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }

    pub async fn shutdown(&self) -> Result<()> {
        let mut errors = Vec::new();
        for plugin in self.plugins.values() {
            if let Err(e) = plugin.shutdown().await {
                errors.push(format!("Error shutting down plugin {}: {}", plugin.name(), e));
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(Error::msg(errors.join("\n")))
        }
    }
}