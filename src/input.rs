//! JSON input types for Codex CLI hook events.
//!
//! Codex passes JSON on stdin when invoking a hook. The schema is defined by
//! `BaseHookInput` and per-event extensions in the Codex hooks reference:
//! <https://docs.anthropic.com/en/docs/claude-code/hooks>
//!
//! `BaseInput` holds the fields common to every event type.
//! `PreToolUseInput` adds `tool_name` and `tool_input` for the PreToolUse event.

use serde::Deserialize;
use std::collections::HashMap;
use std::io::{BufReader, Read};

/// Fields common to every Codex hook event (mirrors `BaseHookInput` in the SDK).
#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct BaseInput {
    pub(crate) session_id: String,
    pub(crate) transcript_path: String,
    pub(crate) cwd: String,
    pub(crate) hook_event_name: String,
}

/// Input for the `PreToolUse` hook event.
#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct PreToolUseInput {
    #[serde(flatten)]
    pub(crate) base: BaseInput,

    pub(crate) tool_name: String,
    pub(crate) tool_input: HashMap<String, serde_json::Value>,
}

/// Read JSON from stdin and deserialize into `PreToolUseInput`.
#[cfg(test)]
pub(crate) fn read_pre_tool_use_input<R: Read>(
    reader: R,
) -> Result<PreToolUseInput, serde_json::Error> {
    let buf = BufReader::new(reader);
    serde_json::from_reader(buf)
}

/// Read JSON from a reader into a buffer and return both the deserialized
/// struct and the raw bytes (for use with the jaq template engine).
pub(crate) fn read_pre_tool_use_input_with_raw<R: Read>(
    reader: R,
) -> Result<(PreToolUseInput, Vec<u8>), InputError> {
    let mut buf = BufReader::new(reader);
    let mut raw = Vec::new();
    buf.read_to_end(&mut raw)
        .map_err(|e| InputError::Io(e.to_string()))?;
    let input: PreToolUseInput =
        serde_json::from_slice(&raw).map_err(|e| InputError::Json(e.to_string()))?;
    Ok((input, raw))
}

/// Errors that can occur when reading hook input.
#[derive(Debug)]
pub(crate) enum InputError {
    Io(String),
    Json(String),
}

impl std::fmt::Display for InputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputError::Io(e) => write!(f, "failed to read input: {}", e),
            InputError::Json(e) => write!(f, "failed to parse JSON input: {}", e),
        }
    }
}

#[cfg(test)]
#[path = "input_tests.rs"]
mod input_tests;
