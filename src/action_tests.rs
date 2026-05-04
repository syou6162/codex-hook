use super::*;
use serde_json::Value;

#[test]
fn execute_echo_command() {
    let status = execute_command("echo test").unwrap();
    assert!(status.success());
}

#[test]
fn execute_command_exit_code_nonzero() {
    let status = execute_command("exit 1").unwrap();
    assert!(!status.success());
    assert_eq!(status.code(), Some(1));
}

#[test]
fn execute_command_exit_code_zero() {
    let status = execute_command("true").unwrap();
    assert_eq!(status.code(), Some(0));
}

#[test]
fn execute_command_nonexistent_binary() {
    let status = execute_command("nonexistent_command_xyz_12345").unwrap();
    assert!(!status.success());
}

#[test]
fn execute_command_with_pipe() {
    let status = execute_command("echo hello | cat").unwrap();
    assert!(status.success());
}

#[test]
fn output_message_json_format() {
    let json = build_output_json("hello");
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["message"], "hello");
}

#[test]
fn output_message_with_special_characters() {
    let json = build_output_json("line1\nline2\ttab \"quoted\"");
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["message"], "line1\nline2\ttab \"quoted\"");
}

#[test]
fn output_message_empty_string() {
    let json = build_output_json("");
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["message"], "");
}

#[test]
fn output_message_has_only_message_field() {
    let json = build_output_json("test");
    let parsed: Value = serde_json::from_str(&json).unwrap();
    let obj = parsed.as_object().unwrap();
    assert_eq!(obj.len(), 1);
    assert!(obj.contains_key("message"));
}

#[test]
fn output_message_merged_multiple_messages() {
    let messages = vec!["msg1".to_string(), "msg2".to_string(), "msg3".to_string()];
    let merged = messages.join("\n");
    let json = build_output_json(&merged);
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["message"], "msg1\nmsg2\nmsg3");
}

#[test]
fn output_message_merged_single_message() {
    let messages = vec!["only one".to_string()];
    let merged = messages.join("\n");
    let json = build_output_json(&merged);
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["message"], "only one");
}

#[test]
fn output_message_merged_produces_single_json_object() {
    let messages = vec!["first".to_string(), "second".to_string()];
    let merged = messages.join("\n");
    let json = build_output_json(&merged);
    // Must be parseable as a single JSON object (Codex requirement)
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert!(parsed.is_object());
    let obj = parsed.as_object().unwrap();
    assert_eq!(obj.len(), 1);
    assert!(obj.contains_key("message"));
}
