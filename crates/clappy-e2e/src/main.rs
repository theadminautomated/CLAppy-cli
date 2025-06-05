#![deny(clippy::all)]
use anyhow::Result;
use async_trait::async_trait;
use cucumber::{given, then, when, World};
use clappy_cli::{CommandRouter, Route, safety_scan, ContextEngine};
use llm_client::{LlmConfig, LlmProvider, Prompt, Provider};

#[derive(World, Default)]
struct MyWorld {
    router: Option<CommandRouter>,
    last_route: Option<Route>,
    plugin_output: Option<String>,
}

#[async_trait(?Send)]
#[given("a command router")]
async fn a_command_router(world: &mut MyWorld) {
    #[derive(Clone)]
    struct FakeProvider;
    #[async_trait]
    impl LlmProvider for FakeProvider {
        async fn complete(&self, req: Prompt) -> Result<llm_client::Resp> {
            Ok(llm_client::Resp { text: format!("echo {}", req.text) })
        }
    }
    let cfg = LlmConfig { provider: Provider::Ollama, base_url: "".into(), api_key: None, model: "m".into() };
    let dir = tempfile::tempdir().unwrap();
    let ctx = ContextEngine::new(dir.path().to_str().unwrap());
    world.router = Some(CommandRouter::with_provider(cfg, Box::new(FakeProvider), ctx));
}

#[when(regex = "^I translate \"(.+)\"$")]
async fn i_translate(world: &mut MyWorld, input: String) {
    let r = world.router.as_mut().unwrap().route(&input).await.unwrap();
    world.last_route = Some(r);
}

#[then(regex = "^the command should be \"(.+)\"$")]
async fn command_should_be(world: &mut MyWorld, cmd: String) {
    match world.last_route.as_ref().unwrap() {
        Route::Exec { cmd: c, .. } => assert_eq!(c, &cmd),
        _ => panic!(),
    }
}

#[when(regex = "^I route \"(.+)\"$")]
async fn i_route(world: &mut MyWorld, input: String) {
    let r = world.router.as_mut().unwrap().route(&input).await.unwrap();
    world.last_route = Some(r);
}

#[then(regex = "^the route should be Switch \"(.+)\"$")]
async fn route_switch(world: &mut MyWorld, model: String) {
    match world.last_route.as_ref().unwrap() {
        Route::Switch(m) => assert_eq!(m, &model),
        _ => panic!(),
    }
}

#[then(regex = "^the route should spawn \"(.+)\"$")]
async fn route_spawn(world: &mut MyWorld, shell: String) {
    match world.last_route.as_ref().unwrap() {
        Route::Spawn(s) => assert_eq!(s, &shell),
        _ => panic!(),
    }
}

#[when(regex = "^plugin bus processes \"(.+)\"$")]
async fn plugin_bus(world: &mut MyWorld, line: String) {
    let router = world.router.as_mut().unwrap();
    let mut bus = clappy_cli::PluginBus::new(LlmConfig { provider: Provider::Ollama, base_url: "".into(), api_key: None, model: "m".into() });
    let _ = bus.load_dir("plugins");
    world.plugin_output = bus.process_line(&line).unwrap();
}

#[then(regex = "^plugin output should be \"(.+)\"$")]
async fn plugin_output(world: &mut MyWorld, out: String) {
    assert_eq!(world.plugin_output.as_deref(), Some(out.as_str()));
}

#[when(regex = "^safety scan \"(.+)\"$")]
async fn safety(world: &mut MyWorld, cmd: String) {
    world.plugin_output = Some(safety_scan(&cmd).to_string());
}

#[then(regex = "^scan result is (.+)$")]
async fn scan_result(world: &mut MyWorld, expect: String) {
    assert_eq!(world.plugin_output.as_deref(), Some(expect.as_str()));
}

#[tokio::main]
async fn main() {
    MyWorld::run("features").await;
}
