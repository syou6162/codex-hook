use clap::Parser;

#[derive(Parser)]
#[command(about = "A hook tool for Codex CLI")]
struct Cli {
    #[arg(long)]
    event: String,

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
        assert_eq!(cli.event, "PreToolUse");
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
        assert_eq!(cli.event, "PostToolUse");
        assert_eq!(cli.config.as_deref(), Some("/path/to/config.yaml"));
    }
}
