//! Action execution for codex-hook.
//!
//! Currently supports `ActionType::Command`, which runs a shell command
//! via `sh -c`. This mirrors cchook (Go)'s `runCommand` function.
//! `ActionType::Output` will be implemented in YAS-422.

use std::process::{Command, ExitStatus};

/// Execute a shell command via `sh -c` and return its exit status.
///
/// stdout and stderr are inherited from the parent process.
pub(crate) fn execute_command(command: &str) -> std::io::Result<ExitStatus> {
    Command::new("sh").arg("-c").arg(command).status()
}

#[cfg(test)]
#[path = "action_tests.rs"]
mod action_tests;
