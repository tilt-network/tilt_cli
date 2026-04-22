use anyhow::{Context, Result};
use clap::Args;
use std::{fs, path::Path, process::Command};

use crate::utils::{ProjectKind, detect_project_kind};

fn tinygo_path() -> String {
    let home = dirs::home_dir().unwrap_or_default();
    [
        home.join("tinygo/bin/tinygo").to_string_lossy().into_owned(),
        "/usr/local/tinygo/bin/tinygo".to_string(),
    ]
    .into_iter()
    .find(|p| Path::new(p).exists())
    .unwrap_or_else(|| "tinygo".to_string())
}

const WASI_REACTOR_ADAPTER: &[u8] =
    include_bytes!("../../static/go/wasi_preview1_reactor.wasm");

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
        let tinygo = tinygo_path();
        let status = Command::new(&tinygo)
            .args(["build", "-o", "tilt.wasm", "-target=wasip1", "."])
            .status()
            .context("Failed to build. Is TinyGo installed? https://tinygo.org/getting-started/install/")?;

        if !status.success() {
            anyhow::bail!("TinyGo build failed");
        }

        let adapter_path = std::env::temp_dir().join("wasi_preview1_reactor.wasm");
        fs::write(&adapter_path, WASI_REACTOR_ADAPTER)
            .context("Failed to write WASI adapter")?;

        let adapt_arg = format!("wasi_snapshot_preview1={}", adapter_path.display());

        let embed_ok = Command::new("wasm-tools")
            .args([
                "component",
                "embed",
                "--world",
                "tilt",
                "wit/",
                "tilt.wasm",
                "-o",
                "tilt-embedded.wasm",
            ])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);

        let input = if embed_ok { "tilt-embedded.wasm" } else { "tilt.wasm" };

        match Command::new("wasm-tools")
            .args([
                "component",
                "new",
                input,
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
}
