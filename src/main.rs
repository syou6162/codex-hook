use clap::Parser;
use codex_hook::{default_config_path, load_config, HookEventType};
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

    let config_path = cli
        .config
        .map(PathBuf::from)
        .unwrap_or_else(default_config_path);

    match load_config(&config_path) {
        Ok(config) => println!("config: {:#?}", config),
        Err(err) => eprintln!("error: {}", err),
    }
}
