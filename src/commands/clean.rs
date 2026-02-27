use crate::commands::Run;
use anyhow::Result;
use clap::Args;
use std::process::Command;

/// Clean your tilt program
#[derive(Debug, Args)]
pub struct Clean;

impl Run for Clean {
    async fn run(&self) -> Result<()> {
        let mut child = Command::new("cargo")
            .arg("clean")
            .spawn()
            .expect("Failed to execute cargo clean. Do you have rust installed");

        let status = child.wait().expect("Failed to wait for cargo clean");

        if !status.success() {
            anyhow::bail!("Cargo clean failed")
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_clean() {
        let clean = Clean;
        assert!(clean.run().await.is_ok());
    }
}
