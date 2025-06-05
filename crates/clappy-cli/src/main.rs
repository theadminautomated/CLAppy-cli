#![deny(clippy::all)]

use anyhow::Result;
use clap::{Parser, Subcommand};
use clappy_cli::{CommandRouter, ContextEngine};
use llm_client::{LlmConfig, Provider, Prompt, provider_from_config};
use tokio::io::{AsyncBufReadExt, BufReader};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Cli {
    #[arg(long, default_value = "ollama")]
    provider: String,
    #[arg(long, default_value = "llama3")]
    model: String,
    #[arg(long, default_value_t = false)]
    insecure_telemetry: bool,
    #[arg(long, default_value_t = false)]
    i_know: bool,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Predict { input: String },
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

    if let Some(Commands::Predict { input }) = args.command {
        let provider = provider_from_config(&cfg);
        let resp = provider.complete(Prompt { text: input }).await?;
        println!("{}", resp.text);
        return Ok(());
    }

    if args.insecure_telemetry {
        println!("Telemetry enabled");
        unsafe { std::env::set_var("CLAPPY_TELEMETRY", "1"); }
    } else {
        unsafe { std::env::set_var("CLAPPY_TELEMETRY", "0"); }
    }

    let context = ContextEngine::new("context.db");
    let mut router = CommandRouter::new(cfg, context, args.i_know);
    let stdin = BufReader::new(tokio::io::stdin());
    let mut lines = stdin.lines();
    while let Some(line) = lines.next_line().await? {
        router.handle_line(&line).await?;
    }
    Ok(())
}
