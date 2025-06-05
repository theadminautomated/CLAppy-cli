# OpenWarp

OpenWarp is a 100% open-source Warp-style terminal that runs as a standalone desktop app and as a CLI. It defaults to zero telemetry and supports BYOM (bring your own model) for AI features.

## Workspace Setup
Run `cargo check` in the repo root to verify all Rust crates.

## CLI Usage
Install with `cargo install --path crates/openwarp-cli` and run `openwarp --provider ollama --model llama3`.
The CLI features a dynamic command router. Enter a shell name (`bash`, `pwsh`, `cmd`) to spawn that shell, `/model <name>` to hot-swap the model, or any natural language which will be converted to a shell command using the selected provider. Telemetry is disabled unless `--insecure-telemetry` is passed.

```bash
> winget doesnt work
# AI: "Try 'winget source reset --force'"
winget source reset --force
```
