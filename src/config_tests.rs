use super::*;
use crate::error::ConfigError;
use std::io::Write;
use std::path::Path;

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
  - actions:
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
fn deserialize_pre_tool_use_with_conditions() {
    let yaml = r#"
PreToolUse:
  - matcher: "Write|Edit"
    conditions:
      - type: file_extension
        value: ".rs"
      - type: cwd_contains
        value: "src"
    actions:
      - type: command
        command: "echo 'hook fired'"
"#;
    let config: Config = serde_saphyr::from_str(yaml).unwrap();
    assert_eq!(config.pre_tool_use.len(), 1);

    let hook = &config.pre_tool_use[0];
    assert_eq!(hook.matcher, "Write|Edit");
    assert_eq!(hook.conditions.len(), 2);
    assert_eq!(
        hook.conditions[0].condition_type,
        ConditionType::FileExtension
    );
    assert_eq!(hook.conditions[0].value, ".rs");
    assert_eq!(
        hook.conditions[1].condition_type,
        ConditionType::CwdContains
    );
    assert_eq!(hook.conditions[1].value, "src");
    assert_eq!(hook.actions.len(), 1);
}

#[test]
fn deserialize_action_with_exit_status() {
    let yaml = r#"
PreToolUse:
  - matcher: "Bash"
    conditions:
      - type: dir_exists
        value: "build"
    actions:
      - type: output
        message: "Build directory already exists."
        exit_status: 1
"#;
    let config: Config = serde_saphyr::from_str(yaml).unwrap();
    let action = &config.pre_tool_use[0].actions[0];
    assert_eq!(action.action_type, ActionType::Output);
    assert_eq!(
        action.message.as_deref(),
        Some("Build directory already exists.")
    );
    assert_eq!(action.exit_status, Some(1));
}

#[test]
fn deserialize_action_without_exit_status() {
    let yaml = r#"
PreToolUse:
  - matcher: "Write"
    actions:
      - type: command
        command: "echo hello"
"#;
    let config: Config = serde_saphyr::from_str(yaml).unwrap();
    assert_eq!(config.pre_tool_use[0].actions[0].exit_status, None);
}

#[test]
fn deserialize_stop_hook_without_matcher() {
    let yaml = r#"
Stop:
  - actions:
      - type: command
        command: "ntfy publish 'session stopped'"
"#;
    let config: Config = serde_saphyr::from_str(yaml).unwrap();
    assert_eq!(config.stop.len(), 1);
    assert_eq!(config.stop[0].actions.len(), 1);
    assert_eq!(
        config.stop[0].actions[0].command.as_deref(),
        Some("ntfy publish 'session stopped'")
    );
}

#[test]
fn deserialize_user_prompt_submit_hook_without_matcher() {
    let yaml = r#"
UserPromptSubmit:
  - conditions:
      - type: prompt_regex
        value: "delete|rm -rf"
    actions:
      - type: output
        message: "Dangerous command detected"
"#;
    let config: Config = serde_saphyr::from_str(yaml).unwrap();
    assert_eq!(config.user_prompt_submit.len(), 1);

    let hook = &config.user_prompt_submit[0];
    assert_eq!(hook.conditions.len(), 1);
    assert_eq!(
        hook.conditions[0].condition_type,
        ConditionType::PromptRegex
    );
    assert_eq!(hook.conditions[0].value, "delete|rm -rf");
}

#[test]
fn deserialize_invalid_condition_type() {
    let yaml = r#"
PreToolUse:
  - matcher: "Write"
    conditions:
      - type: nonexistent_condition
        value: "test"
    actions:
      - type: output
        message: "test"
"#;
    let result: Result<Config, _> = serde_saphyr::from_str(yaml);
    assert!(result.is_err());
}

#[test]
fn deserialize_notification_hook_without_matcher() {
    let yaml = r#"
Notification:
  - actions:
      - type: command
        command: "notify-send 'alert'"
"#;
    let config: Config = serde_saphyr::from_str(yaml).unwrap();
    assert_eq!(config.notification.len(), 1);
    assert_eq!(config.notification[0].matcher, None);
}

#[test]
fn deserialize_notification_hook_with_matcher() {
    let yaml = r#"
Notification:
  - matcher: "permission_prompt"
    actions:
      - type: command
        command: "notify-send 'permission needed'"
"#;
    let config: Config = serde_saphyr::from_str(yaml).unwrap();
    assert_eq!(
        config.notification[0].matcher.as_deref(),
        Some("permission_prompt")
    );
}

#[test]
fn deserialize_cchook_readme_example() {
    let yaml = r#"
PreToolUse:
  - matcher: "Bash"
    conditions:
      - type: dir_exists
        value: "build"
      - type: command_starts_with
        value: "make"
    actions:
      - type: output
        message: "Build directory already exists. Run 'make clean' first."
        exit_status: 1
PostToolUse:
  - matcher: "Write"
    conditions:
      - type: file_extension
        value: ".go"
      - type: file_not_exists_recursive
        value: "main_test.go"
    actions:
      - type: output
        message: "Consider adding tests"
"#;
    let config: Config = serde_saphyr::from_str(yaml).unwrap();

    assert_eq!(config.pre_tool_use.len(), 1);
    assert_eq!(config.pre_tool_use[0].conditions.len(), 2);
    assert_eq!(
        config.pre_tool_use[0].conditions[0].condition_type,
        ConditionType::DirExists
    );
    assert_eq!(
        config.pre_tool_use[0].conditions[1].condition_type,
        ConditionType::CommandStartsWith
    );
    assert_eq!(config.pre_tool_use[0].actions[0].exit_status, Some(1));

    assert_eq!(config.post_tool_use.len(), 1);
    assert_eq!(config.post_tool_use[0].conditions.len(), 2);
    assert_eq!(
        config.post_tool_use[0].conditions[0].condition_type,
        ConditionType::FileExtension
    );
    assert_eq!(
        config.post_tool_use[0].conditions[1].condition_type,
        ConditionType::FileNotExistsRecursive
    );
}

#[test]
fn deserialize_hook_without_conditions() {
    let yaml = r#"
PreToolUse:
  - matcher: "Write"
    actions:
      - type: output
        message: "hello"
"#;
    let config: Config = serde_saphyr::from_str(yaml).unwrap();
    assert!(config.pre_tool_use[0].conditions.is_empty());
}

#[test]
fn default_config_path_resolution() {
    std::env::set_var("XDG_CONFIG_HOME", "/tmp/xdg-test");
    let path = default_config_path();
    assert_eq!(path, PathBuf::from("/tmp/xdg-test/codex-hook/config.yaml"));

    std::env::remove_var("XDG_CONFIG_HOME");
    let path = default_config_path();
    assert!(path
        .to_string_lossy()
        .ends_with(".config/codex-hook/config.yaml"));
}
