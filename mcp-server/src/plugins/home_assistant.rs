use async_trait::async_trait;
use log::{info, error, debug};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use reqwest;

use super::{Plugin, Context, PluginResult, Capability, ParameterDefinition, ParameterType};

#[derive(Debug)]
struct HomeAssistantPluginError(String);

impl fmt::Display for HomeAssistantPluginError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for HomeAssistantPluginError {}

pub struct HomeAssistantPlugin {
    base_url: String,
    token: Option<String>,
}

impl HomeAssistantPlugin {
    pub fn new() -> Self {
        Self {
            base_url: std::env::var("HOMEASSISTANT_URL")
                .unwrap_or_else(|_| "http://localhost:8123".to_string()),
            token: std::env::var("HOMEASSISTANT_TOKEN").ok(),
        }
    }

    fn get_auth_header(&self) -> Result<String, Box<dyn Error + Send + Sync>> {
        match &self.token {
            Some(token) => Ok(format!("Bearer {}", token)),
            None => Err(Box::new(HomeAssistantPluginError("Home Assistant token not configured. Set HOMEASSISTANT_TOKEN environment variable.".to_string())))
        }
    }

    async fn get_states(&self) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let auth_header = self.get_auth_header()?;
        let client = reqwest::Client::new();
        let url = format!("{}/api/states", self.base_url);
        
        debug!("Fetching states from Home Assistant");
        let response = client
            .get(&url)
            .header("Authorization", &auth_header)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| Box::new(HomeAssistantPluginError(format!("Failed to fetch states: {}", e))))?;

        if response.status().is_success() {
            let states = response.json().await
                .map_err(|e| Box::new(HomeAssistantPluginError(format!("Failed to parse states response: {}", e))))?;
            Ok(states)
        } else {
            let error = response.text().await
                .map_err(|e| Box::new(HomeAssistantPluginError(format!("Failed to read error response: {}", e))))?;
            Err(Box::new(HomeAssistantPluginError(format!("Failed to get states: {}", error))))
        }
    }

    async fn get_state(&self, entity_id: &str) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let auth_header = self.get_auth_header()?;
        let client = reqwest::Client::new();
        let url = format!("{}/api/states/{}", self.base_url, entity_id);
        
        debug!("Fetching state for entity: {}", entity_id);
        let response = client
            .get(&url)
            .header("Authorization", &auth_header)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| Box::new(HomeAssistantPluginError(format!("Failed to fetch state: {}", e))))?;

        if response.status().is_success() {
            let state = response.json().await
                .map_err(|e| Box::new(HomeAssistantPluginError(format!("Failed to parse state response: {}", e))))?;
            Ok(state)
        } else {
            let error = response.text().await
                .map_err(|e| Box::new(HomeAssistantPluginError(format!("Failed to read error response: {}", e))))?;
            Err(Box::new(HomeAssistantPluginError(format!("Failed to get state for {}: {}", entity_id, error))))
        }
    }

    async fn call_service(&self, domain: &str, service: &str, service_data: Value) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let auth_header = self.get_auth_header()?;
        let client = reqwest::Client::new();
        let url = format!("{}/api/services/{}/{}", self.base_url, domain, service);
        
        debug!("Calling service {}.{} with data: {:?}", domain, service, service_data);
        let response = client
            .post(&url)
            .header("Authorization", &auth_header)
            .header("Content-Type", "application/json")
            .json(&service_data)
            .send()
            .await
            .map_err(|e| Box::new(HomeAssistantPluginError(format!("Failed to call service: {}", e))))?;

        if response.status().is_success() {
            let result = response.json().await
                .map_err(|e| Box::new(HomeAssistantPluginError(format!("Failed to parse service response: {}", e))))?;
            Ok(result)
        } else {
            let error = response.text().await
                .map_err(|e| Box::new(HomeAssistantPluginError(format!("Failed to read error response: {}", e))))?;
            Err(Box::new(HomeAssistantPluginError(format!("Failed to call service {}.{}: {}", domain, service, error))))
        }
    }

    async fn get_services(&self) -> Result<Value, Box<dyn Error + Send + Sync>> {
        let auth_header = self.get_auth_header()?;
        let client = reqwest::Client::new();
        let url = format!("{}/api/services", self.base_url);
        
        debug!("Fetching available services");
        let response = client
            .get(&url)
            .header("Authorization", &auth_header)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| Box::new(HomeAssistantPluginError(format!("Failed to fetch services: {}", e))))?;

        if response.status().is_success() {
            let services = response.json().await
                .map_err(|e| Box::new(HomeAssistantPluginError(format!("Failed to parse services response: {}", e))))?;
            Ok(services)
        } else {
            let error = response.text().await
                .map_err(|e| Box::new(HomeAssistantPluginError(format!("Failed to read error response: {}", e))))?;
            Err(Box::new(HomeAssistantPluginError(format!("Failed to get services: {}", error))))
        }
    }
}

#[async_trait]
impl Plugin for HomeAssistantPlugin {
    fn name(&self) -> &str {
        "home_assistant"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability {
                name: "get_states".to_string(),
                description: "Get all entity states from Home Assistant".to_string(),
                parameters: vec![],
            },
            Capability {
                name: "get_state".to_string(),
                description: "Get state of a specific entity".to_string(),
                parameters: vec![
                    ParameterDefinition {
                        name: "entity_id".to_string(),
                        description: "ID of the entity to query".to_string(),
                        parameter_type: ParameterType::String,
                        required: true,
                    },
                ],
            },
            Capability {
                name: "call_service".to_string(),
                description: "Call a Home Assistant service".to_string(),
                parameters: vec![
                    ParameterDefinition {
                        name: "domain".to_string(),
                        description: "Service domain".to_string(),
                        parameter_type: ParameterType::String,
                        required: true,
                    },
                    ParameterDefinition {
                        name: "service".to_string(),
                        description: "Service name".to_string(),
                        parameter_type: ParameterType::String,
                        required: true,
                    },
                    ParameterDefinition {
                        name: "service_data".to_string(),
                        description: "Data to pass to the service call".to_string(),
                        parameter_type: ParameterType::Object,
                        required: false,
                    },
                ],
            },
            Capability {
                name: "get_services".to_string(),
                description: "Get list of available Home Assistant services".to_string(),
                parameters: vec![],
            },
        ]
    }

    async fn execute(
        &self,
        capability: &str,
        _context: Context,
        params: HashMap<String, serde_json::Value>,
    ) -> Result<PluginResult, Box<dyn Error + Send + Sync>> {
        info!("Executing home_assistant plugin capability: {}", capability);
        debug!("Parameters received: {:?}", params);

        match capability {
            "get_states" => {
                let states = self.get_states().await?;
                Ok(PluginResult {
                    success: true,
                    data: states,
                    metrics: None,
                    context_updates: None,
                })
            }
            "get_state" => {
                let entity_id = params.get("entity_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Box::new(HomeAssistantPluginError("entity_id is required".to_string())))?;

                let state = self.get_state(entity_id).await?;
                Ok(PluginResult {
                    success: true,
                    data: state,
                    metrics: None,
                    context_updates: None,
                })
            }
            "call_service" => {
                let domain = params.get("domain")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Box::new(HomeAssistantPluginError("domain is required".to_string())))?;

                let service = params.get("service")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Box::new(HomeAssistantPluginError("service is required".to_string())))?;

                let service_data = params.get("service_data")
                    .cloned()
                    .unwrap_or(json!({}));

                let result = self.call_service(domain, service, service_data).await?;
                Ok(PluginResult {
                    success: true,
                    data: result,
                    metrics: None,
                    context_updates: None,
                })
            }
            "get_services" => {
                let services = self.get_services().await?;
                Ok(PluginResult {
                    success: true,
                    data: services,
                    metrics: None,
                    context_updates: None,
                })
            }
            _ => Err(Box::new(HomeAssistantPluginError(format!("Unknown capability: {}", capability)))),
        }
    }
}