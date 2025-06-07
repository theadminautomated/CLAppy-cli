# CLAppy

CLAppy is a 100% open-source Warp-style terminal that runs as a standalone desktop app and as a CLI. It defaults to zero telemetry and supports BYOM (bring your own model) for AI features.
This rebrand uses a glassmorphic design with Framer Motion micro-interactions and WCAG AA+ compliance. The mustache logo spins on click to toggle themes.

## Workspace Setup
Run `cargo check` in the repo root to verify all Rust crates.

## CLI Usage
Install with `cargo install --path crates/clappy-cli` and run `clappy --provider ollama --model llama3`.
The CLI features a dynamic command router. Enter a shell name (`bash`, `pwsh`, `cmd`) to spawn that shell, `/model <name>` to hot-swap the model, or any natural language which will be converted to a shell command using the selected provider. Telemetry is disabled unless `--insecure-telemetry` is passed.

## Windows Build
Run `pwsh scripts/build_setup.ps1` to create a single `clappy.exe` in the `dist` folder. The script now configures Visual Studio build tools, Node and Rust automatically before bundling the Tauri MSI with NSIS and `pkgforge` for a one-click installer.

```bash
> winget doesnt work
# AI: "Try 'winget source reset --force'"
winget source reset --force
```

## Documentation

Full documentation lives at [docs.clappy.dev](https://docs.clappy.dev). API docs are published to [docs.rs](https://docs.rs) and generated via `pnpm run typedoc` and `cargo doc`.
