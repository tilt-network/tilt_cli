use anyhow::Result;
use clap::Args;

use crate::auth::sign_in;
use crate::commands::Run;

#[derive(Debug, Args)]
pub struct Signin {
    #[arg(long = "secret_key", short = 'k')]
    pub secret_key: String,
}

impl Run for Signin {
    async fn run(&self) -> Result<()> {
        sign_in(&self.secret_key).await?;
        Ok(())
    }
}
