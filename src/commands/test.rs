use anyhow::{Context, Result};
use clap::Args;
use std::process::Command;

#[derive(Debug, Args)]
pub struct Test;

impl Test {
    pub async fn run(&self) -> Result<()> {
        let mut child = Command::new("cargo")
            .arg("test")
            .spawn()
            .context("Failed to execute cargo test. Do you have rust installed?")?;

        let status = child
            .wait()
            .context("Failed to wait for tests to complete")?;

        if !status.success() {
            anyhow::bail!("Cargo test failed");
        }
        Ok(())
    }
}
