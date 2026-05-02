use super::*;

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
        "PermissionRequest",
        "Notification",
        "Stop",
        "SubagentStop",
        "SubagentStart",
        "PreCompact",
        "SessionStart",
        "SessionEnd",
        "UserPromptSubmit",
    ];
    for event in events {
        let cli = Cli::parse_from(["codex-hook", "--event", event]);
        assert_eq!(cli.event.to_string(), event);
    }
}
