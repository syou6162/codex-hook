//! Action execution for codex-hook.
//!
//! Currently supports `ActionType::Command`, which runs a shell command
//! via `sh -c`. This mirrors cchook (Go)'s `runCommand` function.
//! `ActionType::Output` will be implemented in YAS-422.

use std::process::{Command, ExitStatus, Stdio};

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
