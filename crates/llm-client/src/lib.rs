#![deny(clippy::all)]

use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Provider {
    Ollama,
    OpenRouter,
    AIFoundry,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    pub provider: Provider,
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: String,
}

/// Send a prompt to the configured LLM provider.
pub async fn send_completion(cfg: &LlmConfig, prompt: &str) -> Result<String> {
    let client = Client::new();
    let body = serde_json::json!({"model": cfg.model, "prompt": prompt});
    let res = client
        .post(&cfg.base_url)
        .bearer_auth(cfg.api_key.clone().unwrap_or_default())
        .json(&body)
        .send()
        .await?;
    let json: serde_json::Value = res.json().await?;
    Ok(json["choices"][0]["text"].as_str().unwrap_or_default().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[tokio::test]
    async fn test_send_completion() {
        let cfg = LlmConfig {
            provider: Provider::Custom,
            base_url: "http://localhost".into(),
            api_key: None,
            model: "test".into(),
        };
        // This will fail because there is no server, but we expect an error
        assert!(send_completion(&cfg, "hi").await.is_err());
    }
}
