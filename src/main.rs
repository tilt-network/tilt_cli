mod commands;
mod utils;

use anyhow::Result;
use clap::Parser;
use commands::Commands;

#[derive(Debug, Parser)]
#[command(name = "tilt", about = "Command Line Application for Tilt network")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    cli.command.run().await
}
