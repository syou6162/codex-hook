use clap::ValueEnum;
use serde::Deserialize;
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Clone, ValueEnum)]
#[value(rename_all = "PascalCase")]
pub enum HookEventType {
    PreToolUse,
    PostToolUse,
    SessionStart,
    UserPromptSubmit,
    Stop,
    SubagentStop,
    Notification,
    PreCompact,
}

impl fmt::Display for HookEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self
            .to_possible_value()
            .expect("all variants have a possible value")
            .get_name()
            .to_owned();
        write!(f, "{}", name)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to parse YAML: {0}")]
    Yaml(#[from] serde_saphyr::Error),
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ActionType {
    Command,
    Output,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct Action {
    #[serde(rename = "type")]
    pub action_type: ActionType,

    #[serde(default)]
    pub command: Option<String>,

    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub struct HookRule {
    pub matcher: String,

    #[serde(default)]
    pub actions: Vec<Action>,
}

#[derive(Debug, Deserialize, PartialEq, Default)]
#[serde(rename_all = "PascalCase")]
pub struct Config {
    #[serde(default)]
    pub pre_tool_use: Vec<HookRule>,

    #[serde(default)]
    pub post_tool_use: Vec<HookRule>,

    #[serde(default)]
    pub session_start: Vec<HookRule>,

    #[serde(default)]
    pub user_prompt_submit: Vec<HookRule>,

    #[serde(default)]
    pub stop: Vec<HookRule>,

    #[serde(default)]
    pub subagent_stop: Vec<HookRule>,

    #[serde(default)]
    pub notification: Vec<HookRule>,

    #[serde(default)]
    pub pre_compact: Vec<HookRule>,
}

pub fn default_config_path() -> PathBuf {
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

pub fn load_config(path: &Path) -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    let config: Config = serde_saphyr::from_str(&content)?;
    Ok(config)
}
