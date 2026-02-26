mod new;

use anyhow::Result;
use clap::Subcommand;

pub trait Run {
    async fn run(&self) -> Result<()>;
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    New(new::New),
}

impl Commands {
    pub async fn run(&self) -> Result<()> {
        match self {
            Commands::New(command) => command.run().await,
        }
    }
}
