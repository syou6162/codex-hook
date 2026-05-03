//! Matcher logic for filtering hooks by tool name.
//!
//! A matcher is a regular expression pattern (compatible with cchook's
//! pipe-separated syntax, since `|` is regex alternation).
//! An empty matcher matches every tool name.

use crate::config::PreToolUseHook;
use regex::Regex;

/// Check whether `tool_name` matches the given matcher pattern.
///
/// Returns `true` when the matcher is empty (matches everything) or when
/// the compiled regex finds a match in `tool_name`.
pub(crate) fn check_matcher(matcher: &str, tool_name: &str) -> Result<bool, regex::Error> {
    if matcher.is_empty() {
        return Ok(true);
    }
    let re = Regex::new(matcher)?;
    Ok(re.is_match(tool_name))
}

/// Return references to the hooks whose matcher matches `tool_name`.
pub(crate) fn filter_pre_tool_use_hooks<'a>(
    hooks: &'a [PreToolUseHook],
    tool_name: &str,
) -> Result<Vec<&'a PreToolUseHook>, regex::Error> {
    let mut matched = Vec::new();
    for hook in hooks {
        if check_matcher(&hook.matcher, tool_name)? {
            matched.push(hook);
        }
    }
    Ok(matched)
}

#[cfg(test)]
#[path = "matcher_tests.rs"]
mod matcher_tests;
