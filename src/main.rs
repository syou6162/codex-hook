use clap::{Parser, ValueEnum};
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
pub struct Action {
    #[serde(rename = "type")]
    pub action_type: String,

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
        .unwrap_or_else(|_| dirs_fallback_config_dir());
    config_dir.join("codex-hook").join("config.yaml")
}

fn dirs_fallback_config_dir() -> PathBuf {
    home_dir().join(".config")
}

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("~"))
}

pub fn load_config(path: &Path) -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    let config: Config = serde_saphyr::from_str(&content)?;
    Ok(config)
}

#[derive(Parser)]
#[command(about = "A hook tool for Codex CLI")]
struct Cli {
    #[arg(long)]
    event: HookEventType,

    #[arg(long)]
    config: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    println!("event: {}", cli.event);

    let config_path = cli
        .config
        .map(PathBuf::from)
        .unwrap_or_else(default_config_path);

    match load_config(&config_path) {
        Ok(config) => println!("config: {:#?}", config),
        Err(err) => eprintln!("error: {}", err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parse_event_flag() {
        let cli = Cli::parse_from(["codex-hook", "--event", "PreToolUse"]);
        assert!(matches!(cli.event, HookEventType::PreToolUse));
        assert!(cli.config.is_none());
    }

    #[test]
    fn parse_event_and_config_flags() {
        let cli = Cli::parse_from([
            "codex-hook",
            "--event",
            "PostToolUse",
            "--config",
            "/path/to/config.yaml",
        ]);
        assert!(matches!(cli.event, HookEventType::PostToolUse));
        assert_eq!(cli.config.as_deref(), Some("/path/to/config.yaml"));
    }

    #[test]
    fn reject_invalid_event() {
        let result = Cli::try_parse_from(["codex-hook", "--event", "Invalid"]);
        assert!(result.is_err());
    }

    #[test]
    fn reject_lowercase_event() {
        let result = Cli::try_parse_from(["codex-hook", "--event", "pretooluse"]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_all_event_types() {
        let events = [
            "PreToolUse",
            "PostToolUse",
            "SessionStart",
            "UserPromptSubmit",
            "Stop",
            "SubagentStop",
            "Notification",
            "PreCompact",
        ];
        for event in events {
            let cli = Cli::parse_from(["codex-hook", "--event", event]);
            assert_eq!(cli.event.to_string(), event);
        }
    }

    #[test]
    fn deserialize_yaml_string() {
        let yaml = r#"
PreToolUse:
  - matcher: "Write"
    actions:
      - type: output
        message: "Allowing write operation"
"#;
        let config: Config = serde_saphyr::from_str(yaml).unwrap();
        assert_eq!(config.pre_tool_use.len(), 1);
        assert_eq!(config.pre_tool_use[0].matcher, "Write");
        assert_eq!(config.pre_tool_use[0].actions.len(), 1);
        assert_eq!(config.pre_tool_use[0].actions[0].action_type, "output");
        assert_eq!(
            config.pre_tool_use[0].actions[0].message.as_deref(),
            Some("Allowing write operation")
        );
    }

    #[test]
    fn deserialize_multiple_event_types() {
        let yaml = r#"
PreToolUse:
  - matcher: "Bash"
    actions:
      - type: command
        command: "echo hello"
PostToolUse:
  - matcher: "Write|Edit"
    actions:
      - type: command
        command: "gofmt -w {.tool_input.file_path}"
"#;
        let config: Config = serde_saphyr::from_str(yaml).unwrap();
        assert_eq!(config.pre_tool_use.len(), 1);
        assert_eq!(config.post_tool_use.len(), 1);
        assert_eq!(config.pre_tool_use[0].matcher, "Bash");
        assert_eq!(
            config.post_tool_use[0].actions[0].command.as_deref(),
            Some("gofmt -w {.tool_input.file_path}")
        );
    }

    #[test]
    fn deserialize_empty_yaml() {
        let yaml = "";
        let config: Config = serde_saphyr::from_str(yaml).unwrap();
        assert_eq!(config, Config::default());
    }

    #[test]
    fn deserialize_partial_config() {
        let yaml = r#"
Stop:
  - matcher: ""
    actions:
      - type: command
        command: "ntfy publish 'done'"
"#;
        let config: Config = serde_saphyr::from_str(yaml).unwrap();
        assert!(config.pre_tool_use.is_empty());
        assert_eq!(config.stop.len(), 1);
    }

    #[test]
    fn load_config_from_file() {
        let dir = std::env::temp_dir().join("codex-hook-test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("test_config.yaml");
        let mut file = std::fs::File::create(&path).unwrap();
        writeln!(
            file,
            r#"PreToolUse:
  - matcher: "Write"
    actions:
      - type: output
        message: "test from file""#
        )
        .unwrap();

        let config = load_config(&path).unwrap();
        assert_eq!(config.pre_tool_use.len(), 1);
        assert_eq!(config.pre_tool_use[0].matcher, "Write");

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn load_config_file_not_found() {
        let result = load_config(Path::new("/nonexistent/config.yaml"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ConfigError::Io(_)));
    }

    #[test]
    fn load_config_invalid_yaml() {
        let dir = std::env::temp_dir().join("codex-hook-test-invalid");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("invalid.yaml");
        std::fs::write(&path, "PreToolUse: [[[invalid").unwrap();

        let result = load_config(&path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ConfigError::Yaml(_)));

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn default_config_path_uses_home() {
        std::env::remove_var("XDG_CONFIG_HOME");
        let path = default_config_path();
        assert!(path
            .to_string_lossy()
            .ends_with(".config/codex-hook/config.yaml"));
    }

    #[test]
    fn default_config_path_uses_xdg() {
        std::env::set_var("XDG_CONFIG_HOME", "/tmp/xdg-test");
        let path = default_config_path();
        assert_eq!(path, PathBuf::from("/tmp/xdg-test/codex-hook/config.yaml"));
        std::env::remove_var("XDG_CONFIG_HOME");
    }
}
