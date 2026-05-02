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
mod cli_tests;
