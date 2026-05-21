//! JSON input types for Codex CLI hook events.
//!
//! Codex passes JSON on stdin when invoking a hook. The schema is defined by
//! `BaseHookInput`-style fields and per-event extensions in the official
//! OpenAI Codex hook schemas:
//! <https://github.com/openai/codex/tree/main/codex-rs/hooks/schema/generated>
//!
//! `BaseInput` holds the fields common to every event type.
//! `PreToolUseInput` adds `tool_name` and `tool_input` for the PreToolUse event.
//! `PostToolUseInput` adds the same tool fields plus `tool_response`.
//! `UserPromptSubmitInput` adds `prompt` for the UserPromptSubmit event.

use serde::Deserialize;
use std::collections::HashMap;
use std::io::{BufReader, Read};

/// Fields common to every Codex hook event (mirrors `BaseHookInput` in the SDK).
#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct BaseInput {
    pub(crate) session_id: String,
    pub(crate) transcript_path: Option<String>,
    pub(crate) cwd: String,
    pub(crate) hook_event_name: String,

    #[serde(default)]
    pub(crate) turn_id: Option<String>,

    #[serde(default)]
    pub(crate) model: Option<String>,

    #[serde(default)]
    pub(crate) permission_mode: Option<String>,
}

/// Input for the `PreToolUse` hook event.
#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct PreToolUseInput {
    #[serde(flatten)]
    pub(crate) base: BaseInput,

    pub(crate) tool_name: String,
    pub(crate) tool_input: HashMap<String, serde_json::Value>,

    #[serde(default)]
    pub(crate) tool_use_id: Option<String>,
}

/// Input for the `PostToolUse` hook event.
#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct PostToolUseInput {
    #[serde(flatten)]
    pub(crate) base: BaseInput,

    pub(crate) tool_name: String,
    pub(crate) tool_input: HashMap<String, serde_json::Value>,

    pub(crate) tool_response: serde_json::Value,

    #[serde(default)]
    pub(crate) tool_use_id: Option<String>,
}

/// Input for the `UserPromptSubmit` hook event.
#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct UserPromptSubmitInput {
    #[serde(flatten)]
    pub(crate) base: BaseInput,

    pub(crate) prompt: String,
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

/// Read JSON from a reader into `PostToolUseInput` and return both the
/// deserialized struct and the raw bytes.
pub(crate) fn read_post_tool_use_input_with_raw<R: Read>(
    reader: R,
) -> Result<(PostToolUseInput, Vec<u8>), InputError> {
    let mut buf = BufReader::new(reader);
    let mut raw = Vec::new();
    buf.read_to_end(&mut raw)
        .map_err(|e| InputError::Io(e.to_string()))?;
    let input: PostToolUseInput =
        serde_json::from_slice(&raw).map_err(|e| InputError::Json(e.to_string()))?;
    Ok((input, raw))
}

/// Read JSON from a reader into `UserPromptSubmitInput` and return both the
/// deserialized struct and the raw bytes.
pub(crate) fn read_user_prompt_submit_input_with_raw<R: Read>(
    reader: R,
) -> Result<(UserPromptSubmitInput, Vec<u8>), InputError> {
    let mut buf = BufReader::new(reader);
    let mut raw = Vec::new();
    buf.read_to_end(&mut raw)
        .map_err(|e| InputError::Io(e.to_string()))?;
    let input: UserPromptSubmitInput =
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

impl std::error::Error for InputError {}

#[cfg(test)]
#[path = "input_tests.rs"]
mod input_tests;
