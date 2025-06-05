#![deny(clippy::all)]

use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Prompt sent to an LLM provider.
#[derive(Debug, Clone)]
pub struct Prompt {
    pub text: String,
}

/// Response returned by an LLM provider.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resp {
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Intent {
    Translate,
    Explain,
    ChangeModel,
    PluginSuggest,
}

impl fmt::Display for Intent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Intent::Translate => write!(f, "translate"),
            Intent::Explain => write!(f, "explain"),
            Intent::ChangeModel => write!(f, "change_model"),
            Intent::PluginSuggest => write!(f, "plugin_suggest"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LlmAction {
    Command(String),
    Explanation(String),
    Model(String),
    Plugin(String),
}

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

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn complete(&self, req: Prompt) -> Result<Resp>;
}

/// Build the prompt for a given [`Intent`].
fn build_prompt(intent: Intent, text: &str, failure: Option<&str>) -> String {
    let os = std::env::consts::OS;
    let shell = std::env::var("SHELL").unwrap_or_default();
    let path = std::env::var("PATH").unwrap_or_default();
    let failure = failure.unwrap_or("");
    let few_shot = "winget install git -> winget install git\nchoco install git -> choco install git\napt install git -> apt-get install git\nbrew install git -> brew install git";
    format!(
        "OS: {os}\nShell: {shell}\nPATH: {path}\nFailure: {failure}\nIntent: {intent}\nInput: {text}\n{few_shot}\nOutput:",
    )
}

/// Use the LLM to infer an action for the given text.
pub async fn infer<P: LlmProvider + ?Sized>(
    provider: &P,
    intent: Intent,
    text: &str,
    failure: Option<&str>,
) -> Result<LlmAction> {
    let prompt = build_prompt(intent, text, failure);
    let resp = provider.complete(Prompt { text: prompt }).await?;
    let action = match intent {
        Intent::Translate => LlmAction::Command(resp.text),
        Intent::Explain => LlmAction::Explanation(resp.text),
        Intent::ChangeModel => LlmAction::Model(resp.text),
        Intent::PluginSuggest => LlmAction::Plugin(resp.text),
    };
    Ok(action)
}

#[derive(Debug, Clone)]
struct HttpProvider {
    client: Client,
    base_url: String,
    api_key: Option<String>,
    model: String,
}

impl HttpProvider {
    fn new(base_url: String, api_key: Option<String>, model: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
            api_key,
            model,
        }
    }
}

#[async_trait]
impl LlmProvider for HttpProvider {
    async fn complete(&self, req: Prompt) -> Result<Resp> {
        let body = serde_json::json!({"model": self.model, "prompt": req.text});
        let mut req = self.client.post(&self.base_url).json(&body);
        if let Some(key) = &self.api_key {
            req = req.bearer_auth(key);
        }
        let res = req.send().await?;
        let json: serde_json::Value = res.json().await?;
        Ok(Resp {
            text: json["choices"][0]["text"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
        })
    }
}

macro_rules! adapter {
    ($name:ident) => {
        #[derive(Debug, Clone)]
        pub struct $name(HttpProvider);

        impl $name {
            pub fn new(base_url: String, api_key: Option<String>, model: String) -> Self {
                Self(HttpProvider::new(base_url, api_key, model))
            }
        }

        #[async_trait]
        impl LlmProvider for $name {
            async fn complete(&self, req: Prompt) -> Result<Resp> {
                self.0.complete(req).await
            }
        }
    };
}

adapter!(Ollama);
adapter!(OpenRouter);
adapter!(AIFoundry);
adapter!(Custom);

pub fn provider_from_config(cfg: &LlmConfig) -> Box<dyn LlmProvider> {
    match cfg.provider {
        Provider::Ollama => Box::new(Ollama::new(
            cfg.base_url.clone(),
            cfg.api_key.clone(),
            cfg.model.clone(),
        )),
        Provider::OpenRouter => Box::new(OpenRouter::new(
            cfg.base_url.clone(),
            cfg.api_key.clone(),
            cfg.model.clone(),
        )),
        Provider::AIFoundry => Box::new(AIFoundry::new(
            cfg.base_url.clone(),
            cfg.api_key.clone(),
            cfg.model.clone(),
        )),
        Provider::Custom => Box::new(Custom::new(
            cfg.base_url.clone(),
            cfg.api_key.clone(),
            cfg.model.clone(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn setup_mock() -> MockServer {
        MockServer::start().await
    }

    async fn mount_success(mock: &MockServer) {
        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw("{\"choices\":[{\"text\":\"pong\"}]}", "application/json"),
            )
            .mount(mock)
            .await;
    }

    #[rstest]
    #[tokio::test]
    async fn ollama_complete() {
        let mock = setup_mock().await;
        mount_success(&mock).await;
        let provider = Ollama::new(mock.uri(), None, "test".into());
        let resp = provider
            .complete(Prompt {
                text: "ping".into(),
            })
            .await
            .unwrap();
        assert_eq!(resp.text, "pong");
    }

    #[rstest]
    #[tokio::test]
    async fn openrouter_complete() {
        let mock = setup_mock().await;
        mount_success(&mock).await;
        let provider = OpenRouter::new(mock.uri(), None, "test".into());
        let resp = provider
            .complete(Prompt {
                text: "ping".into(),
            })
            .await
            .unwrap();
        assert_eq!(resp.text, "pong");
    }

    #[rstest]
    #[tokio::test]
    async fn aifoundry_complete() {
        let mock = setup_mock().await;
        mount_success(&mock).await;
        let provider = AIFoundry::new(mock.uri(), None, "test".into());
        let resp = provider
            .complete(Prompt {
                text: "ping".into(),
            })
            .await
            .unwrap();
        assert_eq!(resp.text, "pong");
    }

    #[rstest]
    #[tokio::test]
    async fn custom_complete() {
        let mock = setup_mock().await;
        mount_success(&mock).await;
        let provider = Custom::new(mock.uri(), None, "test".into());
        let resp = provider
            .complete(Prompt {
                text: "ping".into(),
            })
            .await
            .unwrap();
        assert_eq!(resp.text, "pong");
    }

    #[derive(Clone, Default)]
    struct TestProvider {
        prompts: std::sync::Arc<std::sync::Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl LlmProvider for TestProvider {
        async fn complete(&self, req: Prompt) -> Result<Resp> {
            self.prompts.lock().unwrap().push(req.text);
            Ok(Resp { text: "out".into() })
        }
    }

    #[rstest]
    #[case(Intent::Translate, LlmAction::Command("out".into()))]
    #[case(Intent::Explain, LlmAction::Explanation("out".into()))]
    #[case(Intent::ChangeModel, LlmAction::Model("out".into()))]
    #[case(Intent::PluginSuggest, LlmAction::Plugin("out".into()))]
    #[tokio::test]
    async fn infer_variants(#[case] intent: Intent, #[case] expected: LlmAction) {
        unsafe { std::env::set_var("SHELL", "bash") };
        unsafe { std::env::set_var("PATH", "/usr/bin") };
        let provider = TestProvider::default();
        let res = infer(&provider, intent, "hi", Some("fail")).await.unwrap();
        assert_eq!(res, expected);
        let prompt = provider.prompts.lock().unwrap().pop().unwrap();
        assert!(prompt.contains("OS:"));
        assert!(prompt.contains("Shell:"));
        assert!(prompt.contains("PATH:"));
        assert!(prompt.contains("Failure:"));
    }
}
