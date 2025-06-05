#![deny(clippy::all)]

use anyhow::Result;
use clap::Parser;
use llm_client::{send_completion, LlmConfig, Provider};
use terminal_core::{run, Block};
use tokio_stream::StreamExt;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    /// LLM provider to use
    #[arg(long, default_value = "ollama")]
    provider: String,
    /// Model name
    #[arg(long, default_value = "llama3")]
    model: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let cfg = LlmConfig {
        provider: match args.provider.as_str() {
            "ollama" => Provider::Ollama,
            "openrouter" => Provider::OpenRouter,
            "aifoundry" => Provider::AIFoundry,
            _ => Provider::Custom,
        },
        base_url: "http://localhost".into(),
        api_key: None,
        model: args.model.clone(),
    };
    let _ = send_completion(&cfg, "hello").await.ok();

    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg("echo openwarp");
    let mut stream = run(cmd).await?;
    while let Some(Block { text }) = stream.next().await {
        println!("{}", text);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("--provider", "ollama", Provider::Ollama)]
    #[case("--provider", "openrouter", Provider::OpenRouter)]
    fn parse_provider(#[case] flag: &str, #[case] value: &str, #[case] expected: Provider) {
        let cli = Cli::try_parse_from(["test", flag, value]).unwrap();
        let provider = match cli.provider.as_str() {
            "ollama" => Provider::Ollama,
            "openrouter" => Provider::OpenRouter,
            "aifoundry" => Provider::AIFoundry,
            _ => Provider::Custom,
        };
        match (&provider, &expected) {
            (Provider::Ollama, Provider::Ollama)
            | (Provider::OpenRouter, Provider::OpenRouter) => {}
            _ => panic!("mismatch"),
        }
    }
}
