use async_trait::async_trait;
use chrono::Utc;
use log::{info, error, debug};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::error::Error;
use std::fmt;
use sysinfo::{System, SystemExt, CpuExt};

use crate::context::{Neo4jContext, get_neo4j_context, RelationType};
use super::{Plugin, Context, PluginResult, Capability, ParameterDefinition, ParameterType};

#[derive(Debug)]
struct SystemPluginError(String);

impl fmt::Display for SystemPluginError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// SystemPluginError automatically implements Send + Sync because String does
impl Error for SystemPluginError {}

pub struct SystemInfoPlugin {
    sys: Arc<tokio::sync::Mutex<System>>,
    context: Arc<tokio::sync::RwLock<Option<Arc<Neo4jContext>>>>,
}

impl SystemInfoPlugin {
    pub fn new() -> Self {
        Self {
            sys: Arc::new(tokio::sync::Mutex::new(System::new_all())),
            context: Arc::new(tokio::sync::RwLock::new(None)),
        }
    }
    
    async fn ensure_context(&self) -> Result<Arc<Neo4jContext>, Box<dyn Error + Send + Sync>> {
        let mut context = self.context.write().await;
        if context.is_none() {
            *context = Some(get_neo4j_context().await.map_err(|e| {
                Box::new(SystemPluginError(format!("Failed to get Neo4j context: {}", e))) as Box<dyn Error + Send + Sync>
            })?);
        }
        Ok(context.as_ref().unwrap().clone())
    }
    
    async fn store_metrics(&self, metrics: &HashMap<String, serde_json::Value>) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!("Attempting to store system metrics");
        debug!("Metrics to store: {:?}", metrics);
        
        let context = match self.ensure_context().await {
            Ok(ctx) => {
                info!("Successfully obtained Neo4j context");
                debug!("Neo4j context acquired successfully");
                ctx
            },
            Err(e) => {
                error!("Failed to get Neo4j context: {:#}", e);
                debug!("Full error context: {:?}", e);
                return Err(Box::new(SystemPluginError(format!("Failed to get Neo4j context: {:#}", e))) as Box<dyn Error + Send + Sync>);
            }
        };
        // Store the complete system state
        debug!("Storing complete system state...");
        let state_node = context.store_system_state(metrics.clone()).await
            .map_err(|e| {
                error!("Failed to store system state: {}", e);
                Box::new(SystemPluginError(format!("Failed to store system state: {}", e))) as Box<dyn Error + Send + Sync>
            })?;
        debug!("System state stored successfully");
        
        // Store individual metrics
        for (metric_name, value) in metrics {
            debug!("Storing metric '{}' with value: {:?}", metric_name, value);
            let metric_node = context.store_metric(metric_name, value.clone(), Utc::now()).await
                .map_err(|e| {
                    error!("Failed to store metric '{}': {}", metric_name, e);
                    Box::new(SystemPluginError(format!("Failed to store metric '{}': {}", metric_name, e))) as Box<dyn Error + Send + Sync>
                })?;
            debug!("Metric '{}' stored successfully", metric_name);
            
            // Create relationship between state and metric
            debug!("Creating relationship for metric '{}'...", metric_name);
            let mut props = HashMap::new();
            props.insert("timestamp".to_string(), json!(Utc::now().to_rfc3339()));
            
            let state_id = state_node.get::<String>("id")
                .map_err(|e| {
                    error!("Failed to get state ID: {}", e);
                    Box::new(SystemPluginError(format!("Failed to get state ID: {}", e))) as Box<dyn Error + Send + Sync>
                })?;
            let metric_id = metric_node.get::<String>("id")
                .map_err(|e| {
                    error!("Failed to get metric ID: {}", e);
                    Box::new(SystemPluginError(format!("Failed to get metric ID: {}", e))) as Box<dyn Error + Send + Sync>
                })?;
            debug!("Creating relationship between state '{}' and metric '{}'", state_id, metric_id);
                
            context.create_relationship(
                &state_id,
                &metric_id,
                RelationType::Contains,
                Some(props),
            ).await
                .map_err(|e| {
                    error!("Failed to create relationship for metric '{}': {}", metric_name, e);
                    Box::new(SystemPluginError(format!("Failed to create relationship: {}", e))) as Box<dyn Error + Send + Sync>
                })?;
            debug!("Relationship created successfully for metric '{}'", metric_name);
        }
        
        Ok(())
    }

    async fn get_system_info(&self) -> HashMap<String, serde_json::Value> {
        debug!("Getting system information...");
        let mut sys = self.sys.lock().await;
        debug!("Refreshing system metrics...");
        sys.refresh_all();
        
        let mut info = HashMap::new();
        
        // CPU information
        debug!("Calculating CPU usage...");
        let cpu_usage: f32 = sys.cpus().iter().map(|cpu| cpu.cpu_usage()).sum::<f32>() / 
                            sys.cpus().len() as f32;
        debug!("CPU usage: {:.2}%", cpu_usage);
        info.insert("cpu_usage".to_string(), json!(cpu_usage));
        
        // Memory information
        let total_memory = sys.total_memory();
        let used_memory = sys.used_memory();
        let memory_usage = (used_memory as f64 / total_memory as f64) * 100.0;
        
        info.insert("total_memory_kb".to_string(), json!(total_memory));
        info.insert("used_memory_kb".to_string(), json!(used_memory));
        info.insert("memory_usage_percent".to_string(), json!(memory_usage));
        
        // System information
        if let Some(name) = sys.name() {
            info.insert("os_name".to_string(), json!(name));
        }
        if let Some(version) = sys.os_version() {
            info.insert("os_version".to_string(), json!(version));
        }
        if let Some(hostname) = sys.host_name() {
            info.insert("hostname".to_string(), json!(hostname));
        }

        info
    }
}

#[async_trait]
impl Plugin for SystemInfoPlugin {
    fn name(&self) -> &str {
        "system_info"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability {
                name: "get_system_info".to_string(),
                description: "Get current system information including CPU, memory, and OS details".to_string(),
                parameters: vec![],
            },
            Capability {
                name: "get_memory_usage".to_string(),
                description: "Get current memory usage information".to_string(),
                parameters: vec![
                    ParameterDefinition {
                        name: "include_details".to_string(),
                        description: "Whether to include detailed memory statistics".to_string(),
                        parameter_type: ParameterType::Boolean,
                        required: false,
                    },
                ],
            },
        ]
    }

    async fn execute(
        &self,
        capability: &str,
        _context: Context,
        params: HashMap<String, serde_json::Value>,
    ) -> Result<PluginResult, Box<dyn Error + Send + Sync>> {
        info!("Executing system_info plugin capability: {}", capability);
        
        match capability {
            "get_system_info" => {
                info!("Collecting system information");
                debug!("Parameters received: {:?}", params);
                let info = self.get_system_info().await;
                debug!("Collected system info: {:?}", info);
                
                // Store metrics in Neo4j
                info!("Attempting to store metrics in Neo4j");
                match self.store_metrics(&info).await {
                    Ok(_) => {
                        info!("Successfully stored metrics in Neo4j");
                    },
                    Err(e) => {
                        error!("Failed to store metrics in Neo4j: {:#}", e);
                        debug!("Full error context: {:?}", e);
                        return Err(Box::new(SystemPluginError(format!("Failed to store metrics in Neo4j: {:#}", e))) as Box<dyn Error + Send + Sync>);
                    }
                }
                info!("Successfully stored metrics in Neo4j");
                
                Ok(PluginResult {
                    success: true,
                    data: json!(info),
                    metrics: Some(HashMap::from([
                        ("execution_time_ms".to_string(), 0.0),
                    ])),
                    context_updates: Some(HashMap::from([
                        ("last_system_check".to_string(), json!(chrono::Utc::now())),
                    ])),
                })
            },
            "get_memory_usage" => {
                info!("Getting memory usage information");
                let mut sys = self.sys.lock().await;
                sys.refresh_memory();
                
                let include_details = params.get("include_details")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let mut memory_info = HashMap::new();
                memory_info.insert("total_memory_kb".to_string(), json!(sys.total_memory()));
                memory_info.insert("used_memory_kb".to_string(), json!(sys.used_memory()));
                
                if include_details {
                    memory_info.insert("free_memory_kb".to_string(), json!(sys.free_memory()));
                    memory_info.insert("available_memory_kb".to_string(), json!(sys.available_memory()));
                }
                drop(sys); // Release the lock before async operations
                
                // Store memory metrics in Neo4j
                info!("Attempting to store memory metrics in Neo4j");
                if let Err(e) = self.store_metrics(&memory_info).await {
                    error!("Failed to store memory metrics in Neo4j: {}", e);
                    return Err(e);
                }
                info!("Successfully stored memory metrics in Neo4j");

                Ok(PluginResult {
                    success: true,
                    data: json!(memory_info),
                    metrics: Some(HashMap::from([
                        ("execution_time_ms".to_string(), 0.0),
                    ])),
                    context_updates: None,
                })
            },
            _ => Err(Box::new(SystemPluginError(String::from("Unsupported capability")))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;
    use chrono::Utc;

    #[test]
    fn test_system_plugin_error_display() {
        let error = SystemPluginError("Test error message".to_string());
        assert_eq!(format!("{}", error), "Test error message");
    }

    #[test]
    fn test_system_plugin_error_debug() {
        let error = SystemPluginError("Debug test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Debug test"));
    }

    #[test]
    fn test_system_info_plugin_creation() {
        let plugin = SystemInfoPlugin::new();
        assert_eq!(plugin.name(), "system_info");
        assert_eq!(plugin.version(), "0.1.0");
    }

    #[test]
    fn test_system_info_plugin_capabilities() {
        let plugin = SystemInfoPlugin::new();
        let capabilities = plugin.capabilities();
        
        assert_eq!(capabilities.len(), 2);
        
        // Check get_system_info capability
        let get_info_cap = capabilities.iter()
            .find(|c| c.name == "get_system_info")
            .expect("get_system_info capability should exist");
        
        assert_eq!(get_info_cap.description, "Get current system information including CPU, memory, and OS details");
        assert_eq!(get_info_cap.parameters.len(), 0);
        
        // Check get_memory_usage capability
        let memory_cap = capabilities.iter()
            .find(|c| c.name == "get_memory_usage")
            .expect("get_memory_usage capability should exist");
        
        assert_eq!(memory_cap.description, "Get current memory usage information");
        assert_eq!(memory_cap.parameters.len(), 1);
        assert_eq!(memory_cap.parameters[0].name, "include_details");
        assert_eq!(memory_cap.parameters[0].description, "Whether to include detailed memory statistics");
        assert!(matches!(memory_cap.parameters[0].parameter_type, ParameterType::Boolean));
        assert!(!memory_cap.parameters[0].required);
    }

    #[tokio::test]
    async fn test_get_system_info() {
        let plugin = SystemInfoPlugin::new();
        let info = plugin.get_system_info().await;
        
        // Verify basic system info fields are present based on actual implementation
        assert!(info.contains_key("cpu_usage"));
        assert!(info.contains_key("total_memory_kb"));
        assert!(info.contains_key("used_memory_kb"));
        assert!(info.contains_key("memory_usage_percent"));
        
        // Verify data types
        assert!(info["cpu_usage"].is_number());
        assert!(info["total_memory_kb"].is_number());
        assert!(info["used_memory_kb"].is_number());
        assert!(info["memory_usage_percent"].is_number());
        
        // Verify reasonable values
        let total_memory = info["total_memory_kb"].as_u64().unwrap();
        assert!(total_memory > 0);
        
        let used_memory = info["used_memory_kb"].as_u64().unwrap();
        assert!(used_memory > 0);
        assert!(used_memory <= total_memory);
        
        let memory_usage = info["memory_usage_percent"].as_f64().unwrap();
        assert!(memory_usage >= 0.0 && memory_usage <= 100.0);
        
        let cpu_usage = info["cpu_usage"].as_f64().unwrap();
        assert!(cpu_usage >= 0.0);
    }

    #[tokio::test]
    async fn test_plugin_trait_implementation() {
        let plugin = SystemInfoPlugin::new();
        
        // Test name and version
        assert_eq!(plugin.name(), "system_info");
        assert_eq!(plugin.version(), "0.1.0");
        
        // Test capabilities returns expected structure
        let capabilities = plugin.capabilities();
        assert!(!capabilities.is_empty());
        
        for capability in &capabilities {
            assert!(!capability.name.is_empty());
            assert!(!capability.description.is_empty());
            // Parameters can be empty for some capabilities
        }
    }

    #[tokio::test]
    async fn test_initialize_and_shutdown() {
        let plugin = SystemInfoPlugin::new();
        
        // Test initialization
        let init_result = plugin.initialize().await;
        assert!(init_result.is_ok());
        
        // Test shutdown
        let shutdown_result = plugin.shutdown().await;
        assert!(shutdown_result.is_ok());
    }

    // Note: The following tests would require a Neo4j test database
    // For now, we'll test the structure and error handling without actual execution
    
    #[tokio::test]
    async fn test_unsupported_capability() {
        let plugin = SystemInfoPlugin::new();
        let context = Context {
            correlation_id: "test-123".to_string(),
            timestamp: Utc::now(),
            parameters: HashMap::new(),
        };
        
        let result = plugin.execute(
            "unsupported_capability",
            context,
            HashMap::new(),
        ).await;
        
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Unsupported capability"));
    }

    #[test]
    fn test_parameter_types() {
        let plugin = SystemInfoPlugin::new();
        let capabilities = plugin.capabilities();
        
        for capability in capabilities {
            for param in capability.parameters {
                // Verify parameter types are valid enums
                match param.parameter_type {
                    ParameterType::String |
                    ParameterType::Number |
                    ParameterType::Boolean |
                    ParameterType::Object |
                    ParameterType::Array => {
                        // All valid types
                    }
                }
                
                // Verify parameter names and descriptions are non-empty
                assert!(!param.name.is_empty());
                assert!(!param.description.is_empty());
            }
        }
    }

    #[test]
    fn test_system_plugin_error_trait_implementations() {
        let error = SystemPluginError("test".to_string());
        
        // Test Error trait
        let error_trait: &dyn Error = &error;
        assert_eq!(error_trait.to_string(), "test");
        
        // Test Display trait
        assert_eq!(format!("{}", error), "test");
        
        // Test Debug trait  
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_context_structure() {
        let context = Context {
            correlation_id: "test-correlation-id".to_string(),
            timestamp: Utc::now(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("test_param".to_string(), json!("test_value"));
                params
            },
        };
        
        assert_eq!(context.correlation_id, "test-correlation-id");
        assert!(context.parameters.contains_key("test_param"));
        assert_eq!(context.parameters.get("test_param"), Some(&json!("test_value")));
    }
}