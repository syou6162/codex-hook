mod action;
mod condition;
mod config;
mod error;
mod event;
mod input;
mod matcher;
mod template;

use action::{
    build_output_json, build_post_tool_use_output_json, execute_command, merge_exit_status,
};
use clap::Parser;
use condition::{
    evaluate_conditions, evaluate_user_prompt_submit_conditions, ConditionContext,
    UserPromptSubmitConditionContext,
};
use config::{default_config_path, load_config, load_config_or_default, ActionType, Config};
use error::HookError;
use event::HookEventType;
use input::{
    read_post_tool_use_input_with_raw, read_pre_tool_use_input_with_raw,
    read_user_prompt_submit_input_with_raw,
};
use matcher::{filter_post_tool_use_hooks, filter_pre_tool_use_hooks};
use std::path::PathBuf;
use template::template_replace;

#[derive(Parser)]
#[command(about = "A hook tool for Codex CLI")]
struct Cli {
    #[arg(long)]
    event: HookEventType,

    #[arg(long)]
    config: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    let config = load_config_from_cli(&cli).unwrap_or_else(|e| {
        eprintln!("error: {}", e);
        std::process::exit(1);
    });

    let result = match cli.event {
        HookEventType::PreToolUse => process_pre_tool_use(&config),
        HookEventType::PostToolUse => process_post_tool_use(&config),
        HookEventType::UserPromptSubmit => process_user_prompt_submit(&config),
        _ => {
            // TODO: 他のイベントタイプの入力パース・処理は後続チケットで実装する。
            Ok(None)
        }
    };

    match result {
        Ok(Some(code)) => std::process::exit(code),
        Ok(None) => {}
        Err(e) => {
            eprintln!("error: {}", e);
            std::process::exit(e.exit_code());
        }
    }
}

fn load_config_from_cli(cli: &Cli) -> Result<Config, error::ConfigError> {
    match &cli.config {
        Some(path) => load_config(&PathBuf::from(path)),
        None => load_config_or_default(&default_config_path()),
    }
}

/// Process a PreToolUse event: read input, filter hooks, evaluate conditions,
/// and execute actions.
///
/// Returns `Ok(None)` for normal exit or `Ok(Some(code))` when output actions
/// specify an explicit exit status.
fn process_pre_tool_use(config: &Config) -> Result<Option<i32>, HookError> {
    let (input, raw_bytes) = read_pre_tool_use_input_with_raw(std::io::stdin())?;

    let jaq_input = jaq_json::read::parse_single(&raw_bytes)
        .map_err(|e| HookError::TemplateParse(format!("{:?}", e)))?;

    let matched = filter_pre_tool_use_hooks(&config.pre_tool_use, &input.tool_name)?;

    let mut output_messages: Vec<String> = Vec::new();
    let mut merged_exit_status: Option<i32> = None;
    let cond_ctx = ConditionContext {
        cwd: &input.base.cwd,
        tool_input: &input.tool_input,
        permission_mode: input.base.permission_mode.as_deref(),
    };

    for hook in &matched {
        if !evaluate_conditions(&hook.conditions, &cond_ctx) {
            continue;
        }
        for action in &hook.actions {
            match action.action_type {
                ActionType::Command => {
                    if let Some(cmd) = &action.command {
                        let cmd = template_replace(cmd, &jaq_input);
                        let status =
                            execute_command(&cmd).map_err(|e| HookError::CommandExecution {
                                command: cmd.clone(),
                                source: e,
                            })?;
                        if !status.success() {
                            let code = status.code().unwrap_or(1);
                            return Err(HookError::CommandFailed { code, command: cmd });
                        }
                    }
                }
                ActionType::Output => {
                    if let Some(msg) = &action.message {
                        let msg = template_replace(msg, &jaq_input);
                        output_messages.push(msg);
                    } else {
                        eprintln!("error: output action has no message");
                    }
                    if let Some(code) = action.exit_status {
                        merged_exit_status = merge_exit_status(merged_exit_status, code);
                    }
                }
            }
        }
    }

    if !output_messages.is_empty() {
        let merged = output_messages.join("\n");
        println!("{}", build_output_json(&merged));
    }

    Ok(merged_exit_status)
}

/// Process a PostToolUse event: read input, filter hooks, evaluate conditions,
/// and execute actions.
///
/// Returns `Ok(None)` for normal exit. A non-zero output action is converted
/// to Codex's PostToolUse blocking JSON instead of using the process exit code.
fn process_post_tool_use(config: &Config) -> Result<Option<i32>, HookError> {
    let (input, raw_bytes) = read_post_tool_use_input_with_raw(std::io::stdin())?;

    let jaq_input = jaq_json::read::parse_single(&raw_bytes)
        .map_err(|e| HookError::TemplateParse(format!("{:?}", e)))?;

    let matched = filter_post_tool_use_hooks(&config.post_tool_use, &input.tool_name)?;

    let mut output_messages: Vec<String> = Vec::new();
    let mut merged_exit_status: Option<i32> = None;
    let cond_ctx = ConditionContext {
        cwd: &input.base.cwd,
        tool_input: &input.tool_input,
        permission_mode: input.base.permission_mode.as_deref(),
    };

    for hook in &matched {
        if !evaluate_conditions(&hook.conditions, &cond_ctx) {
            continue;
        }
        for action in &hook.actions {
            match action.action_type {
                ActionType::Command => {
                    if let Some(cmd) = &action.command {
                        let cmd = template_replace(cmd, &jaq_input);
                        let status =
                            execute_command(&cmd).map_err(|e| HookError::CommandExecution {
                                command: cmd.clone(),
                                source: e,
                            })?;
                        if !status.success() {
                            let code = status.code().unwrap_or(1);
                            return Err(HookError::CommandFailed { code, command: cmd });
                        }
                    }
                }
                ActionType::Output => {
                    if let Some(msg) = &action.message {
                        let msg = template_replace(msg, &jaq_input);
                        output_messages.push(msg);
                    } else {
                        eprintln!("error: output action has no message");
                    }
                    if let Some(code) = action.exit_status {
                        merged_exit_status = merge_exit_status(merged_exit_status, code);
                    }
                }
            }
        }
    }

    if let Some(output_json) = build_post_tool_use_output_json(&output_messages, merged_exit_status)
    {
        println!("{}", output_json);
    }

    Ok(None)
}

/// Process a UserPromptSubmit event: read input, evaluate conditions,
/// and execute actions.
///
/// Returns `Ok(None)` for normal exit or `Ok(Some(code))` when output actions
/// specify an explicit exit status.
fn process_user_prompt_submit(config: &Config) -> Result<Option<i32>, HookError> {
    let (input, raw_bytes) = read_user_prompt_submit_input_with_raw(std::io::stdin())?;

    let jaq_input = jaq_json::read::parse_single(&raw_bytes)
        .map_err(|e| HookError::TemplateParse(format!("{:?}", e)))?;

    let mut output_messages: Vec<String> = Vec::new();
    let mut merged_exit_status: Option<i32> = None;
    let cond_ctx = UserPromptSubmitConditionContext {
        cwd: &input.base.cwd,
        prompt: &input.prompt,
        transcript_path: input.base.transcript_path.as_deref(),
        session_id: &input.base.session_id,
    };

    for hook in &config.user_prompt_submit {
        if !evaluate_user_prompt_submit_conditions(&hook.conditions, &cond_ctx) {
            continue;
        }
        for action in &hook.actions {
            match action.action_type {
                ActionType::Command => {
                    if let Some(cmd) = &action.command {
                        let cmd = template_replace(cmd, &jaq_input);
                        let status =
                            execute_command(&cmd).map_err(|e| HookError::CommandExecution {
                                command: cmd.clone(),
                                source: e,
                            })?;
                        if !status.success() {
                            let code = status.code().unwrap_or(1);
                            return Err(HookError::CommandFailed { code, command: cmd });
                        }
                    }
                }
                ActionType::Output => {
                    if let Some(msg) = &action.message {
                        let msg = template_replace(msg, &jaq_input);
                        output_messages.push(msg);
                    } else {
                        eprintln!("error: output action has no message");
                    }
                    if let Some(code) = action.exit_status {
                        merged_exit_status = merge_exit_status(merged_exit_status, code);
                    }
                }
            }
        }
    }

    if !output_messages.is_empty() {
        let merged = output_messages.join("\n");
        println!("{}", build_output_json(&merged));
    }

    Ok(merged_exit_status)
}

#[cfg(test)]
mod cli_tests;
