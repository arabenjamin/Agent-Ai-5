use async_trait::async_trait;
use neo4rs::*;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use std::error::Error as StdError;
use tracing::debug;

use crate::plugins::{Plugin, Context, Capability, ParameterDefinition, ParameterType, PluginResult};

type Result<T> = std::result::Result<T, Box<dyn StdError + Send + Sync>>;

pub struct Neo4jPlugin {
    graph: Graph,
}

impl Neo4jPlugin {
    pub async fn new(uri: &str, user: &str, password: &str) -> Result<Self> {
        let config = ConfigBuilder::new()
            .uri(uri)
            .user(user)
            .password(password)
            .max_connections(4)
            .build()?;
            
        let graph = Graph::connect(config).await?;
        Ok(Self { graph })
    }

    pub fn get_capabilities() -> Vec<Capability> {
        vec![
            Capability {
                name: "neo4j_query".to_string(),
                description: "Execute a Neo4j Cypher query and return the results".to_string(),
                parameters: vec![
                    ParameterDefinition {
                        name: "query".to_string(),
                        description: "The Cypher query to execute".to_string(),
                        parameter_type: ParameterType::String,
                        required: true,
                    },
                    ParameterDefinition {
                        name: "params".to_string(),
                        description: "Optional parameters for the query".to_string(),
                        parameter_type: ParameterType::Object,
                        required: false,
                    }
                ],
            }
        ]
    }
    
    async fn execute_query(&self, query: &str, params: &HashMap<String, Value>) -> Result<Value> {
        debug!("Executing Neo4j query: {} with params: {:?}", query, params);
        
        let mut rows = Vec::new();
        let mut result = self.graph.execute(Query::new(query.to_string())).await?;
        
        while let Some(row) = result.next().await? {
            let mut row_data = serde_json::Map::new();
            
            // Try to get the value using different field names
            for field in ["n", "r", "v", "value"] {
                if let Ok(value) = row.get::<String>(field) {
                    row_data.insert(field.to_string(), Value::String(value));
                    break;
                } else if let Ok(value) = row.get::<i64>(field) {
                    row_data.insert(field.to_string(), Value::Number(value.into()));
                    break;
                } else if let Ok(value) = row.get::<f64>(field) {
                    if let Some(num) = serde_json::Number::from_f64(value) {
                        row_data.insert(field.to_string(), Value::Number(num));
                        break;
                    }
                } else if let Ok(value) = row.get::<bool>(field) {
                    row_data.insert(field.to_string(), Value::Bool(value));
                    break;
                }
            }
            
            if row_data.is_empty() {
                // Fallback: try to get the first value if no named fields matched
                if let Ok(value) = row.get::<String>("0") {
                    row_data.insert("value".to_string(), Value::String(value));
                }
            }
            
            rows.push(Value::Object(row_data));
        }
        
        Ok(Value::Array(rows))
    }
}

#[async_trait]
impl Plugin for Neo4jPlugin {
    fn name(&self) -> &str {
        "neo4j"
    }
    
    fn version(&self) -> &str {
        "1.0.0"
    }
    
    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability {
                name: "query".to_string(),
                description: "Execute a Cypher query against the Neo4j database".to_string(),
                parameters: vec![
                    ParameterDefinition {
                        name: "query".to_string(),
                        description: "The Cypher query to execute".to_string(),
                        parameter_type: ParameterType::String,
                        required: true,
                    },
                    ParameterDefinition {
                        name: "parameters".to_string(),
                        description: "Optional parameters for the query".to_string(),
                        parameter_type: ParameterType::Object,
                        required: false,
                    }
                ],
            }
        ]
    }
    
    async fn execute(
        &self, 
        capability: &str, 
        _context: Context,
        params: HashMap<String, Value>
    ) -> Result<PluginResult> {
        match capability {
            "query" => {
                let query = params.get("query")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        let err = std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "query parameter is required"
                        );
                        Box::new(err) as Box<dyn StdError + Send + Sync>
                    })?;
                
                // Extract query parameters, excluding the query itself
                let query_params: HashMap<String, Value> = params.iter()
                    .filter(|&(k, _)| k != "query")
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                
                let result = self.execute_query(query, &query_params).await?;
                
                let mut metrics = HashMap::new();
                metrics.insert("rows".to_string(), result.as_array().map_or(0.0, |arr| arr.len() as f64));
                
                Ok(PluginResult {
                    success: true,
                    data: result,
                    metrics: Some(metrics),
                    context_updates: None,
                })
            }
            _ => {
                let err = std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Unknown capability: {}", capability)
                );
                Err(Box::new(err) as Box<dyn StdError + Send + Sync>)
            }
        }
    }
}