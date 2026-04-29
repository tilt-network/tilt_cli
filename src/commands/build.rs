use anyhow::{Context, Result};
use clap::Args;
use std::{fs, path::Path, process::Command};

use crate::utils::{ProjectKind, detect_project_kind};

const WASI_REACTOR_ADAPTER: &[u8] = include_bytes!("../../static/go/wasi_preview1_reactor.wasm");

#[derive(Debug, Args)]
pub struct Build {}

impl Build {
    pub async fn run(&self) -> Result<()> {
        match detect_project_kind()? {
            ProjectKind::Rust => self.build_rust(),
            ProjectKind::Go => self.build_go(),
            ProjectKind::Python => self.build_python(),
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
        let status = Command::new("go")
            .args(["build", "-o", "tilt.wasm", "."])
            .env("GOOS", "wasip1")
            .env("GOARCH", "wasm")
            .status()
            .context("Failed to perform build. Do you have Go installed?")?;

        if !status.success() {
            anyhow::bail!("Go build failed");
        }

        let adapter_path = std::env::temp_dir().join("wasi_preview1_reactor.wasm");
        fs::write(&adapter_path, WASI_REACTOR_ADAPTER).context("Failed to write WASI adapter")?;

        let adapt_arg = format!("wasi_snapshot_preview1={}", adapter_path.display());

        match Command::new("wasm-tools")
            .args([
                "component",
                "new",
                "tilt.wasm",
                "--adapt",
                &adapt_arg,
                "-o",
                "tilt:app@0.1.0.wasm",
            ])
            .status()
        {
            Ok(s) if s.success() => println!("Component wrapped: tilt:app@0.1.0.wasm"),
            _ => println!("wasm-tools not found or failed — skipping component wrap"),
        }

        Ok(())
    }

    fn build_python(&self) -> Result<()> {
        let status = if Path::new(".venv/bin/python").exists() {
            // Call the venv interpreter directly so builds still work if the
            // console-script wrapper has a stale absolute shebang.
            Command::new(".venv/bin/python")
                .args([
                    "-c",
                    "import sys; from componentize_py import script; sys.argv = ['componentize-py', '-d', 'wit', '-w', 'tilt', 'componentize', 'app', '-o', 'tilt:app@0.1.0.wasm']; raise SystemExit(script())",
                ])
                .status()
                .context(
                    "Failed to perform Python build. Is componentize-py installed in .venv?",
                )?
        } else {
            Command::new("componentize-py")
                .args([
                    "-d",
                    "wit",
                    "-w",
                    "tilt",
                    "componentize",
                    "app",
                    "-o",
                    "tilt:app@0.1.0.wasm",
                ])
                .status()
                .context("Failed to perform Python build. Do you have componentize-py installed?")?
        };

        if !status.success() {
            anyhow::bail!("componentize-py build failed");
        }

        Ok(())
    }
}
