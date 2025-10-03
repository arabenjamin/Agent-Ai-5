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