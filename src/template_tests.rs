use super::*;
use jaq_json::read;

fn parse_val(json: &str) -> Val {
    read::parse_single(json.as_bytes()).expect("test JSON should be valid")
}

// -- execute_jq_query tests --

#[test]
fn query_simple_field_access() {
    let input = parse_val(r#"{"tool_name": "Write", "tool_input": {"file_path": "test.go"}}"#);
    let result = execute_jq_query(".tool_name", &input).unwrap();
    assert_eq!(result, "Write");
}

#[test]
fn query_nested_field_access() {
    let input = parse_val(r#"{"tool_input": {"file_path": "src/main.rs"}}"#);
    let result = execute_jq_query(".tool_input.file_path", &input).unwrap();
    assert_eq!(result, "src/main.rs");
}

#[test]
fn query_number_value() {
    let input = parse_val(r#"{"count": 42}"#);
    let result = execute_jq_query(".count", &input).unwrap();
    assert_eq!(result, "42");
}

#[test]
fn query_boolean_true() {
    let input = parse_val(r#"{"active": true}"#);
    let result = execute_jq_query(".active", &input).unwrap();
    assert_eq!(result, "true");
}

#[test]
fn query_boolean_false() {
    let input = parse_val(r#"{"active": false}"#);
    let result = execute_jq_query(".active", &input).unwrap();
    assert_eq!(result, "false");
}

#[test]
fn query_null_value() {
    let input = parse_val(r#"{"existing": "value"}"#);
    let result = execute_jq_query(".missing", &input).unwrap();
    assert_eq!(result, "");
}

#[test]
fn query_deeply_nested_null() {
    let input = parse_val(r#"{"test": "value"}"#);
    let result = execute_jq_query(".missing.deeply.nested", &input).unwrap();
    assert_eq!(result, "");
}

#[test]
fn query_invalid_syntax() {
    let input = parse_val(r#"{"test": "value"}"#);
    let result = execute_jq_query(".invalid.[", &input);
    assert!(result.is_err());
}

#[test]
fn query_string_upcase() {
    let input = parse_val(r#"{"name": "hello"}"#);
    let result = execute_jq_query(".name | ascii_upcase", &input).unwrap();
    assert_eq!(result, "HELLO");
}

#[test]
fn query_array_length() {
    let input = parse_val(r#"{"tags": ["dev", "rust"]}"#);
    let result = execute_jq_query(".tags | length", &input).unwrap();
    assert_eq!(result, "2");
}

#[test]
fn query_array_index() {
    let input = parse_val(r#"{"tags": ["dev", "rust"]}"#);
    let result = execute_jq_query(".tags[0]", &input).unwrap();
    assert_eq!(result, "dev");
}

// -- template_replace tests --

#[test]
fn replace_simple_field() {
    let input = parse_val(r#"{"tool_name": "Write"}"#);
    let result = template_replace("echo {.tool_name}", &input);
    assert_eq!(result, "echo Write");
}

#[test]
fn replace_nested_field() {
    let input = parse_val(r#"{"tool_input": {"file_path": "src/main.rs"}, "tool_name": "Write"}"#);
    let result = template_replace("gofmt -w {.tool_input.file_path}", &input);
    assert_eq!(result, "gofmt -w src/main.rs");
}

#[test]
fn replace_multiple_templates() {
    let input = parse_val(
        r#"{"tool_name": "Write", "tool_input": {"file_path": "main.go"}, "cwd": "/home/user"}"#,
    );
    let result = template_replace("{.tool_name} on {.tool_input.file_path} in {.cwd}", &input);
    assert_eq!(result, "Write on main.go in /home/user");
}

#[test]
fn replace_no_templates() {
    let input = parse_val(r#"{"tool_name": "Write"}"#);
    let result = template_replace("plain text without templates", &input);
    assert_eq!(result, "plain text without templates");
}

#[test]
fn replace_empty_braces_not_treated_as_template() {
    let input = parse_val(r#"{"tool_name": "Write"}"#);
    let result = template_replace("empty {} braces", &input);
    assert_eq!(result, "empty {} braces");
}

#[test]
fn replace_nonexistent_field_gives_empty() {
    let input = parse_val(r#"{"tool_name": "Write"}"#);
    let result = template_replace("Missing: {.nonexistent}", &input);
    assert_eq!(result, "Missing: ");
}

#[test]
fn replace_invalid_query_gives_error_marker() {
    let input = parse_val(r#"{"tool_name": "Write"}"#);
    let result = template_replace("Error: {.invalid.[}", &input);
    assert!(result.starts_with("Error: [JQ_ERROR:"));
}

#[test]
fn replace_command_template_from_done_criteria() {
    // Done の定義: `command: "echo {.tool_name}"` が `echo Write` に置換されて実行される
    let input = parse_val(
        r#"{"session_id": "s1", "transcript_path": "/tmp/t.log", "cwd": "/home", "hook_event_name": "PreToolUse", "tool_name": "Write", "tool_input": {"file_path": "test.go"}}"#,
    );
    let result = template_replace("echo {.tool_name}", &input);
    assert_eq!(result, "echo Write");
}

#[test]
fn replace_nested_tool_input_from_done_criteria() {
    // Done の定義: `{.tool_input.file_path}` がネストした値に展開される
    let input = parse_val(
        r#"{"session_id": "s1", "transcript_path": "/tmp/t.log", "cwd": "/home", "hook_event_name": "PreToolUse", "tool_name": "Write", "tool_input": {"file_path": "src/lib.rs"}}"#,
    );
    let result = template_replace("{.tool_input.file_path}", &input);
    assert_eq!(result, "src/lib.rs");
}

#[test]
fn replace_with_pipe_expression() {
    let input = parse_val(r#"{"name": "test-user"}"#);
    let result = template_replace("Name: {.name | ascii_upcase}", &input);
    assert_eq!(result, "Name: TEST-USER");
}

#[test]
fn replace_template_in_message() {
    let input = parse_val(r#"{"tool_name": "Bash", "tool_input": {"command": "git add ."}}"#);
    let result = template_replace("Tool {.tool_name} ran: {.tool_input.command}", &input);
    assert_eq!(result, "Tool Bash ran: git add .");
}
