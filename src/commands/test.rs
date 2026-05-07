use anyhow::{Context, Result};
use clap::Args;
use std::process::Command;

use crate::utils::{ProjectKind, detect_project_kind};

#[derive(Debug, Args)]
pub struct Test;

impl Test {
    pub async fn run(&self) -> Result<()> {
        match detect_project_kind()? {
            ProjectKind::Rust => self.test_rust(),
            ProjectKind::Go => self.test_go(),
            ProjectKind::Python => self.test_python(),
        }
    }

    fn test_rust(&self) -> Result<()> {
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

    fn test_go(&self) -> Result<()> {
        let status = Command::new("go")
            .args(["test", "./..."])
            .status()
            .context("Failed to execute go test. Do you have go installed?")?;

        if !status.success() {
            anyhow::bail!("Go test failed");
        }
        Ok(())
    }

    fn test_python(&self) -> Result<()> {
        let status = Command::new("python")
            .args(["-m", "pytest"])
            .status()
            .context(
                "Failed to execute pytest. Do you have it installed? Run: pip install pytest",
            )?;

        if !status.success() {
            anyhow::bail!("pytest failed");
        }
        Ok(())
    }
}
