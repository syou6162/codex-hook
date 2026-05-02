use crate::error::ConfigError;
use serde::Deserialize;
use std::path::{Path, PathBuf};

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
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct HookRule {
    pub(crate) matcher: String,

    #[serde(default)]
    pub(crate) actions: Vec<Action>,
}

#[derive(Debug, Deserialize, PartialEq, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct Config {
    #[serde(default)]
    pub(crate) pre_tool_use: Vec<HookRule>,

    #[serde(default)]
    pub(crate) post_tool_use: Vec<HookRule>,

    #[serde(default)]
    pub(crate) session_start: Vec<HookRule>,

    #[serde(default)]
    pub(crate) user_prompt_submit: Vec<HookRule>,

    #[serde(default)]
    pub(crate) stop: Vec<HookRule>,

    #[serde(default)]
    pub(crate) subagent_stop: Vec<HookRule>,

    #[serde(default)]
    pub(crate) notification: Vec<HookRule>,

    #[serde(default)]
    pub(crate) pre_compact: Vec<HookRule>,
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
