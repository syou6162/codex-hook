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
mod tests {
    use super::*;
    use std::io::Write;

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
        assert_eq!(
            config.pre_tool_use[0].actions[0].action_type,
            ActionType::Output
        );
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
        let dir = std::env::temp_dir().join("codex-hook-test-config");
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
    fn load_config_or_default_file_not_found() {
        let result = load_config_or_default(Path::new("/nonexistent/config.yaml"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Config::default());
    }

    #[test]
    fn load_config_or_default_invalid_yaml() {
        let dir = std::env::temp_dir().join("codex-hook-test-or-default-invalid");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("invalid.yaml");
        std::fs::write(&path, "PreToolUse: [[[invalid").unwrap();

        let result = load_config_or_default(&path);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::Yaml(_)));

        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn default_config_path_resolution() {
        // XDG_CONFIG_HOME が設定されている場合
        std::env::set_var("XDG_CONFIG_HOME", "/tmp/xdg-test");
        let path = default_config_path();
        assert_eq!(path, PathBuf::from("/tmp/xdg-test/codex-hook/config.yaml"));

        // XDG_CONFIG_HOME が未設定の場合は $HOME/.config にフォールバック
        std::env::remove_var("XDG_CONFIG_HOME");
        let path = default_config_path();
        assert!(path
            .to_string_lossy()
            .ends_with(".config/codex-hook/config.yaml"));
    }
}
