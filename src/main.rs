mod config;
mod error;
mod event;

use clap::Parser;
use config::{default_config_path, load_config, load_config_or_default};
use event::HookEventType;
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
    println!("event: {}", cli.event);

    let result = match cli.config {
        Some(path) => load_config(&PathBuf::from(path)),
        None => load_config_or_default(&default_config_path()),
    };

    match result {
        Ok(config) => println!("config: {:#?}", config),
        Err(err) => {
            eprintln!("error: {}", err);
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_event_flag() {
        let cli = Cli::parse_from(["codex-hook", "--event", "PreToolUse"]);
        assert!(matches!(cli.event, HookEventType::PreToolUse));
        assert!(cli.config.is_none());
    }

    #[test]
    fn parse_event_and_config_flags() {
        let cli = Cli::parse_from([
            "codex-hook",
            "--event",
            "PostToolUse",
            "--config",
            "/path/to/config.yaml",
        ]);
        assert!(matches!(cli.event, HookEventType::PostToolUse));
        assert_eq!(cli.config.as_deref(), Some("/path/to/config.yaml"));
    }

    #[test]
    fn reject_invalid_event() {
        let result = Cli::try_parse_from(["codex-hook", "--event", "Invalid"]);
        assert!(result.is_err());
    }

    #[test]
    fn reject_lowercase_event() {
        let result = Cli::try_parse_from(["codex-hook", "--event", "pretooluse"]);
        assert!(result.is_err());
    }

    #[test]
    fn parse_all_event_types() {
        let events = [
            "PreToolUse",
            "PostToolUse",
            "SessionStart",
            "UserPromptSubmit",
            "Stop",
            "SubagentStop",
            "Notification",
            "PreCompact",
        ];
        for event in events {
            let cli = Cli::parse_from(["codex-hook", "--event", event]);
            assert_eq!(cli.event.to_string(), event);
        }
    }
}
