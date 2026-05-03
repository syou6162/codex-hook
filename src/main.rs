mod action;
mod config;
mod error;
mod event;
mod input;
mod matcher;

use action::execute_command;
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

    // TODO(YAS-422): この println! は debug 用。後続チケットで削除し、
    // Codex が期待する JSON を stdout に出力するように差し替える。
    // stdout に非 JSON テキストがあると Codex がパースに失敗するため、
    // 最終的には全ての stdout 出力を Codex 互換 JSON に統一する必要がある。
    println!("event: {}", cli.event);

    let result = match cli.config {
        Some(path) => load_config(&PathBuf::from(path)),
        None => load_config_or_default(&default_config_path()),
    };

    // TODO(YAS-422): config の debug print も同様に後続チケットで削除する。
    let config = match result {
        Ok(config) => {
            println!("config: {:#?}", config);
            config
        }
        Err(err) => {
            eprintln!("error: {}", err);
            std::process::exit(1);
        }
    };

    // TODO(YAS-419/YAS-422): Action 実装後、マッチ結果に基づいて
    // Codex 互換の JSON（PreToolUseOutput）を stdout に出力する。
    // 出力形式: {"continue":true,"hookSpecificOutput":{"hookEventName":"PreToolUse","permissionDecision":"allow",...}}
    // exit 0 + 空 stdout は「許可」として扱われる。
    match cli.event {
        HookEventType::PreToolUse => match read_pre_tool_use_input(std::io::stdin()) {
            Ok(input) => {
                match filter_pre_tool_use_hooks(&config.pre_tool_use, &input.tool_name) {
                    Ok(matched) => {
                        // TODO(YAS-422): この debug print を Codex 互換 JSON 出力に差し替える。
                        println!("tool_name: {}", input.tool_name);
                        println!("matched_hooks: {}", matched.len());
                        for hook in &matched {
                            println!("  matcher: {:?}, actions: {:?}", hook.matcher, hook.actions);
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
                                    // TODO(YAS-422): ActionType::Output を実装する。
                                    ActionType::Output => {}
                                }
                            }
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
