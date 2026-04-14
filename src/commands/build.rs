use anyhow::{Context, Result};
use clap::Args;
use std::{env::args, process::Command};

use crate::utils::{ProjectKind, detect_project_kind};

#[derive(Debug, Args)]
pub struct Build {}

impl Build {
    pub async fn run(&self) -> Result<()> {
    match detect_project_kind()? {
        ProjectKind::Rust => self.build_rust(),
        ProjectKind::Go => self.build_go(),
    }
    }

    fn build_rust(&self) -> Result<()> {
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

    fn build_go(&self) -> Result<()> {
        .args(["build", "-o", "tilt.wasm", "."])
            .env("GOOS", "wasip1")
            .env("GOARCH", "wasm")
            .status()
            .context("Failed to perfom build. Do you have Go installed?")?;
        
        if !status.success() {
            anyhow::bail!("Go build failed");  
        }
        
        match Command::new("wasm-tools")
            .args(["component", "new", "tilt.wasm", "-o", "tilt:app@0.1.0.wasm"])
            .status()
        {
            Ok(s) if s.success() => println!("Component wrapped: tilt:app@0.1.0.wasm"),
            Ok(_) => anyhow::bail!("Failed to wrap component"),
            Err(_) => anyhow::bail!("Failed to run wasm-tools"),
        }
        Ok(())
    }
}
