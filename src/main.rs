mod action;
mod condition;
mod config;
mod error;
mod event;
mod input;
mod matcher;
mod template;

use action::{build_output_json, execute_command, merge_exit_status};
use clap::Parser;
use condition::{evaluate_conditions, ConditionContext};
use config::{default_config_path, load_config, load_config_or_default, ActionType};
use event::HookEventType;
use input::read_pre_tool_use_input_with_raw;
use matcher::filter_pre_tool_use_hooks;
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

    let result = match cli.config {
        Some(path) => load_config(&PathBuf::from(path)),
        None => load_config_or_default(&default_config_path()),
    };

    let config = match result {
        Ok(config) => config,
        Err(err) => {
            eprintln!("error: {}", err);
            std::process::exit(1);
        }
    };

    match cli.event {
        HookEventType::PreToolUse => match read_pre_tool_use_input_with_raw(std::io::stdin()) {
            Ok((input, raw_bytes)) => {
                let jaq_input = match jaq_json::read::parse_single(&raw_bytes) {
                    Ok(val) => val,
                    Err(err) => {
                        eprintln!("error: failed to parse input for templates: {:?}", err);
                        std::process::exit(1);
                    }
                };
                match filter_pre_tool_use_hooks(&config.pre_tool_use, &input.tool_name) {
                    Ok(matched) => {
                        let mut output_messages: Vec<String> = Vec::new();
                        let mut merged_exit_status: Option<i32> = None;
                        let cond_ctx = ConditionContext {
                            cwd: &input.base.cwd,
                            tool_input: &input.tool_input,
                            permission_mode: input.permission_mode.as_deref(),
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
                                            match execute_command(&cmd) {
                                                Ok(status) => {
                                                    if !status.success() {
                                                        let code = status.code().unwrap_or(1);
                                                        eprintln!(
                                                            "error: command failed with exit code {}: {}",
                                                            code, cmd
                                                        );
                                                        std::process::exit(2);
                                                    }
                                                }
                                                Err(err) => {
                                                    eprintln!(
                                                        "error: failed to execute command: {}: {}",
                                                        cmd, err
                                                    );
                                                    std::process::exit(2);
                                                }
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
                                            merged_exit_status =
                                                merge_exit_status(merged_exit_status, code);
                                        }
                                    }
                                }
                            }
                        }
                        if !output_messages.is_empty() {
                            let merged = output_messages.join("\n");
                            println!("{}", build_output_json(&merged));
                        }
                        if let Some(code) = merged_exit_status {
                            std::process::exit(code);
                        }
                    }
                    Err(err) => {
                        eprintln!("error: invalid matcher regex: {}", err);
                        std::process::exit(1);
                    }
                }
            }
            Err(err) => {
                eprintln!("error: {}", err);
                std::process::exit(1);
            }
        },
        _ => {
            // TODO: 他のイベントタイプの入力パース・処理は後続チケットで実装する。
        }
    }
}

#[cfg(test)]
mod cli_tests;
