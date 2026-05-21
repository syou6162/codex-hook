use super::*;

#[test]
fn deserialize_pre_tool_use_input() {
    let json = r#"{
        "session_id": "test-session",
        "transcript_path": "/tmp/transcript.log",
        "cwd": "/home/user/project",
        "hook_event_name": "PreToolUse",
        "turn_id": "turn-1",
        "model": "gpt-test",
        "permission_mode": "default",
        "tool_name": "Write",
        "tool_input": {"file_path": "test.go"},
        "tool_use_id": "call-1"
    }"#;
    let input: PreToolUseInput = serde_json::from_str(json).unwrap();
    assert_eq!(input.base.session_id, "test-session");
    assert_eq!(
        input.base.transcript_path.as_deref(),
        Some("/tmp/transcript.log")
    );
    assert_eq!(input.base.cwd, "/home/user/project");
    assert_eq!(input.base.hook_event_name, "PreToolUse");
    assert_eq!(input.base.turn_id.as_deref(), Some("turn-1"));
    assert_eq!(input.base.model.as_deref(), Some("gpt-test"));
    assert_eq!(input.base.permission_mode.as_deref(), Some("default"));
    assert_eq!(input.tool_name, "Write");
    assert_eq!(input.tool_use_id.as_deref(), Some("call-1"));
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

#[test]
fn deserialize_transcript_path_null() {
    let json = r#"{
        "session_id": "s1",
        "transcript_path": null,
        "cwd": "/home/user",
        "hook_event_name": "PreToolUse",
        "tool_name": "Bash",
        "tool_input": {"command": "ls"}
    }"#;
    let input: PreToolUseInput = serde_json::from_str(json).unwrap();
    assert_eq!(input.base.transcript_path, None);
}

// -- PostToolUseInput tests --

#[test]
fn deserialize_post_tool_use_input() {
    let json = r#"{
        "session_id": "test-session",
        "turn_id": "turn-1",
        "transcript_path": null,
        "cwd": "/home/user/project",
        "hook_event_name": "PostToolUse",
        "model": "gpt-test",
        "permission_mode": "default",
        "tool_name": "Bash",
        "tool_input": {"command": "echo hello"},
        "tool_response": {"exit_code": 0, "stdout": "hello\n"},
        "tool_use_id": "call-123"
    }"#;
    let input: PostToolUseInput = serde_json::from_str(json).unwrap();
    assert_eq!(input.base.session_id, "test-session");
    assert_eq!(input.base.turn_id.as_deref(), Some("turn-1"));
    assert_eq!(input.base.transcript_path, None);
    assert_eq!(input.base.cwd, "/home/user/project");
    assert_eq!(input.base.hook_event_name, "PostToolUse");
    assert_eq!(input.base.model.as_deref(), Some("gpt-test"));
    assert_eq!(input.base.permission_mode.as_deref(), Some("default"));
    assert_eq!(input.tool_name, "Bash");
    assert_eq!(
        input.tool_input.get("command").and_then(|v| v.as_str()),
        Some("echo hello")
    );
    assert_eq!(
        input
            .tool_response
            .get("exit_code")
            .and_then(|v| v.as_i64()),
        Some(0)
    );
    assert_eq!(input.tool_use_id.as_deref(), Some("call-123"));
}

#[test]
fn reject_post_tool_use_missing_tool_response() {
    let json = r#"{
        "session_id": "s1",
        "transcript_path": null,
        "cwd": "/home/user",
        "hook_event_name": "PostToolUse",
        "tool_name": "Bash",
        "tool_input": {"command": "ls"}
    }"#;
    let result: Result<PostToolUseInput, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

#[test]
fn read_post_tool_use_input_with_raw_roundtrip() {
    let json = r#"{"session_id":"test","turn_id":"turn-1","transcript_path":null,"cwd":"/home","hook_event_name":"PostToolUse","model":"gpt-test","permission_mode":"default","tool_name":"Write","tool_input":{"file_path":"test.go"},"tool_response":{"ok":true},"tool_use_id":"call-1"}"#;
    let (input, raw) = read_post_tool_use_input_with_raw(json.as_bytes()).unwrap();
    assert_eq!(input.tool_name, "Write");
    assert_eq!(
        input.tool_input.get("file_path").and_then(|v| v.as_str()),
        Some("test.go")
    );
    assert_eq!(
        input.tool_response.get("ok").and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(raw, json.as_bytes());
}

// -- UserPromptSubmitInput tests --

#[test]
fn deserialize_user_prompt_submit_input() {
    let json = r#"{
        "session_id": "test-session",
        "transcript_path": "/tmp/transcript.log",
        "cwd": "/home/user/project",
        "hook_event_name": "UserPromptSubmit",
        "turn_id": "turn-1",
        "model": "gpt-test",
        "permission_mode": "default",
        "prompt": "fix the bug in main.rs"
    }"#;
    let input: UserPromptSubmitInput = serde_json::from_str(json).unwrap();
    assert_eq!(input.base.session_id, "test-session");
    assert_eq!(
        input.base.transcript_path.as_deref(),
        Some("/tmp/transcript.log")
    );
    assert_eq!(input.base.cwd, "/home/user/project");
    assert_eq!(input.base.hook_event_name, "UserPromptSubmit");
    assert_eq!(input.base.turn_id.as_deref(), Some("turn-1"));
    assert_eq!(input.base.model.as_deref(), Some("gpt-test"));
    assert_eq!(input.base.permission_mode.as_deref(), Some("default"));
    assert_eq!(input.prompt, "fix the bug in main.rs");
}

#[test]
fn deserialize_user_prompt_submit_empty_prompt() {
    let json = r#"{
        "session_id": "s1",
        "transcript_path": "/tmp/t.log",
        "cwd": "/home/user",
        "hook_event_name": "UserPromptSubmit",
        "prompt": ""
    }"#;
    let input: UserPromptSubmitInput = serde_json::from_str(json).unwrap();
    assert_eq!(input.prompt, "");
}

#[test]
fn reject_user_prompt_submit_missing_prompt() {
    let json = r#"{
        "session_id": "s1",
        "transcript_path": "/tmp/t.log",
        "cwd": "/home/user",
        "hook_event_name": "UserPromptSubmit"
    }"#;
    let result: Result<UserPromptSubmitInput, _> = serde_json::from_str(json);
    assert!(result.is_err());
}

#[test]
fn user_prompt_submit_ignores_extra_fields() {
    let json = r#"{
        "session_id": "s1",
        "transcript_path": "/tmp/t.log",
        "cwd": "/home/user",
        "hook_event_name": "UserPromptSubmit",
        "prompt": "hello",
        "extra_field": 42
    }"#;
    let input: UserPromptSubmitInput = serde_json::from_str(json).unwrap();
    assert_eq!(input.prompt, "hello");
}

#[test]
fn read_user_prompt_submit_input_with_raw_roundtrip() {
    let json = r#"{"session_id":"test","transcript_path":"/tmp/t.log","cwd":"/home","hook_event_name":"UserPromptSubmit","prompt":"do something"}"#;
    let (input, raw) = read_user_prompt_submit_input_with_raw(json.as_bytes()).unwrap();
    assert_eq!(input.prompt, "do something");
    assert_eq!(raw, json.as_bytes());
}
