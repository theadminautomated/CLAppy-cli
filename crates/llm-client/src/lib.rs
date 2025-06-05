#![deny(clippy::all)]

use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fmt;
use regex::Regex;

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

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Provider::Ollama => "ollama",
            Provider::OpenRouter => "openrouter",
            Provider::AIFoundry => "aifoundry",
            Provider::Custom => "custom",
        };
        write!(f, "{}", s)
    }
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
    let telemetry = std::env::var("OPENWARP_TELEMETRY").unwrap_or_else(|_| "0".into()) == "1";
    let os = if telemetry { std::env::consts::OS } else { "unknown" };
    let shell = if telemetry { std::env::var("SHELL").unwrap_or_default() } else { "redacted".into() };
    let path = if telemetry { std::env::var("PATH").unwrap_or_default() } else { "<redacted>".into() };
    let failure = failure.unwrap_or("");
    let text = if telemetry {
        text.to_string()
    } else {
        Regex::new(r"/[^\s]+")
            .unwrap()
            .replace_all(text, "<path>")
            .into_owned()
    };
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
