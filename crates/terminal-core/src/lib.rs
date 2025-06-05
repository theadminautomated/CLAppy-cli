#![deny(clippy::all)]

use anyhow::Result;
use futures_util::compat::Future01CompatExt;
use futures::Future as Future01;
use regex::Regex;
use std::process::Command;
use tokio::sync::mpsc::unbounded_channel;
use tokio_pty_process::{AsyncPtyMaster, CommandExt};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::Stream;
use tokio_compat::runtime::Runtime;
use tokio01 as tokio_old;

/// A block of terminal output grouped by prompt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub text: String,
}

/// Strip ANSI escape sequences from a line.
fn strip_ansi(input: &str) -> String {
    // simple matcher for CSI codes
    static RE: once_cell::sync::Lazy<Regex> = once_cell::sync::Lazy::new(|| {
        Regex::new(r"\x1b\[[0-9;]*[mKJ]").unwrap()
    });
    RE.replace_all(input, "").into_owned()
}

/// Split raw output into blocks separated by shell prompts (`$ `).
pub fn parse_blocks(output: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut current = String::new();
    for line in output.lines() {
        let clean = strip_ansi(line);
        if clean.starts_with("$ ") && !current.is_empty() {
            blocks.push(Block { text: current.trim_end().to_string() });
            current.clear();
        }
        if !current.is_empty() {
            current.push('\n');
        }
        current.push_str(&clean);
    }
    if !current.is_empty() {
        blocks.push(Block { text: current.trim_end().to_string() });
    }
    blocks
}

/// Run a command asynchronously and stream its output as [`Block`]s.
pub async fn run(mut command: Command) -> Result<impl Stream<Item = Block>> {
    let (tx, rx) = unbounded_channel();

    tokio::task::spawn_blocking(move || {
        let mut rt = Runtime::new().expect("compat runtime");
        let master = AsyncPtyMaster::open().expect("pty");
        let mut child = command.spawn_pty_async(&master).expect("spawn");
        let (reader, _writer) = master.split();

        let read_fut = tokio_old::io::read_to_end(reader, Vec::new())
            .map(|(_, buf)| buf)
            .compat();

        let buf = rt.block_on_std(read_fut).expect("read");
        let _ = rt.block_on_std(child.compat());

        let text = String::from_utf8_lossy(&buf);
        for block in parse_blocks(&text) {
            let _ = tx.send(block);
        }
    });

    Ok(UnboundedReceiverStream::new(rx))
}


#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("$ echo hi\nhi\n", vec!["$ echo hi\nhi"])]
    #[case("\u{1b}[31mred\u{1b}[0m\n$ done\n", vec!["red", "$ done"])]
    fn parse_cases(#[case] input: &str, #[case] expected: Vec<&str>) {
        let blocks = parse_blocks(input);
        let texts: Vec<String> = blocks.into_iter().map(|b| b.text).collect();
        assert_eq!(texts, expected);
    }
}
