use super::*;
use std::collections::HashMap;
use std::fs;

fn make_ctx<'a>(
    cwd: &'a str,
    tool_input: &'a HashMap<String, serde_json::Value>,
    permission_mode: Option<&'a str>,
) -> ConditionContext<'a> {
    ConditionContext {
        cwd,
        tool_input,
        permission_mode,
    }
}

fn make_condition(condition_type: ConditionType, value: &str) -> Condition {
    Condition {
        condition_type,
        value: value.to_string(),
    }
}

// -- evaluate_conditions tests --

#[test]
fn empty_conditions_returns_true() {
    let tool_input = HashMap::new();
    let ctx = make_ctx("/tmp", &tool_input, None);
    assert!(evaluate_conditions(&[], &ctx));
}

#[test]
fn all_conditions_must_pass() {
    let dir = tempdir("all-pass");
    fs::write(dir.join("test.rs"), "").unwrap();

    let tool_input = HashMap::new();
    let ctx = make_ctx(dir.to_str().unwrap(), &tool_input, None);

    let conditions = vec![
        make_condition(ConditionType::FileExists, "test.rs"),
        make_condition(ConditionType::CwdContains, "all-pass"),
    ];
    assert!(evaluate_conditions(&conditions, &ctx));
    cleanup(&dir);
}

#[test]
fn one_false_condition_fails_all() {
    let dir = tempdir("one-false");
    fs::write(dir.join("test.rs"), "").unwrap();

    let tool_input = HashMap::new();
    let ctx = make_ctx(dir.to_str().unwrap(), &tool_input, None);

    let conditions = vec![
        make_condition(ConditionType::FileExists, "test.rs"),
        make_condition(ConditionType::FileExists, "nonexistent.rs"),
    ];
    assert!(!evaluate_conditions(&conditions, &ctx));
    cleanup(&dir);
}

// -- FileExists / FileNotExists --

#[test]
fn file_exists_true() {
    let dir = tempdir("fe-true");
    fs::write(dir.join("hello.txt"), "").unwrap();

    let tool_input = HashMap::new();
    let ctx = make_ctx(dir.to_str().unwrap(), &tool_input, None);
    let cond = make_condition(ConditionType::FileExists, "hello.txt");
    assert!(evaluate_condition(&cond, &ctx));
    cleanup(&dir);
}

#[test]
fn file_exists_false() {
    let dir = tempdir("fe-false");

    let tool_input = HashMap::new();
    let ctx = make_ctx(dir.to_str().unwrap(), &tool_input, None);
    let cond = make_condition(ConditionType::FileExists, "nope.txt");
    assert!(!evaluate_condition(&cond, &ctx));
    cleanup(&dir);
}

#[test]
fn file_not_exists_true() {
    let dir = tempdir("fne-true");

    let tool_input = HashMap::new();
    let ctx = make_ctx(dir.to_str().unwrap(), &tool_input, None);
    let cond = make_condition(ConditionType::FileNotExists, "nope.txt");
    assert!(evaluate_condition(&cond, &ctx));
    cleanup(&dir);
}

#[test]
fn file_not_exists_false() {
    let dir = tempdir("fne-false");
    fs::write(dir.join("exists.txt"), "").unwrap();

    let tool_input = HashMap::new();
    let ctx = make_ctx(dir.to_str().unwrap(), &tool_input, None);
    let cond = make_condition(ConditionType::FileNotExists, "exists.txt");
    assert!(!evaluate_condition(&cond, &ctx));
    cleanup(&dir);
}

// -- FileExistsRecursive / FileNotExistsRecursive --

#[test]
fn file_exists_recursive_true() {
    let dir = tempdir("fer-true");
    let sub = dir.join("sub");
    fs::create_dir_all(&sub).unwrap();
    fs::write(sub.join("deep.txt"), "").unwrap();

    let tool_input = HashMap::new();
    let ctx = make_ctx(dir.to_str().unwrap(), &tool_input, None);
    let cond = make_condition(ConditionType::FileExistsRecursive, "deep.txt");
    assert!(evaluate_condition(&cond, &ctx));
    cleanup(&dir);
}

#[test]
fn file_exists_recursive_false() {
    let dir = tempdir("fer-false");

    let tool_input = HashMap::new();
    let ctx = make_ctx(dir.to_str().unwrap(), &tool_input, None);
    let cond = make_condition(ConditionType::FileExistsRecursive, "nofile.txt");
    assert!(!evaluate_condition(&cond, &ctx));
    cleanup(&dir);
}

#[test]
fn file_exists_recursive_empty_value() {
    let dir = tempdir("fer-empty");

    let tool_input = HashMap::new();
    let ctx = make_ctx(dir.to_str().unwrap(), &tool_input, None);
    let cond = make_condition(ConditionType::FileExistsRecursive, "");
    assert!(!evaluate_condition(&cond, &ctx));
    cleanup(&dir);
}

#[test]
fn file_not_exists_recursive_true() {
    let dir = tempdir("fner-true");

    let tool_input = HashMap::new();
    let ctx = make_ctx(dir.to_str().unwrap(), &tool_input, None);
    let cond = make_condition(ConditionType::FileNotExistsRecursive, "nope.txt");
    assert!(evaluate_condition(&cond, &ctx));
    cleanup(&dir);
}

#[test]
fn file_not_exists_recursive_false() {
    let dir = tempdir("fner-false");
    let sub = dir.join("nested");
    fs::create_dir_all(&sub).unwrap();
    fs::write(sub.join("found.txt"), "").unwrap();

    let tool_input = HashMap::new();
    let ctx = make_ctx(dir.to_str().unwrap(), &tool_input, None);
    let cond = make_condition(ConditionType::FileNotExistsRecursive, "found.txt");
    assert!(!evaluate_condition(&cond, &ctx));
    cleanup(&dir);
}

// -- DirExists / DirNotExists --

#[test]
fn dir_exists_true() {
    let dir = tempdir("de-true");
    fs::create_dir_all(dir.join("subdir")).unwrap();

    let tool_input = HashMap::new();
    let ctx = make_ctx(dir.to_str().unwrap(), &tool_input, None);
    let cond = make_condition(ConditionType::DirExists, "subdir");
    assert!(evaluate_condition(&cond, &ctx));
    cleanup(&dir);
}

#[test]
fn dir_exists_false() {
    let dir = tempdir("de-false");

    let tool_input = HashMap::new();
    let ctx = make_ctx(dir.to_str().unwrap(), &tool_input, None);
    let cond = make_condition(ConditionType::DirExists, "nodir");
    assert!(!evaluate_condition(&cond, &ctx));
    cleanup(&dir);
}

#[test]
fn dir_not_exists_true() {
    let dir = tempdir("dne-true");

    let tool_input = HashMap::new();
    let ctx = make_ctx(dir.to_str().unwrap(), &tool_input, None);
    let cond = make_condition(ConditionType::DirNotExists, "nodir");
    assert!(evaluate_condition(&cond, &ctx));
    cleanup(&dir);
}

#[test]
fn dir_not_exists_false() {
    let dir = tempdir("dne-false");
    fs::create_dir_all(dir.join("exists")).unwrap();

    let tool_input = HashMap::new();
    let ctx = make_ctx(dir.to_str().unwrap(), &tool_input, None);
    let cond = make_condition(ConditionType::DirNotExists, "exists");
    assert!(!evaluate_condition(&cond, &ctx));
    cleanup(&dir);
}

// -- DirExistsRecursive / DirNotExistsRecursive --

#[test]
fn dir_exists_recursive_true() {
    let dir = tempdir("der-true");
    fs::create_dir_all(dir.join("a/b/target")).unwrap();

    let tool_input = HashMap::new();
    let ctx = make_ctx(dir.to_str().unwrap(), &tool_input, None);
    let cond = make_condition(ConditionType::DirExistsRecursive, "target");
    assert!(evaluate_condition(&cond, &ctx));
    cleanup(&dir);
}

#[test]
fn dir_exists_recursive_false() {
    let dir = tempdir("der-false");

    let tool_input = HashMap::new();
    let ctx = make_ctx(dir.to_str().unwrap(), &tool_input, None);
    let cond = make_condition(ConditionType::DirExistsRecursive, "nodir");
    assert!(!evaluate_condition(&cond, &ctx));
    cleanup(&dir);
}

#[test]
fn dir_not_exists_recursive_true() {
    let dir = tempdir("dner-true");

    let tool_input = HashMap::new();
    let ctx = make_ctx(dir.to_str().unwrap(), &tool_input, None);
    let cond = make_condition(ConditionType::DirNotExistsRecursive, "nodir");
    assert!(evaluate_condition(&cond, &ctx));
    cleanup(&dir);
}

#[test]
fn dir_not_exists_recursive_false() {
    let dir = tempdir("dner-false");
    fs::create_dir_all(dir.join("x/y/deep")).unwrap();

    let tool_input = HashMap::new();
    let ctx = make_ctx(dir.to_str().unwrap(), &tool_input, None);
    let cond = make_condition(ConditionType::DirNotExistsRecursive, "deep");
    assert!(!evaluate_condition(&cond, &ctx));
    cleanup(&dir);
}

// -- CwdIs / CwdIsNot --

#[test]
fn cwd_is_true() {
    let tool_input = HashMap::new();
    let ctx = make_ctx("/home/user/project", &tool_input, None);
    let cond = make_condition(ConditionType::CwdIs, "/home/user/project");
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn cwd_is_false() {
    let tool_input = HashMap::new();
    let ctx = make_ctx("/home/user/project", &tool_input, None);
    let cond = make_condition(ConditionType::CwdIs, "/other/path");
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn cwd_is_not_true() {
    let tool_input = HashMap::new();
    let ctx = make_ctx("/home/user/project", &tool_input, None);
    let cond = make_condition(ConditionType::CwdIsNot, "/other/path");
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn cwd_is_not_false() {
    let tool_input = HashMap::new();
    let ctx = make_ctx("/home/user/project", &tool_input, None);
    let cond = make_condition(ConditionType::CwdIsNot, "/home/user/project");
    assert!(!evaluate_condition(&cond, &ctx));
}

// -- CwdContains / CwdNotContains --

#[test]
fn cwd_contains_true() {
    let tool_input = HashMap::new();
    let ctx = make_ctx("/home/user/project", &tool_input, None);
    let cond = make_condition(ConditionType::CwdContains, "user/project");
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn cwd_contains_false() {
    let tool_input = HashMap::new();
    let ctx = make_ctx("/home/user/project", &tool_input, None);
    let cond = make_condition(ConditionType::CwdContains, "other");
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn cwd_not_contains_true() {
    let tool_input = HashMap::new();
    let ctx = make_ctx("/home/user/project", &tool_input, None);
    let cond = make_condition(ConditionType::CwdNotContains, "other");
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn cwd_not_contains_false() {
    let tool_input = HashMap::new();
    let ctx = make_ctx("/home/user/project", &tool_input, None);
    let cond = make_condition(ConditionType::CwdNotContains, "project");
    assert!(!evaluate_condition(&cond, &ctx));
}

// -- PermissionModeIs --

#[test]
fn permission_mode_is_true() {
    let tool_input = HashMap::new();
    let ctx = make_ctx("/tmp", &tool_input, Some("default"));
    let cond = make_condition(ConditionType::PermissionModeIs, "default");
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn permission_mode_is_false() {
    let tool_input = HashMap::new();
    let ctx = make_ctx("/tmp", &tool_input, Some("default"));
    let cond = make_condition(ConditionType::PermissionModeIs, "acceptEdits");
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn permission_mode_is_none() {
    let tool_input = HashMap::new();
    let ctx = make_ctx("/tmp", &tool_input, None);
    let cond = make_condition(ConditionType::PermissionModeIs, "default");
    assert!(!evaluate_condition(&cond, &ctx));
}

// -- FileExtension --

#[test]
fn file_extension_true() {
    let mut tool_input = HashMap::new();
    tool_input.insert(
        "file_path".to_string(),
        serde_json::Value::String("src/main.rs".to_string()),
    );
    let ctx = make_ctx("/tmp", &tool_input, None);
    let cond = make_condition(ConditionType::FileExtension, ".rs");
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn file_extension_false() {
    let mut tool_input = HashMap::new();
    tool_input.insert(
        "file_path".to_string(),
        serde_json::Value::String("src/main.go".to_string()),
    );
    let ctx = make_ctx("/tmp", &tool_input, None);
    let cond = make_condition(ConditionType::FileExtension, ".rs");
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn file_extension_no_file_path() {
    let tool_input = HashMap::new();
    let ctx = make_ctx("/tmp", &tool_input, None);
    let cond = make_condition(ConditionType::FileExtension, ".rs");
    assert!(!evaluate_condition(&cond, &ctx));
}

// -- CommandContains --

#[test]
fn command_contains_true() {
    let mut tool_input = HashMap::new();
    tool_input.insert(
        "command".to_string(),
        serde_json::Value::String("rm -rf /tmp/build".to_string()),
    );
    let ctx = make_ctx("/tmp", &tool_input, None);
    let cond = make_condition(ConditionType::CommandContains, "rm -rf");
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn command_contains_false() {
    let mut tool_input = HashMap::new();
    tool_input.insert(
        "command".to_string(),
        serde_json::Value::String("echo hello".to_string()),
    );
    let ctx = make_ctx("/tmp", &tool_input, None);
    let cond = make_condition(ConditionType::CommandContains, "rm");
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn command_contains_no_command_field() {
    let tool_input = HashMap::new();
    let ctx = make_ctx("/tmp", &tool_input, None);
    let cond = make_condition(ConditionType::CommandContains, "rm");
    assert!(!evaluate_condition(&cond, &ctx));
}

// -- CommandStartsWith --

#[test]
fn command_starts_with_true() {
    let mut tool_input = HashMap::new();
    tool_input.insert(
        "command".to_string(),
        serde_json::Value::String("make build".to_string()),
    );
    let ctx = make_ctx("/tmp", &tool_input, None);
    let cond = make_condition(ConditionType::CommandStartsWith, "make");
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn command_starts_with_false() {
    let mut tool_input = HashMap::new();
    tool_input.insert(
        "command".to_string(),
        serde_json::Value::String("cargo build".to_string()),
    );
    let ctx = make_ctx("/tmp", &tool_input, None);
    let cond = make_condition(ConditionType::CommandStartsWith, "make");
    assert!(!evaluate_condition(&cond, &ctx));
}

// -- UrlStartsWith --

#[test]
fn url_starts_with_true() {
    let mut tool_input = HashMap::new();
    tool_input.insert(
        "url".to_string(),
        serde_json::Value::String("https://example.com/api".to_string()),
    );
    let ctx = make_ctx("/tmp", &tool_input, None);
    let cond = make_condition(ConditionType::UrlStartsWith, "https://example.com");
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn url_starts_with_false() {
    let mut tool_input = HashMap::new();
    tool_input.insert(
        "url".to_string(),
        serde_json::Value::String("https://other.com/api".to_string()),
    );
    let ctx = make_ctx("/tmp", &tool_input, None);
    let cond = make_condition(ConditionType::UrlStartsWith, "https://example.com");
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn url_starts_with_no_url_field() {
    let tool_input = HashMap::new();
    let ctx = make_ctx("/tmp", &tool_input, None);
    let cond = make_condition(ConditionType::UrlStartsWith, "https://example.com");
    assert!(!evaluate_condition(&cond, &ctx));
}

// -- GitTrackedFileOperation --

#[test]
fn git_tracked_file_operation_no_command() {
    let tool_input = HashMap::new();
    let ctx = make_ctx("/tmp", &tool_input, None);
    let cond = make_condition(ConditionType::GitTrackedFileOperation, "rm|mv");
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn git_tracked_file_operation_non_blocked_command() {
    let mut tool_input = HashMap::new();
    tool_input.insert(
        "command".to_string(),
        serde_json::Value::String("echo hello".to_string()),
    );
    let ctx = make_ctx("/tmp", &tool_input, None);
    let cond = make_condition(ConditionType::GitTrackedFileOperation, "rm|mv");
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn git_tracked_file_operation_on_tracked_file() {
    // Use the codex-hook repo itself as a git repo with tracked files
    let cwd = env!("CARGO_MANIFEST_DIR");
    let mut tool_input = HashMap::new();
    tool_input.insert(
        "command".to_string(),
        serde_json::Value::String("rm Cargo.toml".to_string()),
    );
    let ctx = make_ctx(cwd, &tool_input, None);
    let cond = make_condition(ConditionType::GitTrackedFileOperation, "rm|mv");
    assert!(evaluate_condition(&cond, &ctx));
}

#[test]
fn git_tracked_file_operation_on_untracked_file() {
    let cwd = env!("CARGO_MANIFEST_DIR");
    let mut tool_input = HashMap::new();
    tool_input.insert(
        "command".to_string(),
        serde_json::Value::String("rm nonexistent_file_xyz_123.txt".to_string()),
    );
    let ctx = make_ctx(cwd, &tool_input, None);
    let cond = make_condition(ConditionType::GitTrackedFileOperation, "rm|mv");
    assert!(!evaluate_condition(&cond, &ctx));
}

// -- PromptRegex / EveryNPrompts / ReasonIs (not applicable to PreToolUse) --

#[test]
fn prompt_regex_false_for_pre_tool_use() {
    let tool_input = HashMap::new();
    let ctx = make_ctx("/tmp", &tool_input, None);
    let cond = make_condition(ConditionType::PromptRegex, ".*");
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn every_n_prompts_false_for_pre_tool_use() {
    let tool_input = HashMap::new();
    let ctx = make_ctx("/tmp", &tool_input, None);
    let cond = make_condition(ConditionType::EveryNPrompts, "5");
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn reason_is_false_for_pre_tool_use() {
    let tool_input = HashMap::new();
    let ctx = make_ctx("/tmp", &tool_input, None);
    let cond = make_condition(ConditionType::ReasonIs, "clear");
    assert!(!evaluate_condition(&cond, &ctx));
}

// -- UserPromptSubmit condition tests --

fn make_ups_ctx<'a>(
    cwd: &'a str,
    prompt: &'a str,
    transcript_path: &'a str,
    session_id: &'a str,
) -> UserPromptSubmitConditionContext<'a> {
    UserPromptSubmitConditionContext {
        cwd,
        prompt,
        transcript_path,
        session_id,
    }
}

#[test]
fn ups_prompt_regex_match() {
    let ctx = make_ups_ctx("/tmp", "please delete all files", "/dev/null", "s1");
    let cond = make_condition(ConditionType::PromptRegex, "delete|rm -rf");
    assert!(evaluate_user_prompt_submit_condition(&cond, &ctx));
}

#[test]
fn ups_prompt_regex_no_match() {
    let ctx = make_ups_ctx("/tmp", "add a new feature", "/dev/null", "s1");
    let cond = make_condition(ConditionType::PromptRegex, "delete|rm -rf");
    assert!(!evaluate_user_prompt_submit_condition(&cond, &ctx));
}

#[test]
fn ups_prompt_regex_invalid_pattern() {
    let ctx = make_ups_ctx("/tmp", "hello", "/dev/null", "s1");
    let cond = make_condition(ConditionType::PromptRegex, "[invalid");
    assert!(!evaluate_user_prompt_submit_condition(&cond, &ctx));
}

#[test]
fn ups_prompt_regex_empty_prompt() {
    let ctx = make_ups_ctx("/tmp", "", "/dev/null", "s1");
    let cond = make_condition(ConditionType::PromptRegex, ".*");
    assert!(evaluate_user_prompt_submit_condition(&cond, &ctx));
}

#[test]
fn ups_every_n_prompts_fires_on_first() {
    let dir = tempdir("ups-every-n-first");
    let transcript = dir.join("transcript.jsonl");
    // Empty transcript → count=0+1=1, 1%1==0 → true
    fs::write(&transcript, "").unwrap();
    let tp = transcript.to_str().unwrap();
    let ctx = make_ups_ctx("/tmp", "hello", tp, "s1");
    let cond = make_condition(ConditionType::EveryNPrompts, "1");
    assert!(evaluate_user_prompt_submit_condition(&cond, &ctx));
    cleanup(&dir);
}

#[test]
fn ups_every_n_prompts_with_transcript() {
    let dir = tempdir("ups-every-n-transcript");
    let transcript = dir.join("transcript.jsonl");
    // 2 user prompts in transcript for session s1 → count=2+1=3, 3%3==0 → true
    let content = r#"{"type":"user","sessionId":"s1","content":"first"}
{"type":"assistant","sessionId":"s1","content":"reply"}
{"type":"user","sessionId":"s1","content":"second"}
"#;
    fs::write(&transcript, content).unwrap();
    let tp = transcript.to_str().unwrap();
    let ctx = make_ups_ctx("/tmp", "third", tp, "s1");
    let cond = make_condition(ConditionType::EveryNPrompts, "3");
    assert!(evaluate_user_prompt_submit_condition(&cond, &ctx));
    cleanup(&dir);
}

#[test]
fn ups_every_n_prompts_not_multiple() {
    let dir = tempdir("ups-every-n-not-mult");
    let transcript = dir.join("transcript.jsonl");
    // 1 user prompt → count=1+1=2, 2%3!=0 → false
    let content = r#"{"type":"user","sessionId":"s1","content":"first"}
"#;
    fs::write(&transcript, content).unwrap();
    let tp = transcript.to_str().unwrap();
    let ctx = make_ups_ctx("/tmp", "second", tp, "s1");
    let cond = make_condition(ConditionType::EveryNPrompts, "3");
    assert!(!evaluate_user_prompt_submit_condition(&cond, &ctx));
    cleanup(&dir);
}

#[test]
fn ups_every_n_prompts_filters_by_session_id() {
    let dir = tempdir("ups-every-n-session");
    let transcript = dir.join("transcript.jsonl");
    // 1 prompt for s1, 1 for s2 → count for s1 = 1+1=2, 2%2==0 → true
    let content = r#"{"type":"user","sessionId":"s1","content":"first"}
{"type":"user","sessionId":"s2","content":"other"}
"#;
    fs::write(&transcript, content).unwrap();
    let tp = transcript.to_str().unwrap();
    let ctx = make_ups_ctx("/tmp", "second", tp, "s1");
    let cond = make_condition(ConditionType::EveryNPrompts, "2");
    assert!(evaluate_user_prompt_submit_condition(&cond, &ctx));
    cleanup(&dir);
}

#[test]
fn ups_every_n_prompts_invalid_value() {
    let ctx = make_ups_ctx("/tmp", "hello", "/dev/null", "s1");
    let cond = make_condition(ConditionType::EveryNPrompts, "abc");
    assert!(!evaluate_user_prompt_submit_condition(&cond, &ctx));
}

#[test]
fn ups_every_n_prompts_zero_value() {
    let ctx = make_ups_ctx("/tmp", "hello", "/dev/null", "s1");
    let cond = make_condition(ConditionType::EveryNPrompts, "0");
    assert!(!evaluate_user_prompt_submit_condition(&cond, &ctx));
}

#[test]
fn ups_every_n_prompts_missing_transcript() {
    let ctx = make_ups_ctx("/tmp", "hello", "/nonexistent/transcript.jsonl", "s1");
    let cond = make_condition(ConditionType::EveryNPrompts, "3");
    assert!(!evaluate_user_prompt_submit_condition(&cond, &ctx));
}

#[test]
fn ups_cwd_conditions_work() {
    let ctx = make_ups_ctx("/home/user/project", "hello", "/dev/null", "s1");
    let cond = make_condition(ConditionType::CwdContains, "user/project");
    assert!(evaluate_user_prompt_submit_condition(&cond, &ctx));
}

#[test]
fn ups_tool_specific_conditions_return_false() {
    let ctx = make_ups_ctx("/tmp", "hello", "/dev/null", "s1");
    let cond = make_condition(ConditionType::FileExtension, ".rs");
    assert!(!evaluate_user_prompt_submit_condition(&cond, &ctx));
}

#[test]
fn ups_evaluate_all_conditions_and_logic() {
    let ctx = make_ups_ctx("/home/user", "delete everything", "/dev/null", "s1");
    let conditions = vec![
        make_condition(ConditionType::PromptRegex, "delete"),
        make_condition(ConditionType::CwdContains, "user"),
    ];
    assert!(evaluate_user_prompt_submit_conditions(&conditions, &ctx));
}

#[test]
fn ups_evaluate_conditions_one_false_fails() {
    let ctx = make_ups_ctx("/home/user", "add feature", "/dev/null", "s1");
    let conditions = vec![
        make_condition(ConditionType::PromptRegex, "delete"),
        make_condition(ConditionType::CwdContains, "user"),
    ];
    assert!(!evaluate_user_prompt_submit_conditions(&conditions, &ctx));
}

#[test]
fn ups_empty_conditions_returns_true() {
    let ctx = make_ups_ctx("/tmp", "hello", "/dev/null", "s1");
    assert!(evaluate_user_prompt_submit_conditions(&[], &ctx));
}

// -- Helpers --

fn tempdir(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir()
        .join("codex-hook-condition-tests")
        .join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn cleanup(dir: &std::path::Path) {
    let _ = fs::remove_dir_all(dir);
}
