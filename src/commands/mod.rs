mod build;
mod deploy;
mod list;
mod new;
mod signin;
mod test;

use anyhow::Result;
use clap::Subcommand;

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Create a new tilt program.
    New(new::New),
    /// Build your tilt program.
    Build(build::Build),
    /// Deploy your program to tilt network.
    Deploy(deploy::Deploy),
    /// List your tilt programs.
    List(list::List),
    /// Sign in to your tilt account with your private key.
    Signin(signin::Signin),
    /// Run the tests on your program.
    Test(test::Test),
}

impl Commands {
    pub async fn run(&self) -> Result<()> {
        match self {
            Commands::New(command) => command.run().await,
            Commands::Build(command) => command.run().await,
            Commands::Deploy(command) => command.run().await,
            Commands::List(command) => command.run().await,
            Commands::Signin(command) => command.run().await,
            Commands::Test(command) => command.run().await,
        }
    }
}
