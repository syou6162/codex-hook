use super::*;
use crate::config::PreToolUseHook;

// -- check_matcher tests --

#[test]
fn empty_matcher_matches_all() {
    assert!(check_matcher("", "Write").unwrap());
    assert!(check_matcher("", "Bash").unwrap());
    assert!(check_matcher("", "").unwrap());
}

#[test]
fn exact_match() {
    assert!(check_matcher("Write", "Write").unwrap());
}

#[test]
fn no_match() {
    assert!(!check_matcher("Write", "Bash").unwrap());
}

#[test]
fn partial_match() {
    assert!(check_matcher("Write", "WriteFile").unwrap());
}

#[test]
fn pipe_alternation_first() {
    assert!(check_matcher("Write|Edit", "Write").unwrap());
}

#[test]
fn pipe_alternation_second() {
    assert!(check_matcher("Write|Edit", "Edit").unwrap());
}

#[test]
fn pipe_alternation_no_match() {
    assert!(!check_matcher("Write|Edit", "Read").unwrap());
}

#[test]
fn case_sensitive() {
    assert!(!check_matcher("write", "Write").unwrap());
}

#[test]
fn complex_tool_name_partial() {
    assert!(check_matcher("Multi", "MultiEdit").unwrap());
}

#[test]
fn invalid_regex_returns_error() {
    let result = check_matcher("[invalid", "Write");
    assert!(result.is_err());
}

// -- filter_pre_tool_use_hooks tests --

fn make_hook(matcher: &str) -> PreToolUseHook {
    PreToolUseHook {
        matcher: matcher.to_string(),
        conditions: vec![],
        actions: vec![],
    }
}

#[test]
fn filter_returns_matching_hooks() {
    let hooks = vec![make_hook("Write"), make_hook("Bash"), make_hook("Edit")];
    let matched = filter_pre_tool_use_hooks(&hooks, "Write").unwrap();
    assert_eq!(matched.len(), 1);
    assert_eq!(matched[0].matcher, "Write");
}

#[test]
fn filter_empty_matcher_matches_all() {
    let hooks = vec![make_hook(""), make_hook("Bash")];
    let matched = filter_pre_tool_use_hooks(&hooks, "Write").unwrap();
    assert_eq!(matched.len(), 1);
    assert_eq!(matched[0].matcher, "");
}

#[test]
fn filter_multiple_matches() {
    let hooks = vec![
        make_hook("Write|Edit"),
        make_hook("Write"),
        make_hook("Bash"),
    ];
    let matched = filter_pre_tool_use_hooks(&hooks, "Write").unwrap();
    assert_eq!(matched.len(), 2);
}

#[test]
fn filter_no_matches() {
    let hooks = vec![make_hook("Bash"), make_hook("Edit")];
    let matched = filter_pre_tool_use_hooks(&hooks, "Read").unwrap();
    assert!(matched.is_empty());
}

#[test]
fn filter_propagates_regex_error() {
    let hooks = vec![make_hook("[invalid")];
    let result = filter_pre_tool_use_hooks(&hooks, "Write");
    assert!(result.is_err());
}
