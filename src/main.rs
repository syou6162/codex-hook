mod action;
mod config;
mod error;
mod event;
mod input;
mod matcher;

use action::{build_output_json, execute_command};
use clap::Parser;
use config::{default_config_path, load_config, load_config_or_default, ActionType};
use event::HookEventType;
use input::read_pre_tool_use_input;
use matcher::filter_pre_tool_use_hooks;
use std::path::PathBuf;

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
        HookEventType::PreToolUse => match read_pre_tool_use_input(std::io::stdin()) {
            Ok(input) => match filter_pre_tool_use_hooks(&config.pre_tool_use, &input.tool_name) {
                Ok(matched) => {
                    let mut output_emitted = false;
                    for hook in &matched {
                        for action in &hook.actions {
                            match action.action_type {
                                ActionType::Command => {
                                    if let Some(cmd) = &action.command {
                                        match execute_command(cmd) {
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
                                    if output_emitted {
                                        eprintln!(
                                            "warning: multiple output actions matched; \
                                             only the first output is sent to stdout \
                                             (Codex expects a single JSON object)"
                                        );
                                        continue;
                                    }
                                    if let Some(msg) = &action.message {
                                        println!("{}", build_output_json(msg));
                                        output_emitted = true;
                                    } else {
                                        eprintln!("error: output action has no message");
                                    }
                                    if let Some(code) = action.exit_status {
                                        std::process::exit(code);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(err) => {
                    eprintln!("error: invalid matcher regex: {}", err);
                    std::process::exit(1);
                }
            },
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
