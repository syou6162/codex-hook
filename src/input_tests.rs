use super::*;

#[test]
fn deserialize_pre_tool_use_input() {
    let json = r#"{
        "session_id": "test-session",
        "transcript_path": "/tmp/transcript.log",
        "cwd": "/home/user/project",
        "hook_event_name": "PreToolUse",
        "tool_name": "Write",
        "tool_input": {"file_path": "test.go"}
    }"#;
    let input: PreToolUseInput = serde_json::from_str(json).unwrap();
    assert_eq!(input.base.session_id, "test-session");
    assert_eq!(input.base.transcript_path, "/tmp/transcript.log");
    assert_eq!(input.base.cwd, "/home/user/project");
    assert_eq!(input.base.hook_event_name, "PreToolUse");
    assert_eq!(input.tool_name, "Write");
    assert_eq!(
        input.tool_input.get("file_path").and_then(|v| v.as_str()),
        Some("test.go")
    );
}

#[test]
fn deserialize_bash_tool_input() {
    let json = r#"{
        "session_id": "s1",
        "transcript_path": "/tmp/t.log",
        "cwd": "/home/user",
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": {"command": "echo hello"}
    }"#;
    let input: PreToolUseInput = serde_json::from_str(json).unwrap();
    assert_eq!(input.tool_name, "Bash");
    assert_eq!(
        input.tool_input.get("command").and_then(|v| v.as_str()),
        Some("echo hello")
    );
}

#[test]
fn deserialize_empty_tool_input() {
    let json = r#"{
        "session_id": "s1",
        "transcript_path": "/tmp/t.log",
        "cwd": "/home/user",
        "hook_event_name": "PreToolUse",
        "tool_name": "Read",
        "tool_input": {}
    }"#;
    let input: PreToolUseInput = serde_json::from_str(json).unwrap();
    assert_eq!(input.tool_name, "Read");
    assert!(input.tool_input.is_empty());
}

#[test]
fn deserialize_tool_input_with_unknown_fields() {
    let json = r#"{
        "session_id": "s1",
        "transcript_path": "/tmp/t.log",
        "cwd": "/home/user",
        "hook_event_name": "PreToolUse",
        "tool_name": "WebFetch",
        "tool_input": {"url": "https://example.com", "prompt": "summarize", "extra_field": 42}
    }"#;
    let input: PreToolUseInput = serde_json::from_str(json).unwrap();
    assert_eq!(input.tool_name, "WebFetch");
    assert_eq!(
        input.tool_input.get("url").and_then(|v| v.as_str()),
        Some("https://example.com")
    );
    assert_eq!(
        input.tool_input.get("extra_field").and_then(|v| v.as_i64()),
        Some(42)
    );
}

#[test]
fn reject_missing_required_field() {
    let json = r#"{
        "session_id": "s1",
        "hook_event_name": "PreToolUse",
        "tool_name": "Write",
        "tool_input": {}
    }"#;
    let result: Result<PreToolUseInput, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

#[test]
fn reject_invalid_json() {
    let json = r#"{ not valid json }"#;
    let result: Result<PreToolUseInput, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

#[test]
fn read_pre_tool_use_input_from_reader() {
    let json = r#"{"session_id":"test","transcript_path":"/tmp/t.log","cwd":"/home","hook_event_name":"PreToolUse","tool_name":"Write","tool_input":{"file_path":"test.go"}}"#;
    let input = read_pre_tool_use_input(json.as_bytes()).unwrap();
    assert_eq!(input.tool_name, "Write");
    assert_eq!(
        input.tool_input.get("file_path").and_then(|v| v.as_str()),
        Some("test.go")
    );
}

#[test]
fn ignore_extra_top_level_fields() {
    let json = r#"{
        "session_id": "s1",
        "transcript_path": "/tmp/t.log",
        "cwd": "/home/user",
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": {"command": "ls"},
        "permission_mode": "default",
        "tool_use_id": "tu_123"
    }"#;
    let input: PreToolUseInput = serde_json::from_str(json).unwrap();
    assert_eq!(input.tool_name, "Bash");
}
