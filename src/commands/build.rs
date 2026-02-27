use anyhow::{Context, Result};
use clap::Args;
use std::process::Command;

#[derive(Debug, Args)]
pub struct Build {}

impl Build {
    pub async fn run(&self) -> Result<()> {
        let mut child = Command::new("cargo")
            .args(["build", "--target", "wasm32-wasip2", "--release"])
            .spawn()
            .context("Failed to perform build. Do you have Rust installed?")?;

        let status = child
            .wait()
            .context("Failed to wait for build to complete")?;

        if !status.success() {
            anyhow::bail!("Cargo build failed");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_build() {
        let build = Build {};
        let result = build.run().await;
        assert!(result.is_ok());
    }
}
