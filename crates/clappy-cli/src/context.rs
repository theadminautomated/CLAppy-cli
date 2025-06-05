#![deny(clippy::all)]

use std::collections::VecDeque;
use rocksdb::DB;
use terminal_core::Block;

pub struct ContextEngine {
    history: VecDeque<Block>,
    db: DB,
}

impl ContextEngine {
    pub fn new(path: &str) -> Self {
        let db = DB::open_default(path).expect("db");
        Self {
            history: VecDeque::new(),
            db,
        }
    }

    pub fn push(&mut self, block: Block) {
        if self.history.len() == 50 {
            self.history.pop_front();
        }
        self.history.push_back(block);
    }

    pub fn context(&self) -> String {
        self.history
            .iter()
            .map(|b| b.text.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn cached_cmd(&self, input: &str) -> Option<String> {
        self.db.get(input).ok().flatten().map(|v| String::from_utf8_lossy(&v).to_string())
    }

    pub fn cache_translation(&self, input: &str, cmd: &str) {
        let _ = self.db.put(input, cmd);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use rstest::rstest;

    #[rstest]
    fn sliding_window() {
        let dir = tempdir().unwrap();
        let mut ctx = ContextEngine::new(dir.path().to_str().unwrap());
        for i in 0..55 {
            ctx.push(Block { text: format!("b{i}") });
        }
        assert_eq!(ctx.history.len(), 50);
        assert!(ctx.context().contains("b54"));
        assert!(!ctx.context().contains("b0"));
    }

    #[rstest]
    fn cache_roundtrip() {
        let dir = tempdir().unwrap();
        let ctx = ContextEngine::new(dir.path().to_str().unwrap());
        assert!(ctx.cached_cmd("hi").is_none());
        ctx.cache_translation("hi", "echo hi");
        assert_eq!(ctx.cached_cmd("hi").as_deref(), Some("echo hi"));
    }
}
