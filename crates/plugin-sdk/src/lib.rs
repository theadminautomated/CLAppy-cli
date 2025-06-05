#![deny(clippy::all)]

use anyhow::{Result, anyhow};
use llm_client::{LlmConfig, LlmProvider, Prompt, provider_from_config};
use tokio::runtime::Handle;
use wasmtime::{Caller, Engine, Extern, Instance, Linker, Memory, Module, Store};

struct HostState {
    stdin: String,
    selected: String,
    stdout: String,
    provider: Box<dyn LlmProvider>,
    handle: Handle,
}

impl HostState {
    fn memory<'a>(caller: &'a mut Caller<'_, Self>) -> Result<Memory> {
        match caller.get_export("memory") {
            Some(Extern::Memory(m)) => Ok(m),
            _ => Err(anyhow!("memory export not found")),
        }
    }

    fn write_str(caller: &mut Caller<'_, Self>, ptr: i32, s: &str) -> Result<i32> {
        let memory = Self::memory(caller)?;
        memory.write(caller, ptr as usize, s.as_bytes())?;
        Ok(s.len() as i32)
    }

    fn read_str(caller: &mut Caller<'_, Self>, ptr: i32, len: i32) -> Result<String> {
        let memory = Self::memory(caller)?;
        let mut buf = vec![0u8; len as usize];
        memory.read(caller, ptr as usize, &mut buf)?;
        Ok(String::from_utf8_lossy(&buf).to_string())
    }
}

pub struct Plugin {
    module: Module,
}

pub struct PluginInstance {
    store: Store<HostState>,
    instance: Instance,
}

impl Plugin {
    pub fn load(engine: &Engine, path: &str) -> Result<Self> {
        Ok(Self {
            module: Module::from_file(engine, path)?,
        })
    }

    pub fn instantiate(
        &self,
        engine: &Engine,
        cfg: LlmConfig,
        stdin: String,
        selected: String,
    ) -> Result<PluginInstance> {
        let provider = provider_from_config(&cfg);
        let handle = Handle::current();
        let state = HostState {
            stdin,
            selected,
            stdout: String::new(),
            provider,
            handle,
        };
        let mut store = Store::new(engine, state);
        let mut linker = Linker::new(engine);

        linker.func_wrap(
            "host",
            "stdin",
            |mut caller: Caller<'_, HostState>, ptr: i32| {
                let input = caller.data().stdin.clone();
                HostState::write_str(&mut caller, ptr, &input)
            },
        )?;

        linker.func_wrap(
            "host",
            "stdout",
            |mut caller: Caller<'_, HostState>, ptr: i32, len: i32| {
                let text = HostState::read_str(&mut caller, ptr, len)?;
                caller.data_mut().stdout.push_str(&text);
                Ok(())
            },
        )?;

        linker.func_wrap(
            "host",
            "selected_text",
            |mut caller: Caller<'_, HostState>, ptr: i32| {
                let sel = caller.data().selected.clone();
                HostState::write_str(&mut caller, ptr, &sel)
            },
        )?;

        linker.func_wrap(
            "host",
            "llm_complete",
            |mut caller: Caller<'_, HostState>, in_ptr: i32, in_len: i32, out_ptr: i32| {
                let prompt = HostState::read_str(&mut caller, in_ptr, in_len)?;
                let handle = caller.data().handle.clone();
                let fut = caller.data().provider.complete(Prompt { text: prompt });
                let resp = tokio::task::block_in_place(|| handle.block_on(fut))?;
                HostState::write_str(&mut caller, out_ptr, &resp.text)
            },
        )?;

        let instance = linker.instantiate(&mut store, &self.module)?;
        Ok(PluginInstance { store, instance })
    }
}

impl PluginInstance {
    pub fn invoke(&mut self, func: &str) -> Result<()> {
        let f = self
            .instance
            .get_typed_func::<(), ()>(&mut self.store, func)?;
        f.call(&mut self.store, ())?;
        Ok(())
    }

    pub fn output(&self) -> &str {
        &self.store.data().stdout
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use wasmtime::Engine;

    #[rstest]
    fn load_missing_plugin() {
        let engine = Engine::default();
        assert!(Plugin::load(&engine, "missing.wasm").is_err());
    }

    #[rstest]
    #[tokio::test(flavor = "multi_thread")]
    #[ignore]
    async fn run_gif_search() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw("{\"choices\":[{\"text\":\"gif-url\"}]}", "application/json"),
            )
            .mount(&server)
            .await;

        let engine = Engine::default();
        let plugin = Plugin::load(&engine, "../../plugins/gif_search.wasm").unwrap();
        let cfg = LlmConfig {
            provider: llm_client::Provider::Ollama,
            base_url: server.uri(),
            api_key: None,
            model: "test".into(),
        };
        let mut inst = plugin
            .instantiate(&engine, cfg, "cats".into(), "cats".into())
            .unwrap();
        inst.invoke("run").unwrap();
        assert_eq!(inst.output(), "gif-url");
    }
}
