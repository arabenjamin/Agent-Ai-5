use async_trait::async_trait;
use log::{info, error, debug};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use reqwest;

use super::{Plugin, Context, PluginResult, Capability, ParameterDefinition, ParameterType};

#[derive(Debug)]
struct HttpPluginError(String);

impl fmt::Display for HttpPluginError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for HttpPluginError {}

pub struct HttpPlugin;

impl HttpPlugin {
    pub fn new() -> Self {
        Self
    }

    async fn make_request(
        &self,
        method: &str,
        url: &str,
        headers: Option<HashMap<String, String>>,
        body: Option<String>,
        timeout: u64,
    ) -> Result<serde_json::Value, Box<dyn Error + Send + Sync>> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(timeout))
            .build()
            .map_err(|e| Box::new(HttpPluginError(format!("Failed to create HTTP client: {}", e))))?;

        let mut request = match method {
            "GET" => client.get(url),
            "POST" => client.post(url),
            "PUT" => client.put(url),
            "DELETE" => client.delete(url),
            "PATCH" => client.patch(url),
            _ => return Err(Box::new(HttpPluginError(format!("Unsupported HTTP method: {}", method)))),
        };

        // Add headers if provided
        if let Some(headers_map) = headers {
            for (key, value) in headers_map {
                request = request.header(&key, value);
            }
        }

        // Add body for POST, PUT, PATCH
        if matches!(method, "POST" | "PUT" | "PATCH") {
            if let Some(body_str) = body {
                request = request.body(body_str);
            }
        }

        debug!("Sending {} request to {}", method, url);
        let response = request.send().await
            .map_err(|e| Box::new(HttpPluginError(format!("Request failed: {}", e))))?;
        
        let status = response.status();
        let headers: HashMap<String, String> = response.headers()
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_str().unwrap_or("<invalid>").to_string()))
            .collect();
        
        let body = response.text().await
            .map_err(|e| Box::new(HttpPluginError(format!("Failed to read response body: {}", e))))?;

        Ok(json!({
            "status": status.as_u16(),
            "status_text": status.to_string(),
            "headers": headers,
            "body": body
        }))
    }
}

#[async_trait]
impl Plugin for HttpPlugin {
    fn name(&self) -> &str {
        "http"
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability {
                name: "request".to_string(),
                description: "Make an HTTP request to a URL".to_string(),
                parameters: vec![
                    ParameterDefinition {
                        name: "method".to_string(),
                        description: "HTTP method to use (GET, POST, PUT, DELETE, PATCH)".to_string(),
                        parameter_type: ParameterType::String,
                        required: true,
                    },
                    ParameterDefinition {
                        name: "url".to_string(),
                        description: "URL to send the request to".to_string(),
                        parameter_type: ParameterType::String,
                        required: true,
                    },
                    ParameterDefinition {
                        name: "headers".to_string(),
                        description: "HTTP headers to include".to_string(),
                        parameter_type: ParameterType::Object,
                        required: false,
                    },
                    ParameterDefinition {
                        name: "body".to_string(),
                        description: "Request body (for POST, PUT, PATCH)".to_string(),
                        parameter_type: ParameterType::String,
                        required: false,
                    },
                    ParameterDefinition {
                        name: "timeout".to_string(),
                        description: "Request timeout in seconds (default: 30)".to_string(),
                        parameter_type: ParameterType::Number,
                        required: false,
                    },
                ],
            }
        ]
    }

    async fn execute(
        &self,
        capability: &str,
        _context: Context,
        params: HashMap<String, serde_json::Value>,
    ) -> Result<PluginResult, Box<dyn Error + Send + Sync>> {
        info!("Executing http plugin capability: {}", capability);
        debug!("Parameters received: {:?}", params);

        match capability {
            "request" => {
                let method = params.get("method")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Box::new(HttpPluginError("method is required".to_string())))?
                    .to_uppercase();

                let url = params.get("url")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| Box::new(HttpPluginError("url is required".to_string())))?;

                let timeout = params.get("timeout")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(30);

                let headers = params.get("headers")
                    .and_then(|v| v.as_object())
                    .map(|obj| {
                        obj.iter()
                            .filter_map(|(k, v)| {
                                v.as_str().map(|s| (k.clone(), s.to_string()))
                            })
                            .collect::<HashMap<String, String>>()
                    });

                let body = params.get("body").and_then(|v| v.as_str()).map(|s| s.to_string());

                let result = self.make_request(&method, url, headers, body, timeout).await?;

                Ok(PluginResult {
                    success: true,
                    data: result,
                    metrics: None,
                    context_updates: None,
                })
            }
            _ => Err(Box::new(HttpPluginError(format!("Unknown capability: {}", capability)))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use chrono::Utc;

    #[test]
    fn test_http_plugin_error_display() {
        let error = HttpPluginError("Test error".to_string());
        assert_eq!(format!("{}", error), "Test error");
    }

    #[test]
    fn test_http_plugin_creation() {
        let plugin = HttpPlugin::new();
        assert_eq!(plugin.name(), "http");
        assert_eq!(plugin.version(), "0.1.0");
    }

    #[test]
    fn test_http_plugin_capabilities() {
        let plugin = HttpPlugin::new();
        let capabilities = plugin.capabilities();
        
        assert_eq!(capabilities.len(), 1);
        
        let request_cap = &capabilities[0];
        assert_eq!(request_cap.name, "request");
        assert_eq!(request_cap.description, "Make an HTTP request to a URL");
        assert_eq!(request_cap.parameters.len(), 5);
        
        // Check required parameters
        let method_param = request_cap.parameters.iter()
            .find(|p| p.name == "method")
            .expect("method parameter should exist");
        assert!(method_param.required);
        assert!(matches!(method_param.parameter_type, ParameterType::String));
        
        let url_param = request_cap.parameters.iter()
            .find(|p| p.name == "url")
            .expect("url parameter should exist");
        assert!(url_param.required);
        assert!(matches!(url_param.parameter_type, ParameterType::String));
        
        // Check optional parameters
        let headers_param = request_cap.parameters.iter()
            .find(|p| p.name == "headers")
            .expect("headers parameter should exist");
        assert!(!headers_param.required);
        assert!(matches!(headers_param.parameter_type, ParameterType::Object));
    }

    #[tokio::test]
    async fn test_unsupported_capability() {
        let plugin = HttpPlugin::new();
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
        assert!(error_msg.contains("Unknown capability"));
    }

    #[tokio::test]
    async fn test_initialize_and_shutdown() {
        let plugin = HttpPlugin::new();
        
        // Test initialization
        let init_result = plugin.initialize().await;
        assert!(init_result.is_ok());
        
        // Test shutdown
        let shutdown_result = plugin.shutdown().await;
        assert!(shutdown_result.is_ok());
    }

    #[test]
    fn test_http_plugin_error_trait_implementations() {
        let error = HttpPluginError("test".to_string());
        
        // Test Error trait
        let error_trait: &dyn Error = &error;
        assert_eq!(error_trait.to_string(), "test");
        
        // Test Display trait
        assert_eq!(format!("{}", error), "test");
        
        // Test Debug trait  
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("test"));
    }

    // Note: Testing actual HTTP requests would require mock servers
    // For integration tests, we'd use tools like wiremock
}