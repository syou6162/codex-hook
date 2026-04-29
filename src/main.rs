use clap::{Parser, ValueEnum};
use std::fmt;

#[derive(Clone, ValueEnum)]
#[value(rename_all = "PascalCase")]
pub enum HookEventType {
    PreToolUse,
    PostToolUse,
    SessionStart,
    UserPromptSubmit,
    Stop,
    SubagentStop,
    Notification,
    PreCompact,
}

impl fmt::Display for HookEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self
            .to_possible_value()
            .expect("all variants have a possible value")
            .get_name()
            .to_owned();
        write!(f, "{}", name)
    }
}

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
    if let Some(config) = &cli.config {
        println!("config: {}", config);
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
