#![deny(clippy::all)]

use anyhow::Result;
use std::process::Command;

/// Run a shell command and return its stdout.
pub fn run_command(cmd: &str) -> Result<String> {
    let output = Command::new("sh").arg("-c").arg(cmd).output()?;
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("echo hi", "hi")]
    fn test_run_command(#[case] input: &str, #[case] expected: &str) {
        let result = run_command(input).unwrap();
        assert_eq!(result, expected);
    }
}
