mod auth;
mod commands;
mod custom_lib;
mod entities;
mod helpers;
mod organization;
mod task;

use anyhow::Result;
use clap::Parser;

use commands::Commands;

#[derive(Debug, Parser)]
#[command(name = "tilt", about = "Command Line Application for Tilt network")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(command) => command.run().await,
        None => {
            use clap::CommandFactory;
            Cli::command().print_help().unwrap();
            Ok(())
        }
    }
}
