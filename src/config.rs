//! cchook 互換の YAML 設定スキーマ。
//!
//! このモジュールは cchook (Go 版) の YAML config 形式を Rust で再実装したものです。
//! Codex (Claude Code) 本体の `settings.json` 形式とは異なるレイヤーです。
//!
//! Codex → (settings.json で shell hook として codex-hook を起動) → codex-hook が
//! この YAML config を読み込み、条件評価・アクション実行を行い、Codex が理解する
//! JSON を stdout に返します。
//!
//! 参照:
//! - Codex hooks reference: https://docs.anthropic.com/en/docs/claude-code/hooks
//! - cchook (Go 版): https://github.com/syou6162/cchook

use crate::error::ConfigError;
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Codex が hook に渡す JSON input のフィールドに基づく条件型。
///
/// cchook と同じ 21 variant を定義。各 variant が対応する Codex input field を
/// コメントで示す。
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ConditionType {
    // ---- Common: 全イベント共通（BaseInput.cwd / 環境に基づく） ----
    /// ファイル存在チェック（カレントディレクトリ直下）
    FileExists,
    /// ファイル存在チェック（再帰的）
    FileExistsRecursive,
    /// ファイル非存在チェック（カレントディレクトリ直下）
    FileNotExists,
    /// ファイル非存在チェック（再帰的）
    FileNotExistsRecursive,
    /// ディレクトリ存在チェック（カレントディレクトリ直下）
    DirExists,
    /// ディレクトリ存在チェック（再帰的）
    DirExistsRecursive,
    /// ディレクトリ非存在チェック（カレントディレクトリ直下）
    DirNotExists,
    /// ディレクトリ非存在チェック（再帰的）
    DirNotExistsRecursive,
    /// BaseInput.cwd が完全一致
    CwdIs,
    /// BaseInput.cwd が完全不一致
    CwdIsNot,
    /// BaseInput.cwd に部分文字列を含む
    CwdContains,
    /// BaseInput.cwd に部分文字列を含まない
    CwdNotContains,
    /// BaseInput.permission_mode が完全一致
    PermissionModeIs,

    // ---- Tool-specific: PreToolUse/PostToolUse（tool_input フィールドに基づく） ----
    /// tool_input.file_path の拡張子チェック
    FileExtension,
    /// tool_input.command に部分文字列を含む
    CommandContains,
    /// tool_input.command が指定文字列で始まる
    CommandStartsWith,
    /// tool_input.url が指定文字列で始まる
    UrlStartsWith,
    /// tool_input.command で Git 管理ファイルへの操作をチェック
    GitTrackedFileOperation,

    // ---- Prompt-specific: UserPromptSubmit（prompt フィールドに基づく） ----
    /// UserPromptSubmitInput.prompt に対する正規表現マッチ
    PromptRegex,
    /// transcript 内の user prompt カウントに基づく定期実行
    EveryNPrompts,

    // ---- Session-specific: SessionEnd（reason フィールドに基づく） ----
    /// SessionEndInput のセッション終了理由が完全一致
    ReasonIs,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct Condition {
    #[serde(rename = "type")]
    pub(crate) condition_type: ConditionType,

    pub(crate) value: String,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ActionType {
    Command,
    Output,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct Action {
    #[serde(rename = "type")]
    pub(crate) action_type: ActionType,

    #[serde(default)]
    pub(crate) command: Option<String>,

    #[serde(default)]
    pub(crate) message: Option<String>,

    #[serde(default)]
    pub(crate) exit_status: Option<i32>,
}

// ---- matcher 必須のイベント型 ----
// Codex の PreToolUse/PostToolUse は tool_name でマッチ
// SessionStart は source ("startup", "resume", "clear", "compact") でマッチ
// PreCompact は trigger ("manual", "auto") でマッチ
// PermissionRequest は tool_name でマッチ
// SubagentStart は agent_type でマッチ

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct PreToolUseHook {
    pub(crate) matcher: String,

    #[serde(default)]
    pub(crate) conditions: Vec<Condition>,

    #[serde(default)]
    pub(crate) actions: Vec<Action>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct PostToolUseHook {
    pub(crate) matcher: String,

    #[serde(default)]
    pub(crate) conditions: Vec<Condition>,

    #[serde(default)]
    pub(crate) actions: Vec<Action>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct PermissionRequestHook {
    pub(crate) matcher: String,

    #[serde(default)]
    pub(crate) conditions: Vec<Condition>,

    #[serde(default)]
    pub(crate) actions: Vec<Action>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct SessionStartHook {
    pub(crate) matcher: String,

    #[serde(default)]
    pub(crate) conditions: Vec<Condition>,

    #[serde(default)]
    pub(crate) actions: Vec<Action>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct PreCompactHook {
    pub(crate) matcher: String,

    #[serde(default)]
    pub(crate) conditions: Vec<Condition>,

    #[serde(default)]
    pub(crate) actions: Vec<Action>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct SubagentStartHook {
    pub(crate) matcher: String,

    #[serde(default)]
    pub(crate) conditions: Vec<Condition>,

    #[serde(default)]
    pub(crate) actions: Vec<Action>,
}

// ---- matcher 任意のイベント型 ----
// Codex の Notification は notification_type でマッチ（省略可）
// SubagentStop は agent_type でマッチ（省略可）
// SessionEnd は reason でマッチ（省略可）

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct NotificationHook {
    #[serde(default)]
    pub(crate) matcher: Option<String>,

    #[serde(default)]
    pub(crate) conditions: Vec<Condition>,

    #[serde(default)]
    pub(crate) actions: Vec<Action>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct SubagentStopHook {
    #[serde(default)]
    pub(crate) matcher: Option<String>,

    #[serde(default)]
    pub(crate) conditions: Vec<Condition>,

    #[serde(default)]
    pub(crate) actions: Vec<Action>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct SessionEndHook {
    #[serde(default)]
    pub(crate) matcher: Option<String>,

    #[serde(default)]
    pub(crate) conditions: Vec<Condition>,

    #[serde(default)]
    pub(crate) actions: Vec<Action>,
}

// ---- matcher なしのイベント型 ----
// Codex の Stop / UserPromptSubmit にはマッチ対象フィールドがない

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct StopHook {
    #[serde(default)]
    pub(crate) conditions: Vec<Condition>,

    #[serde(default)]
    pub(crate) actions: Vec<Action>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(crate) struct UserPromptSubmitHook {
    #[serde(default)]
    pub(crate) conditions: Vec<Condition>,

    #[serde(default)]
    pub(crate) actions: Vec<Action>,
}

/// cchook 互換の設定ファイル構造。
///
/// cchook (Go 版) が対応する 11 イベント型すべてを定義。
/// Codex 本体は 27+ のイベント型を持つが、cchook が対応していないものは
/// 今後必要に応じて追加する。
#[derive(Debug, Deserialize, PartialEq, Default)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct Config {
    #[serde(default)]
    pub(crate) pre_tool_use: Vec<PreToolUseHook>,

    #[serde(default)]
    pub(crate) post_tool_use: Vec<PostToolUseHook>,

    #[serde(default)]
    pub(crate) permission_request: Vec<PermissionRequestHook>,

    #[serde(default)]
    pub(crate) notification: Vec<NotificationHook>,

    #[serde(default)]
    pub(crate) stop: Vec<StopHook>,

    #[serde(default)]
    pub(crate) subagent_stop: Vec<SubagentStopHook>,

    #[serde(default)]
    pub(crate) subagent_start: Vec<SubagentStartHook>,

    #[serde(default)]
    pub(crate) pre_compact: Vec<PreCompactHook>,

    #[serde(default)]
    pub(crate) session_start: Vec<SessionStartHook>,

    #[serde(default)]
    pub(crate) session_end: Vec<SessionEndHook>,

    #[serde(default)]
    pub(crate) user_prompt_submit: Vec<UserPromptSubmitHook>,
}

pub(crate) fn default_config_path() -> PathBuf {
    let config_dir = std::env::var("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| home_dir().join(".config"));
    config_dir.join("codex-hook").join("config.yaml")
}

fn home_dir() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .expect("HOME environment variable is not set")
}

pub(crate) fn load_config(path: &Path) -> Result<Config, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    let config: Config = serde_saphyr::from_str(&content)?;
    Ok(config)
}

pub(crate) fn load_config_or_default(path: &Path) -> Result<Config, ConfigError> {
    match std::fs::read_to_string(path) {
        Ok(content) => Ok(serde_saphyr::from_str(&content)?),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Config::default()),
        Err(e) => Err(ConfigError::Io(e)),
    }
}

#[cfg(test)]
#[path = "config_tests.rs"]
mod config_tests;
