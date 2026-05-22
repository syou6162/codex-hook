//! Condition evaluation logic for codex-hook.
//!
//! Evaluates conditions defined in config against hook input context.
//! All conditions in a hook must evaluate to `true` (AND semantics) for
//! actions to execute.
//!
//! Reference: cchook `checkCommonCondition` / `checkToolCondition` in utils.go.

use crate::config::{Condition, ConditionType};
use regex::Regex;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Command;
use walkdir::WalkDir;

/// Context needed for condition evaluation in PreToolUse events.
pub(crate) struct ConditionContext<'a> {
    pub cwd: &'a str,
    pub tool_input: &'a HashMap<String, serde_json::Value>,
    pub permission_mode: Option<&'a str>,
}

/// Context needed for condition evaluation in UserPromptSubmit events.
pub(crate) struct UserPromptSubmitConditionContext<'a> {
    pub cwd: &'a str,
    pub prompt: &'a str,
    pub transcript_path: Option<&'a str>,
    pub session_id: &'a str,
}

/// Evaluate all conditions for a hook. Returns `true` if all conditions pass
/// (or if the conditions list is empty).
pub(crate) fn evaluate_conditions(conditions: &[Condition], ctx: &ConditionContext) -> bool {
    conditions.iter().all(|c| evaluate_condition(c, ctx))
}

/// Evaluate a single condition against the context.
fn evaluate_condition(condition: &Condition, ctx: &ConditionContext) -> bool {
    match condition.condition_type {
        // -- Common: filesystem --
        ConditionType::FileExists => check_file_exists(ctx.cwd, &condition.value),
        ConditionType::FileNotExists => !check_file_exists(ctx.cwd, &condition.value),
        ConditionType::FileExistsRecursive => {
            check_file_exists_recursive(ctx.cwd, &condition.value)
        }
        ConditionType::FileNotExistsRecursive => {
            !check_file_exists_recursive(ctx.cwd, &condition.value)
        }
        ConditionType::DirExists => check_dir_exists(ctx.cwd, &condition.value),
        ConditionType::DirNotExists => !check_dir_exists(ctx.cwd, &condition.value),
        ConditionType::DirExistsRecursive => check_dir_exists_recursive(ctx.cwd, &condition.value),
        ConditionType::DirNotExistsRecursive => {
            !check_dir_exists_recursive(ctx.cwd, &condition.value)
        }

        // -- Common: cwd --
        ConditionType::CwdIs => ctx.cwd == condition.value,
        ConditionType::CwdIsNot => ctx.cwd != condition.value,
        ConditionType::CwdContains => ctx.cwd.contains(&condition.value),
        ConditionType::CwdNotContains => !ctx.cwd.contains(&condition.value),

        // -- Common: permission --
        ConditionType::PermissionModeIs => ctx.permission_mode == Some(condition.value.as_str()),

        // -- Tool-specific --
        ConditionType::FileExtension => check_file_extension(ctx.tool_input, &condition.value),
        ConditionType::CommandContains => check_command_contains(ctx.tool_input, &condition.value),
        ConditionType::CommandStartsWith => {
            check_command_starts_with(ctx.tool_input, &condition.value)
        }
        ConditionType::UrlStartsWith => check_url_starts_with(ctx.tool_input, &condition.value),
        ConditionType::GitTrackedFileOperation => {
            check_git_tracked_file_operation(ctx.cwd, ctx.tool_input, &condition.value)
        }

        // -- Not applicable to PreToolUse --
        ConditionType::PromptRegex | ConditionType::EveryNPrompts | ConditionType::ReasonIs => {
            false
        }
    }
}

/// Evaluate all conditions for a UserPromptSubmit hook.
pub(crate) fn evaluate_user_prompt_submit_conditions(
    conditions: &[Condition],
    ctx: &UserPromptSubmitConditionContext,
) -> bool {
    conditions
        .iter()
        .all(|c| evaluate_user_prompt_submit_condition(c, ctx))
}

/// Evaluate a single condition against the UserPromptSubmit context.
fn evaluate_user_prompt_submit_condition(
    condition: &Condition,
    ctx: &UserPromptSubmitConditionContext,
) -> bool {
    match condition.condition_type {
        // -- Common: filesystem --
        ConditionType::FileExists => check_file_exists(ctx.cwd, &condition.value),
        ConditionType::FileNotExists => !check_file_exists(ctx.cwd, &condition.value),
        ConditionType::FileExistsRecursive => {
            check_file_exists_recursive(ctx.cwd, &condition.value)
        }
        ConditionType::FileNotExistsRecursive => {
            !check_file_exists_recursive(ctx.cwd, &condition.value)
        }
        ConditionType::DirExists => check_dir_exists(ctx.cwd, &condition.value),
        ConditionType::DirNotExists => !check_dir_exists(ctx.cwd, &condition.value),
        ConditionType::DirExistsRecursive => check_dir_exists_recursive(ctx.cwd, &condition.value),
        ConditionType::DirNotExistsRecursive => {
            !check_dir_exists_recursive(ctx.cwd, &condition.value)
        }

        // -- Common: cwd --
        ConditionType::CwdIs => ctx.cwd == condition.value,
        ConditionType::CwdIsNot => ctx.cwd != condition.value,
        ConditionType::CwdContains => ctx.cwd.contains(&condition.value),
        ConditionType::CwdNotContains => !ctx.cwd.contains(&condition.value),

        // -- Common: permission (not available in UserPromptSubmit) --
        ConditionType::PermissionModeIs => false,

        // -- Tool-specific (not applicable to UserPromptSubmit) --
        ConditionType::FileExtension
        | ConditionType::CommandContains
        | ConditionType::CommandStartsWith
        | ConditionType::UrlStartsWith
        | ConditionType::GitTrackedFileOperation => false,

        // -- Prompt-specific --
        ConditionType::PromptRegex => check_prompt_regex(ctx.prompt, &condition.value),
        ConditionType::EveryNPrompts => {
            check_every_n_prompts(ctx.transcript_path, ctx.session_id, &condition.value)
        }

        // -- Not applicable to UserPromptSubmit --
        ConditionType::ReasonIs => false,
    }
}

// -- Prompt-specific helpers --

fn check_prompt_regex(prompt: &str, pattern: &str) -> bool {
    match Regex::new(pattern) {
        Ok(re) => re.is_match(prompt),
        Err(e) => {
            eprintln!("warning: invalid prompt_regex pattern '{}': {}", pattern, e);
            false
        }
    }
}

fn check_every_n_prompts(transcript_path: Option<&str>, session_id: &str, value: &str) -> bool {
    let n: usize = match value.parse() {
        Ok(v) if v > 0 => v,
        _ => {
            eprintln!("warning: invalid every_n_prompts value '{}'", value);
            return false;
        }
    };

    let transcript_path = match transcript_path {
        Some(path) => path,
        None => {
            eprintln!("warning: every_n_prompts requires transcript_path");
            return false;
        }
    };

    let count = match count_user_prompts_from_transcript(transcript_path, session_id) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("warning: failed to count prompts: {}", e);
            return false;
        }
    };

    count % n == 0
}

/// Count user prompts in the transcript file for the specified session.
/// Returns the count including the current prompt (count + 1).
fn count_user_prompts_from_transcript(
    transcript_path: &str,
    session_id: &str,
) -> Result<usize, std::io::Error> {
    let file = std::fs::File::open(transcript_path)?;
    let reader = BufReader::new(file);
    let mut count: usize = 0;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => continue,
        };
        let entry: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        if entry.get("type").and_then(|v| v.as_str()) == Some("user")
            && entry.get("sessionId").and_then(|v| v.as_str()) == Some(session_id)
        {
            count += 1;
        }
    }

    // Include the current prompt
    Ok(count + 1)
}

// -- Filesystem helpers --

fn check_file_exists(cwd: &str, name: &str) -> bool {
    let path = Path::new(cwd).join(name);
    path.is_file()
}

fn check_file_exists_recursive(cwd: &str, name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    for entry in WalkDir::new(cwd).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.file_name().to_string_lossy() == name {
            return true;
        }
    }
    false
}

fn check_dir_exists(cwd: &str, name: &str) -> bool {
    let path = Path::new(cwd).join(name);
    path.is_dir()
}

fn check_dir_exists_recursive(cwd: &str, name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    for entry in WalkDir::new(cwd).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_dir() {
            continue;
        }
        if entry.file_name().to_string_lossy() == name {
            return true;
        }
    }
    false
}

// -- Tool-specific helpers --

fn check_file_extension(tool_input: &HashMap<String, serde_json::Value>, ext: &str) -> bool {
    tool_input
        .get("file_path")
        .and_then(|v| v.as_str())
        .is_some_and(|fp| fp.ends_with(ext))
}

fn check_command_contains(tool_input: &HashMap<String, serde_json::Value>, value: &str) -> bool {
    tool_input
        .get("command")
        .and_then(|v| v.as_str())
        .is_some_and(|cmd| cmd.contains(value))
}

fn check_command_starts_with(tool_input: &HashMap<String, serde_json::Value>, value: &str) -> bool {
    tool_input
        .get("command")
        .and_then(|v| v.as_str())
        .is_some_and(|cmd| cmd.starts_with(value))
}

fn check_url_starts_with(tool_input: &HashMap<String, serde_json::Value>, value: &str) -> bool {
    tool_input
        .get("url")
        .and_then(|v| v.as_str())
        .is_some_and(|url| url.starts_with(value))
}

/// Check if a command operates on git-tracked files.
///
/// `value` is a pipe-separated list of commands to check (e.g. "rm|mv").
/// Returns true if the command in tool_input matches one of those and
/// any of its file arguments are tracked by git.
fn check_git_tracked_file_operation(
    cwd: &str,
    tool_input: &HashMap<String, serde_json::Value>,
    value: &str,
) -> bool {
    let command = match tool_input.get("command").and_then(|v| v.as_str()) {
        Some(cmd) => cmd,
        None => return false,
    };

    let args: Vec<&str> = command.split_whitespace().collect();
    if args.is_empty() {
        return false;
    }

    let cmd_name = args[0];
    let blocked_ops: Vec<&str> = value.split('|').map(|s| s.trim()).collect();

    if !blocked_ops.contains(&cmd_name) {
        return false;
    }

    // Extract file paths from arguments (skip flags starting with -)
    let file_args: Vec<&str> = args[1..]
        .iter()
        .filter(|a| !a.starts_with('-'))
        .copied()
        .collect();

    for file_arg in &file_args {
        if is_git_tracked(cwd, file_arg) {
            return true;
        }
    }

    false
}

/// Check if a file is tracked by git using `git ls-files`.
fn is_git_tracked(cwd: &str, file_path: &str) -> bool {
    Command::new("git")
        .args(["ls-files", "--error-unmatch", file_path])
        .current_dir(cwd)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

#[cfg(test)]
#[path = "condition_tests.rs"]
mod condition_tests;
