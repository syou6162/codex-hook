//! Template engine for replacing `{.field}` patterns with jq query results.
//!
//! Uses jaq (jq clone in Rust) to evaluate queries against the JSON input
//! that Codex passes to hooks on stdin.
//!
//! Pattern: `{<jq_query>}` — content inside braces is treated as a jq query.
//! Empty braces `{}` are NOT treated as templates (regex requires at least one char).
//!
//! Reference: cchook `template_jq.go` — `unifiedTemplateReplace`.

use jaq_core::load::{Arena, File, Loader};
use jaq_core::{data, unwrap_valr, Compiler, Ctx, Vars};
use jaq_json::Val;
use regex::Regex;

/// Execute a jq query against a JSON input value and return the result as a string.
pub(crate) fn execute_jq_query(query_str: &str, input: &Val) -> Result<String, String> {
    let defs = jaq_core::defs()
        .chain(jaq_std::defs())
        .chain(jaq_json::defs());
    let funs = jaq_core::funs()
        .chain(jaq_std::funs())
        .chain(jaq_json::funs());
    let loader = Loader::new(defs);
    let arena = Arena::default();
    let program = File {
        code: query_str,
        path: (),
    };
    let modules = loader
        .load(&arena, program)
        .map_err(|errs| format!("invalid jq query '{}': {:?}", query_str, errs))?;
    let filter = Compiler::<_, data::JustLut<Val>>::default()
        .with_funs(funs)
        .compile(modules)
        .map_err(|errs| format!("compile error for '{}': {:?}", query_str, errs))?;

    let ctx = Ctx::<data::JustLut<Val>>::new(&filter.lut, Vars::new([]));
    let out = filter.id.run((ctx, input.clone()));

    let results: Vec<Val> = out
        .map(unwrap_valr)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("jq query execution error: {:?}", e))?;

    match results.len() {
        0 => Ok(String::new()),
        1 => Ok(val_to_string(&results[0])),
        _ => {
            let parts: Vec<String> = results.iter().map(val_to_json).collect();
            Ok(format!("[{}]", parts.join(",")))
        }
    }
}

/// Convert a jaq Val to a user-facing string (matching cchook's jqValueToString).
///
/// - Null → empty string
/// - Bool → "true" / "false"
/// - String (TStr/BStr) → raw string content (no quotes)
/// - Number, Array, Object → JSON representation
fn val_to_string(val: &Val) -> String {
    match val {
        Val::Null => String::new(),
        Val::Bool(b) => b.to_string(),
        Val::TStr(s) => String::from_utf8_lossy(s).into_owned(),
        Val::BStr(s) => String::from_utf8_lossy(s).into_owned(),
        _ => val_to_json(val),
    }
}

/// Serialize a Val to its JSON representation.
fn val_to_json(val: &Val) -> String {
    let mut buf = Vec::new();
    jaq_json::write::write(&mut buf, &Default::default(), 0, val)
        .expect("write to Vec should not fail");
    // Trim trailing newline if present
    if buf.last() == Some(&b'\n') {
        buf.pop();
    }
    String::from_utf8(buf).expect("jaq JSON output should be valid UTF-8")
}

/// Replace all `{<query>}` patterns in a template string with jq query results.
///
/// Uses the same regex as cchook: `\{([^}]+)\}` — matches `{` followed by one or
/// more non-`}` characters, then `}`. Empty braces `{}` are not matched.
pub(crate) fn template_replace(template: &str, input: &Val) -> String {
    let pattern = Regex::new(r"\{([^}]+)\}").expect("regex is valid");
    pattern
        .replace_all(template, |caps: &regex::Captures| {
            let jq_query = caps[1].trim();
            match execute_jq_query(jq_query, input) {
                Ok(result) => result,
                Err(err) => format!("[JQ_ERROR: {}]", err),
            }
        })
        .into_owned()
}

#[cfg(test)]
#[path = "template_tests.rs"]
mod template_tests;
