use anyhow::{Context, Result};
use clap::Args;
use std::{path::Path, process::Command};

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
        let python = if Path::new(".venv/bin/python").exists() {
            ".venv/bin/python"
        } else {
            "python3"
        };

        let status = if Path::new("test_app.py").exists() {
            Command::new(python)
                .args(["-m", "unittest", "test_app.py"])
                .status()
                .context("Failed to execute Python tests from test_app.py")?
        } else {
            Command::new(python)
                .args([
                    "-c",
                    "import app; assert app.WitWorld().execute(b'test') == b'processed: test'",
                ])
                .status()
                .context("Failed to execute Python test. Do you have Python installed?")?
        };

        if !status.success() {
            anyhow::bail!("Python test failed");
        }
        Ok(())
    }
}
