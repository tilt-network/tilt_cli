use anyhow::Result;
use clap::Args;
use std::process::Command;

use crate::commands::Run;

#[derive(Debug, Args)]
pub struct Build;

impl Run for Build {
    async fn run(&self) -> Result<()> {
        let mut child = Command::new("cargo")
            .args(["build", "--target", "wasm32-wasip2", "--release"])
            .spawn()
            .expect("Failed to perform build. Do you have rust installed?");

        let status = child.wait().expect("Failed to wait for build to complete");

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
        let build = Build;
        let result = build.run().await;
        assert!(result.is_ok());
    }
}
