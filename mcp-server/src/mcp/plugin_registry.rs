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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::{Plugin, PluginResult, Capability, ParameterDefinition, ParameterType, Context};
    use async_trait::async_trait;
    use std::collections::HashMap;
    use std::error::Error as StdError;
    use serde_json::json;

    // Mock plugin for testing
    struct MockPlugin {
        name: String,
        version: String,
        initialize_should_fail: bool,
        shutdown_should_fail: bool,
    }

    impl MockPlugin {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                version: "1.0.0".to_string(),
                initialize_should_fail: false,
                shutdown_should_fail: false,
            }
        }

        fn with_init_failure(mut self) -> Self {
            self.initialize_should_fail = true;
            self
        }

        fn with_shutdown_failure(mut self) -> Self {
            self.shutdown_should_fail = true;
            self
        }
    }

    #[async_trait]
    impl Plugin for MockPlugin {
        fn name(&self) -> &str {
            &self.name
        }

        fn version(&self) -> &str {
            &self.version
        }

        fn capabilities(&self) -> Vec<Capability> {
            vec![Capability {
                name: "test_capability".to_string(),
                description: "A test capability".to_string(),
                parameters: vec![ParameterDefinition {
                    name: "param1".to_string(),
                    description: "Test parameter".to_string(),
                    parameter_type: ParameterType::String,
                    required: true,
                }],
            }]
        }

        async fn execute(
            &self,
            _capability: &str,
            _context: Context,
            _params: HashMap<String, serde_json::Value>,
        ) -> Result<PluginResult, Box<dyn StdError + Send + Sync>> {
            Ok(PluginResult {
                success: true,
                data: json!({"message": "Mock execution successful"}),
                metrics: None,
                context_updates: None,
            })
        }

        async fn initialize(&self) -> Result<(), Box<dyn StdError + Send + Sync>> {
            if self.initialize_should_fail {
                Err("Mock initialization failure".into())
            } else {
                Ok(())
            }
        }

        async fn shutdown(&self) -> Result<(), Box<dyn StdError + Send + Sync>> {
            if self.shutdown_should_fail {
                Err("Mock shutdown failure".into())
            } else {
                Ok(())
            }
        }
    }

    #[tokio::test]
    async fn test_new_registry() {
        let registry = PluginRegistry::new();
        assert_eq!(registry.list_plugins().len(), 0);
    }

    #[tokio::test]
    async fn test_register_plugin_success() {
        let mut registry = PluginRegistry::new();
        let plugin = Arc::new(MockPlugin::new("test_plugin"));
        
        let result = registry.register_plugin(plugin.clone()).await;
        assert!(result.is_ok());
        
        let plugins = registry.list_plugins();
        assert_eq!(plugins.len(), 1);
        assert!(plugins.contains(&"test_plugin".to_string()));
    }

    #[tokio::test]
    async fn test_register_plugin_init_failure() {
        let mut registry = PluginRegistry::new();
        let plugin = Arc::new(MockPlugin::new("failing_plugin").with_init_failure());
        
        let result = registry.register_plugin(plugin).await;
        assert!(result.is_err());
        
        // Plugin should not be registered if initialization fails
        let plugins = registry.list_plugins();
        assert_eq!(plugins.len(), 0);
    }

    #[tokio::test]
    async fn test_get_plugin_exists() {
        let mut registry = PluginRegistry::new();
        let plugin = Arc::new(MockPlugin::new("test_plugin"));
        
        registry.register_plugin(plugin.clone()).await.unwrap();
        
        let retrieved = registry.get_plugin("test_plugin");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name(), "test_plugin");
    }

    #[tokio::test]
    async fn test_get_plugin_not_exists() {
        let registry = PluginRegistry::new();
        let retrieved = registry.get_plugin("nonexistent_plugin");
        assert!(retrieved.is_none());
    }

    #[tokio::test]
    async fn test_register_multiple_plugins() {
        let mut registry = PluginRegistry::new();
        
        let plugin1 = Arc::new(MockPlugin::new("plugin1"));
        let plugin2 = Arc::new(MockPlugin::new("plugin2"));
        let plugin3 = Arc::new(MockPlugin::new("plugin3"));
        
        registry.register_plugin(plugin1).await.unwrap();
        registry.register_plugin(plugin2).await.unwrap();
        registry.register_plugin(plugin3).await.unwrap();
        
        let plugins = registry.list_plugins();
        assert_eq!(plugins.len(), 3);
        assert!(plugins.contains(&"plugin1".to_string()));
        assert!(plugins.contains(&"plugin2".to_string()));
        assert!(plugins.contains(&"plugin3".to_string()));
    }

    #[tokio::test]
    async fn test_list_plugins_empty() {
        let registry = PluginRegistry::new();
        let plugins = registry.list_plugins();
        assert!(plugins.is_empty());
    }

    #[tokio::test]
    async fn test_shutdown_success() {
        let mut registry = PluginRegistry::new();
        
        let plugin1 = Arc::new(MockPlugin::new("plugin1"));
        let plugin2 = Arc::new(MockPlugin::new("plugin2"));
        
        registry.register_plugin(plugin1).await.unwrap();
        registry.register_plugin(plugin2).await.unwrap();
        
        let result = registry.shutdown().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_shutdown_with_failures() {
        let mut registry = PluginRegistry::new();
        
        let plugin1 = Arc::new(MockPlugin::new("plugin1"));
        let plugin2 = Arc::new(MockPlugin::new("plugin2").with_shutdown_failure());
        let plugin3 = Arc::new(MockPlugin::new("plugin3").with_shutdown_failure());
        
        registry.register_plugin(plugin1).await.unwrap();
        registry.register_plugin(plugin2).await.unwrap();
        registry.register_plugin(plugin3).await.unwrap();
        
        let result = registry.shutdown().await;
        assert!(result.is_err());
        
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("plugin2"));
        assert!(error_msg.contains("plugin3"));
        assert!(error_msg.contains("Mock shutdown failure"));
    }

    #[tokio::test]
    async fn test_plugin_replacement() {
        let mut registry = PluginRegistry::new();
        
        // Register first plugin
        let plugin1 = Arc::new(MockPlugin::new("test_plugin"));
        registry.register_plugin(plugin1).await.unwrap();
        
        // Register second plugin with same name (should replace)
        let plugin2 = Arc::new(MockPlugin::new("test_plugin"));
        registry.register_plugin(plugin2.clone()).await.unwrap();
        
        let plugins = registry.list_plugins();
        assert_eq!(plugins.len(), 1);
        
        let retrieved = registry.get_plugin("test_plugin").unwrap();
        // Verify it's the second plugin by checking if it's the same Arc reference
        assert_eq!(Arc::as_ptr(&retrieved), Arc::as_ptr(&plugin2));
    }

    #[tokio::test]
    async fn test_plugin_capabilities() {
        let mut registry = PluginRegistry::new();
        let plugin = Arc::new(MockPlugin::new("test_plugin"));
        
        registry.register_plugin(plugin.clone()).await.unwrap();
        
        let retrieved = registry.get_plugin("test_plugin").unwrap();
        let capabilities = retrieved.capabilities();
        
        assert_eq!(capabilities.len(), 1);
        assert_eq!(capabilities[0].name, "test_capability");
        assert_eq!(capabilities[0].description, "A test capability");
        assert_eq!(capabilities[0].parameters.len(), 1);
        assert_eq!(capabilities[0].parameters[0].name, "param1");
    }

    #[tokio::test]
    async fn test_shutdown_empty_registry() {
        let registry = PluginRegistry::new();
        let result = registry.shutdown().await;
        assert!(result.is_ok());
    }
}