//! YAML config schema compatible with cchook (Go).
//!
//! This is a separate layer from Codex's own `settings.json` format.
//! Codex invokes codex-hook as a shell hook; codex-hook reads this YAML config,
//! evaluates conditions, executes actions, and returns JSON to stdout.
//!
//! References:
//! - Codex hooks: <https://docs.anthropic.com/en/docs/claude-code/hooks>
//! - cchook (Go): <https://github.com/syou6162/cchook>

use crate::error::ConfigError;
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Condition types that evaluate fields from the JSON input Codex passes to hooks.
///
/// 21 variants matching cchook. Each variant maps to a specific Codex input field.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ConditionType {
    // -- Common: based on BaseInput.cwd / filesystem --
    FileExists,
    FileExistsRecursive,
    FileNotExists,
    FileNotExistsRecursive,
    DirExists,
    DirExistsRecursive,
    DirNotExists,
    DirNotExistsRecursive,
    CwdIs,
    CwdIsNot,
    CwdContains,
    CwdNotContains,
    PermissionModeIs,

    // -- Tool-specific: based on tool_input (PreToolUse/PostToolUse) --
    FileExtension,
    CommandContains,
    CommandStartsWith,
    UrlStartsWith,
    GitTrackedFileOperation,

    // -- Prompt-specific: based on UserPromptSubmitInput.prompt --
    PromptRegex,
    EveryNPrompts,

    // -- Session-specific: based on SessionEndInput.reason --
    ReasonIs,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct Condition {
    #[serde(rename = "type")]
    pub(crate) condition_type: ConditionType,

    pub(crate) value: String,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ActionType {
    Command,
    Output,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct Action {
    #[serde(rename = "type")]
    pub(crate) action_type: ActionType,

    #[serde(default)]
    pub(crate) command: Option<String>,

    #[serde(default)]
    pub(crate) message: Option<String>,

    #[serde(default)]
    pub(crate) exit_status: Option<i32>,
}

// -- Hooks with required matcher --

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct PreToolUseHook {
    pub(crate) matcher: String,

    #[serde(default)]
    pub(crate) conditions: Vec<Condition>,

    #[serde(default)]
    pub(crate) actions: Vec<Action>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct PostToolUseHook {
    pub(crate) matcher: String,

    #[serde(default)]
    pub(crate) conditions: Vec<Condition>,

    #[serde(default)]
    pub(crate) actions: Vec<Action>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct PermissionRequestHook {
    pub(crate) matcher: String,

    #[serde(default)]
    pub(crate) conditions: Vec<Condition>,

    #[serde(default)]
    pub(crate) actions: Vec<Action>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct SessionStartHook {
    pub(crate) matcher: String,

    #[serde(default)]
    pub(crate) conditions: Vec<Condition>,

    #[serde(default)]
    pub(crate) actions: Vec<Action>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct PreCompactHook {
    pub(crate) matcher: String,

    #[serde(default)]
    pub(crate) conditions: Vec<Condition>,

    #[serde(default)]
    pub(crate) actions: Vec<Action>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct SubagentStartHook {
    pub(crate) matcher: String,

    #[serde(default)]
    pub(crate) conditions: Vec<Condition>,

    #[serde(default)]
    pub(crate) actions: Vec<Action>,
}

// -- Hooks with optional matcher --

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct NotificationHook {
    #[serde(default)]
    pub(crate) matcher: Option<String>,

    #[serde(default)]
    pub(crate) conditions: Vec<Condition>,

    #[serde(default)]
    pub(crate) actions: Vec<Action>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct SubagentStopHook {
    #[serde(default)]
    pub(crate) matcher: Option<String>,

    #[serde(default)]
    pub(crate) conditions: Vec<Condition>,

    #[serde(default)]
    pub(crate) actions: Vec<Action>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct SessionEndHook {
    #[serde(default)]
    pub(crate) matcher: Option<String>,

    #[serde(default)]
    pub(crate) conditions: Vec<Condition>,

    #[serde(default)]
    pub(crate) actions: Vec<Action>,
}

// -- Hooks without matcher --

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct StopHook {
    #[serde(default)]
    pub(crate) conditions: Vec<Condition>,

    #[serde(default)]
    pub(crate) actions: Vec<Action>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct UserPromptSubmitHook {
    #[serde(default)]
    pub(crate) conditions: Vec<Condition>,

    #[serde(default)]
    pub(crate) actions: Vec<Action>,
}

/// Top-level config covering all 11 event types supported by cchook.
#[derive(Debug, Deserialize, PartialEq, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct Config {
    #[serde(default)]
    pub(crate) pre_tool_use: Vec<PreToolUseHook>,

    #[serde(default)]
    pub(crate) post_tool_use: Vec<PostToolUseHook>,

    #[serde(default)]
    pub(crate) permission_request: Vec<PermissionRequestHook>,

    #[serde(default)]
    pub(crate) notification: Vec<NotificationHook>,

    #[serde(default)]
    pub(crate) stop: Vec<StopHook>,

    #[serde(default)]
    pub(crate) subagent_stop: Vec<SubagentStopHook>,

    #[serde(default)]
    pub(crate) subagent_start: Vec<SubagentStartHook>,

    #[serde(default)]
    pub(crate) pre_compact: Vec<PreCompactHook>,

    #[serde(default)]
    pub(crate) session_start: Vec<SessionStartHook>,

    #[serde(default)]
    pub(crate) session_end: Vec<SessionEndHook>,

    #[serde(default)]
    pub(crate) user_prompt_submit: Vec<UserPromptSubmitHook>,
}

pub(crate) fn default_config_path() -> PathBuf {
    let config_dir = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home_dir().join(".config"));
    config_dir.join("codex-hook").join("config.yaml")
}

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .expect("HOME environment variable is not set")
}

pub(crate) fn load_config(path: &Path) -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    let config: Config = serde_saphyr::from_str(&content)?;
    Ok(config)
}

pub(crate) fn load_config_or_default(path: &Path) -> Result<Config, ConfigError> {
    match std::fs::read_to_string(path) {
        Ok(content) => Ok(serde_saphyr::from_str(&content)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
        Err(e) => Err(ConfigError::Io(e)),
    }
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod config_tests;
