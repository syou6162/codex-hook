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

// -- PromptRegex / EveryNPrompts / ReasonIs (not applicable) --

#[test]
fn prompt_regex_always_false() {
    let tool_input = HashMap::new();
    let ctx = make_ctx("/tmp", &tool_input, None);
    let cond = make_condition(ConditionType::PromptRegex, ".*");
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn every_n_prompts_always_false() {
    let tool_input = HashMap::new();
    let ctx = make_ctx("/tmp", &tool_input, None);
    let cond = make_condition(ConditionType::EveryNPrompts, "5");
    assert!(!evaluate_condition(&cond, &ctx));
}

#[test]
fn reason_is_always_false() {
    let tool_input = HashMap::new();
    let ctx = make_ctx("/tmp", &tool_input, None);
    let cond = make_condition(ConditionType::ReasonIs, "clear");
    assert!(!evaluate_condition(&cond, &ctx));
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
