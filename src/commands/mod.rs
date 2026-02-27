mod build;
mod clean;
mod deploy;
mod list;
mod new;
mod signin;
mod test;

use anyhow::Result;
use clap::Subcommand;

pub trait Run {
    async fn run(&self) -> Result<()>;
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    New(new::New),
    Build(build::Build),
    Clean(clean::Clean),
    Deploy(deploy::Deploy),
    List(list::List),
    Signin(signin::Signin),
    Test(test::Test),
}

impl Commands {
    pub async fn run(&self) -> Result<()> {
        match self {
            Commands::New(command) => command.run().await,
            Commands::Build(command) => command.run().await,
            Commands::Clean(command) => command.run().await,
            Commands::Deploy(command) => command.run().await,
            Commands::List(command) => command.run().await,
            Commands::Signin(command) => command.run().await,
            Commands::Test(command) => command.run().await,
        }
    }
}
