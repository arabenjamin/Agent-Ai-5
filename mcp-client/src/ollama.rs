use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Model {
    pub name: String,
}

#[derive(Debug, Serialize)]
struct GenerateRequest<'a> {
    model: &'a str,
    prompt: &'a str,
}

#[derive(Deserialize)]
struct GenerateResponse {
    response: String,
    done: bool,
}

pub struct OllamaClient {
    base_url: String,
    client: reqwest::Client,
}

impl OllamaClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            client: reqwest::Client::new(),
        }
    }

    pub async fn list_models(&self) -> Result<Vec<Model>> {
        let response = self.client
            .get(&format!("{}/api/tags", self.base_url))
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!(
                "Ollama server returned error status: {} with body: {}",
                status,
                error_text
            ));
        }

        #[derive(Deserialize)]
        struct ModelsResponse {
            models: Vec<Model>,
        }

        let response_data: ModelsResponse = response.json().await?;
        Ok(response_data.models)
    }

    pub async fn generate(&self, model: &str, prompt: &str) -> Result<String> {
        let request = GenerateRequest { model, prompt };

        let response = self.client
            .post(&format!("{}/api/generate", self.base_url))
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text().await?;
            return Err(anyhow::anyhow!(
                "Ollama server returned error status: {} with body: {}",
                status,
                error_text
            ));
        }

        let mut response_text = String::new();
        let mut stream = response.bytes_stream();
        use futures_util::StreamExt;
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let text = String::from_utf8_lossy(&chunk);
            let response_data: GenerateResponse = serde_json::from_str(&text)?;
            response_text.push_str(&response_data.response);
            if response_data.done {
                break;
            }
        }
        
        Ok(response_text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{
        matchers::{method, path, body_json},
        Mock, MockServer, ResponseTemplate,
    };
    use serde_json::json;

    #[tokio::test]
    async fn test_ollama_client_new() {
        let client = OllamaClient::new("http://localhost:11434");
        assert_eq!(client.base_url, "http://localhost:11434");
    }

    #[tokio::test]
    async fn test_list_models_success() {
        let mock_server = MockServer::start().await;
        
        let mock_response = json!({
            "models": [
                {
                    "name": "llama2:7b"
                },
                {
                    "name": "codellama:13b"
                },
                {
                    "name": "mistral:latest"
                }
            ]
        });

        Mock::given(method("GET"))
            .and(path("/api/tags"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
            .mount(&mock_server)
            .await;

        let client = OllamaClient::new(&mock_server.uri());
        let models = client.list_models().await.unwrap();

        assert_eq!(models.len(), 3);
        assert_eq!(models[0].name, "llama2:7b");
        assert_eq!(models[1].name, "codellama:13b");
        assert_eq!(models[2].name, "mistral:latest");
    }

    #[tokio::test]
    async fn test_list_models_empty_response() {
        let mock_server = MockServer::start().await;
        
        let mock_response = json!({
            "models": []
        });

        Mock::given(method("GET"))
            .and(path("/api/tags"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&mock_response))
            .mount(&mock_server)
            .await;

        let client = OllamaClient::new(&mock_server.uri());
        let models = client.list_models().await.unwrap();

        assert_eq!(models.len(), 0);
    }

    #[tokio::test]
    async fn test_list_models_server_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/tags"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal server error"))
            .mount(&mock_server)
            .await;

        let client = OllamaClient::new(&mock_server.uri());
        let result = client.list_models().await;

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("500"));
        assert!(error_msg.contains("Internal server error"));
    }

    #[tokio::test]
    async fn test_list_models_network_error() {
        let client = OllamaClient::new("http://localhost:99999");
        let result = client.list_models().await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_models_invalid_json() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/api/tags"))
            .respond_with(ResponseTemplate::new(200).set_body_string("invalid json"))
            .mount(&mock_server)
            .await;

        let client = OllamaClient::new(&mock_server.uri());
        let result = client.list_models().await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_generate_success_single_chunk() {
        let mock_server = MockServer::start().await;
        
        let expected_request = json!({
            "model": "llama2:7b",
            "prompt": "What is the capital of France?"
        });

        let response_chunk = json!({
            "response": "The capital of France is Paris.",
            "done": true
        });

        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .and(body_json(&expected_request))
            .respond_with(ResponseTemplate::new(200)
                .set_body_string(&serde_json::to_string(&response_chunk).unwrap()))
            .mount(&mock_server)
            .await;

        let client = OllamaClient::new(&mock_server.uri());
        let result = client.generate("llama2:7b", "What is the capital of France?").await.unwrap();

        assert_eq!(result, "The capital of France is Paris.");
    }

    #[tokio::test]
    async fn test_generate_success_streaming_simulation() {
        let mock_server = MockServer::start().await;
        
        let expected_request = json!({
            "model": "llama2:7b",
            "prompt": "Write a short story"
        });

        // Simulate a complete story response with done=true
        let response_chunk = json!({
            "response": "Once upon a time, there was a brave knight who saved the kingdom.",
            "done": true
        });

        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .and(body_json(&expected_request))
            .respond_with(ResponseTemplate::new(200)
                .set_body_string(&serde_json::to_string(&response_chunk).unwrap()))
            .mount(&mock_server)
            .await;

        let client = OllamaClient::new(&mock_server.uri());
        let result = client.generate("llama2:7b", "Write a short story").await.unwrap();

        assert_eq!(result, "Once upon a time, there was a brave knight who saved the kingdom.");
    }

    #[tokio::test]
    async fn test_generate_empty_response() {
        let mock_server = MockServer::start().await;
        
        let expected_request = json!({
            "model": "llama2:7b",
            "prompt": "Empty prompt"
        });

        let response_chunk = json!({
            "response": "",
            "done": true
        });

        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .and(body_json(&expected_request))
            .respond_with(ResponseTemplate::new(200)
                .set_body_string(&serde_json::to_string(&response_chunk).unwrap()))
            .mount(&mock_server)
            .await;

        let client = OllamaClient::new(&mock_server.uri());
        let result = client.generate("llama2:7b", "Empty prompt").await.unwrap();

        assert_eq!(result, "");
    }

    #[tokio::test]
    async fn test_generate_server_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Model not found"))
            .mount(&mock_server)
            .await;

        let client = OllamaClient::new(&mock_server.uri());
        let result = client.generate("nonexistent:model", "Test prompt").await;

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("500"));
        assert!(error_msg.contains("Model not found"));
    }

    #[tokio::test]
    async fn test_generate_network_error() {
        let client = OllamaClient::new("http://localhost:99999");
        let result = client.generate("llama2:7b", "Test prompt").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_generate_invalid_json_response() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .respond_with(ResponseTemplate::new(200).set_body_string("invalid json"))
            .mount(&mock_server)
            .await;

        let client = OllamaClient::new(&mock_server.uri());
        let result = client.generate("llama2:7b", "Test prompt").await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_generate_with_special_characters() {
        let mock_server = MockServer::start().await;
        
        let special_prompt = "What is 2+2? Include symbols: @#$%^&*()";
        let expected_request = json!({
            "model": "llama2:7b",
            "prompt": special_prompt
        });

        let response_chunk = json!({
            "response": "2+2 equals 4. Here are the symbols: @#$%^&*()",
            "done": true
        });

        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .and(body_json(&expected_request))
            .respond_with(ResponseTemplate::new(200)
                .set_body_string(&serde_json::to_string(&response_chunk).unwrap()))
            .mount(&mock_server)
            .await;

        let client = OllamaClient::new(&mock_server.uri());
        let result = client.generate("llama2:7b", special_prompt).await.unwrap();

        assert_eq!(result, "2+2 equals 4. Here are the symbols: @#$%^&*()");
    }

    #[tokio::test]
    async fn test_generate_long_prompt() {
        let mock_server = MockServer::start().await;
        
        let long_prompt = "A".repeat(1000);
        let expected_request = json!({
            "model": "llama2:7b",
            "prompt": long_prompt
        });

        let response_chunk = json!({
            "response": "This is a response to a very long prompt.",
            "done": true
        });

        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .and(body_json(&expected_request))
            .respond_with(ResponseTemplate::new(200)
                .set_body_string(&serde_json::to_string(&response_chunk).unwrap()))
            .mount(&mock_server)
            .await;

        let client = OllamaClient::new(&mock_server.uri());
        let result = client.generate("llama2:7b", &long_prompt).await.unwrap();

        assert_eq!(result, "This is a response to a very long prompt.");
    }

    #[tokio::test]
    async fn test_model_deserialization() {
        let json_data = json!({
            "name": "test-model:latest"
        });

        let model: Model = serde_json::from_value(json_data).unwrap();
        assert_eq!(model.name, "test-model:latest");
    }

    #[tokio::test]
    async fn test_generate_request_serialization() {
        let request = GenerateRequest {
            model: "llama2:7b",
            prompt: "Test prompt",
        };

        let json_value = serde_json::to_value(&request).unwrap();
        let expected = json!({
            "model": "llama2:7b",
            "prompt": "Test prompt"
        });

        assert_eq!(json_value, expected);
    }

    #[tokio::test]
    async fn test_generate_response_deserialization() {
        let json_data = json!({
            "response": "Test response text",
            "done": true
        });

        let response: GenerateResponse = serde_json::from_value(json_data).unwrap();
        assert_eq!(response.response, "Test response text");
        assert_eq!(response.done, true);
    }

    #[tokio::test]
    async fn test_generate_response_partial_deserialization() {
        let json_data = json!({
            "response": "Partial response",
            "done": false
        });

        let response: GenerateResponse = serde_json::from_value(json_data).unwrap();
        assert_eq!(response.response, "Partial response");
        assert_eq!(response.done, false);
    }
}