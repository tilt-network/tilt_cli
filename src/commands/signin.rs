use anyhow::Result;
use clap::Args;

use crate::auth::sign_in;
use crate::commands::Run;

/// Sign in to your tilt account with your private key
#[derive(Debug, Args)]
pub struct Signin {
    ///Your tilt private key. You can find it in the Tilt Console at right after choosing your organization in User -> Settings.
    #[arg(long = "secret_key", short = 'k')]
    pub secret_key: String,
}

impl Run for Signin {
    async fn run(&self) -> Result<()> {
        sign_in(&self.secret_key).await?;
        Ok(())
    }
}
