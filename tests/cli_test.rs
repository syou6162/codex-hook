use std::process::Command;

fn codex_hook_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_codex-hook"))
}

#[test]
fn parse_event_flag() {
    let output = codex_hook_bin()
        .args(["--event", "PreToolUse", "--config", "/nonexistent"])
        .output()
        .expect("failed to run codex-hook");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("event: PreToolUse"));
}

#[test]
fn reject_invalid_event() {
    let output = codex_hook_bin()
        .args(["--event", "Invalid"])
        .output()
        .expect("failed to run codex-hook");
    assert!(!output.status.success());
}

#[test]
fn reject_lowercase_event() {
    let output = codex_hook_bin()
        .args(["--event", "pretooluse"])
        .output()
        .expect("failed to run codex-hook");
    assert!(!output.status.success());
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
        let output = codex_hook_bin()
            .args(["--event", event, "--config", "/nonexistent"])
            .output()
            .expect("failed to run codex-hook");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            stdout.contains(&format!("event: {}", event)),
            "expected 'event: {}' in stdout: {}",
            event,
            stdout
        );
    }
}
