//! Action execution for codex-hook.
//!
//! Supports two action types:
//! - `ActionType::Command`: runs a shell command via `sh -c` (cchook `runCommand`).
//! - `ActionType::Output`: serializes a message as JSON to stdout.

use serde::Serialize;
use std::process::{Command, ExitStatus, Stdio};

/// JSON output structure for the `output` action type.
///
/// Codex reads `{"message": "..."}` from the hook's stdout.
#[derive(Serialize)]
pub(crate) struct OutputMessage<'a> {
    pub(crate) message: &'a str,
}

/// Build a JSON string `{"message":"..."}` for the given message.
pub(crate) fn build_output_json(message: &str) -> String {
    serde_json::to_string(&OutputMessage { message }).expect("JSON serialization should not fail")
}

/// Execute a shell command via `sh -c` and return its exit status.
///
/// The child's stdout is suppressed (redirected to null) so that command
/// output does not pollute the Codex JSON protocol on stdout.
/// stderr is inherited so error output is visible to the user.
pub(crate) fn execute_command(command: &str) -> std::io::Result<ExitStatus> {
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdout(Stdio::null())
        .stderr(Stdio::inherit())
        .status()
}

#[cfg(test)]
#[path = "action_tests.rs"]
mod action_tests;
