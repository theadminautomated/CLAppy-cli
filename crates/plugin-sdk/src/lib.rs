#![deny(clippy::all)]

use anyhow::Result;
use wasmtime::{AsContextMut, Engine, Instance, Module, Store};

pub struct Plugin {
    module: Module,
}

impl Plugin {
    /// Load a WebAssembly plugin from file.
    pub fn load(engine: &Engine, path: &str) -> Result<Self> {
        Ok(Self { module: Module::from_file(engine, path)? })
    }

    /// Invoke a function inside the plugin.
    pub fn invoke(&self, store: &mut Store<()>, func: &str) -> Result<()> {
        let instance = Instance::new(store.as_context_mut(), &self.module, &[])?;
        let f = instance.get_typed_func::<(), ()>(store.as_context_mut(), func)?;
        f.call(store.as_context_mut(), ())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    fn load_missing_plugin() {
        let engine = Engine::default();
        assert!(Plugin::load(&engine, "missing.wasm").is_err());
    }
}
