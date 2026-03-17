use crate::commands::build::Build;
use crate::utils;
use anyhow::{Context, Result, anyhow};
use clap::Args;
use reqwest::{Client, multipart};
use serde_json::Value as JsonValue;
use std::{env, fs, path::Path, process::Command, time::Duration};
use toml::Value as TomlValue;

#[derive(Debug, Args)]
pub struct Deploy {
    /// Path to compiled wasm artifact. Overrides auto-detected path.
    #[arg(long)]
    artifact: Option<String>,
}

enum ProjectKind {
    Rust,
    JavaScript,
}

impl Deploy {
    pub async fn run(&self) -> Result<()> {
        let project_kind = detect_project_kind()?;

        // ensure the program is built before deploying.
        match project_kind {
            ProjectKind::Rust => Build {}.run().await?,
            ProjectKind::JavaScript => build_js_project()?,
        }

        let client = Client::builder().timeout(Duration::from_secs(5)).build()?;
        let base_url = utils::url_from_env();
        let url = format!("{base_url}/programs");

        let (mut filename, name, description) = match project_kind {
            ProjectKind::Rust => {
                let (name, description) = get_rust_package_metadata()?;
                let filename = rust_release_path(&name);
                (filename, name, description)
            }
            ProjectKind::JavaScript => {
                let (name, description) = get_js_package_metadata()?;
                let filename = js_release_path()?;
                (filename, name, description)
            }
        };

        if let Some(artifact) = &self.artifact {
            filename = artifact.clone();
        }

        let file_bytes = fs::read(Path::new(&filename))
            .with_context(|| format!("Failed to read compiled artifact at {}", filename))?;

        let part = multipart::Part::bytes(file_bytes)
            .file_name("program")
            .mime_str("application/wasm")?;

        let organization_id = utils::load_selected_organization_id()?;
        let token = utils::load_auth_token()?;

        let form = multipart::Form::new()
            .text("name", name)
            .text("description", description)
            .text("organization_id", organization_id)
            .part("program", part);

        let response = client
            .post(&url)
            .bearer_auth(&token)
            .multipart(form)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await.unwrap_or_default();

        if status.is_success() {
            println!("Program deployed successfully");
        } else if body.is_empty() {
            println!("Failed to deploy program: {}", status);
        } else {
            println!("Failed to deploy program: {} - {}", status, body);
        }

        Ok(())
    }
}

fn detect_project_kind() -> Result<ProjectKind> {
    let cwd = env::current_dir().context("error getting current directory")?;
    let cargo_toml = cwd.join("Cargo.toml");
    let package_json = cwd.join("package.json");

    if cargo_toml.exists() {
        return Ok(ProjectKind::Rust);
    }

    if package_json.exists() {
        return Ok(ProjectKind::JavaScript);
    }

    Err(anyhow!(
        "Unsupported project type. Expected Cargo.toml (Rust) or package.json (JavaScript) in current directory."
    ))
}

fn rust_release_path(name: &str) -> String {
    format!(
        "./target/wasm32-wasip2/release/{}.wasm",
        name.replace("-", "_")
    )
}

fn js_release_path() -> Result<String> {
    let package_json_path = env::current_dir()
        .context("error getting current directory")?
        .join("package.json");

    let package_json_content =
        fs::read_to_string(&package_json_path).context("failed reading package.json")?;

    let parsed: JsonValue = serde_json::from_str(&package_json_content)
        .context("failed parsing package.json as JSON")?;

    if let Some(build_script) = parsed
        .get("scripts")
        .and_then(|s| s.get("build"))
        .and_then(|v| v.as_str())
    {
        if build_script.contains("dist/index.component.wasm") {
            return Ok("./dist/index.component.wasm".to_string());
        }

        if build_script.contains("dist/index.wasm") {
            return Ok("./dist/index.wasm".to_string());
        }
    }

    // Default for JS projects in this repository.
    Ok("./dist/index.component.wasm".to_string())
}

fn get_rust_package_metadata() -> Result<(String, String)> {
    let cargo_toml_path = env::current_dir()
        .context("error getting current directory")?
        .join("Cargo.toml");

    let cargo_toml_content = fs::read_to_string(cargo_toml_path)?;
    let parsed: TomlValue = cargo_toml_content.parse::<TomlValue>()?;

    let package = parsed
        .get("package")
        .and_then(|v| v.as_table())
        .ok_or_else(|| anyhow!("Missing [package] section"))?;

    let name = package
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'name' in [package]"))?
        .to_string();

    let description = package
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Ok((name, description))
}

fn get_js_package_metadata() -> Result<(String, String)> {
    let package_json_path = env::current_dir()
        .context("error getting current directory")?
        .join("package.json");

    let package_json_content =
        fs::read_to_string(package_json_path).context("failed reading package.json")?;

    let parsed: JsonValue = serde_json::from_str(&package_json_content)
        .context("failed parsing package.json as JSON")?;

    let name = parsed
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'name' in package.json"))?
        .to_string();

    let description = parsed
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Ok((name, description))
}

fn build_js_project() -> Result<()> {
    // Prefer pnpm, fallback to npm.
    let pnpm_status = Command::new("pnpm").args(["run", "build"]).status();

    match pnpm_status {
        Ok(status) if status.success() => return Ok(()),
        Ok(_) => {
            // pnpm existed but failed; try npm as fallback anyway.
        }
        Err(_) => {
            // pnpm not available or could not run; try npm fallback.
        }
    }

    let npm_status = Command::new("npm")
        .args(["run", "build"])
        .status()
        .context("Failed to run JS build. Ensure pnpm or npm is installed.")?;

    if !npm_status.success() {
        anyhow::bail!("JavaScript build failed");
    }

    Ok(())
}
