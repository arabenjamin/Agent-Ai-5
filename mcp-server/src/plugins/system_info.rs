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