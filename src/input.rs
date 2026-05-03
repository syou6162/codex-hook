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
pub(crate) fn read_pre_tool_use_input<R: Read>(
    reader: R,
) -> Result<PreToolUseInput, serde_json::Error> {
    let buf = BufReader::new(reader);
    serde_json::from_reader(buf)
}

#[cfg(test)]
#[path = "input_tests.rs"]
mod input_tests;
