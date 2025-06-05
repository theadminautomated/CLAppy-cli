# Bring Your Own Model

Configure `clappy.toml` to point at your local or remote LLM provider.

```toml
[llm]
provider = "ollama" # openrouter | aifoundry | custom
base_url = "http://localhost:11434"
api_key = ""
model = "llama3"
```
