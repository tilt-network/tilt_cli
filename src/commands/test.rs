use anyhow::Result;
use clap::Args;
use std::process::Command;

use crate::commands::Run;

#[derive(Debug, Args)]
pub struct Test;

impl Run for Test {
    async fn run(&self) -> Result<()> {
        let mut child = Command::new("cargo")
            .arg("test")
            .spawn()
            .expect("Failed to execute cargo test. Do you have rust installed?");

        let status = child.wait().expect("Failed to wait for tests to complete");

        if !status.success() {
            anyhow::bail!("Cargo test failed");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_test() {
        let test = Test;
        assert!(test.run().await.is_ok());
    }
}
