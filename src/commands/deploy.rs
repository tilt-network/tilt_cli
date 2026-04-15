use crate::{commands::build::Build, utils::detect_project_kind};
use crate::utils::{self, ProjectKind, go_package_metadata};
use anyhow::{Context, Ok, Result, anyhow};
use clap::Args;
use reqwest::{Client, multipart};
use std::{env, fs, path::Path, time::Duration};
use toml::Value;

#[derive(Debug, Args)]
pub struct Deploy {}

impl Deploy {
    pub async fn run(&self) -> Result<()> {
        // ensure the program is built before deploying.
        Build {}.run().await?;

        let client = Client::builder().timeout(Duration::from_secs(5)).build()?;
        let base_url = utils::url_from_env();
        let url = format!("{base_url}/programs");

        let filename = release_path()?;
        let file_bytes = fs::read(Path::new(&filename))?;

        let part = multipart::Part::bytes(file_bytes)
            .file_name("program")
            .mime_str("application/wasm")?;
        let (name, description) = match detect_project_kind()? {
            ProjectKind::Rust => rust_package_metadata()
            ProjectKind::Go => go_package_metadata()


        }
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
        if status.is_success() {
            println!("Program deployed successfully");
        } else {
            println!("Failed to deploy program: {}", status);
        }

        Ok(())
    }
}

fn release_path() -> Result<String> {
    match detect_project_kind()? {
        ProjectKind::Rust => {
            let (name, _) = get_rust_metadata()?;
            Ok(format!(
                "./target/wasm32-wasip2/release/{}.wasm",
                name.replace("-", "_")
            ))
        }
        ProjectKind::Go => {
            if Path::new("tilt:app@0.1.0.wasm").exists() {
                Ok("tilt:app@0.1.0.wasm".to_string())
            } else {
                Ok("tilt.wasm".to_string())
            }
        }
        }
    }


fn get_rust_metadata() -> Result<(String, String)> {
    let cargo_toml_path = env::current_dir()
        .context("error getting current directory")?
        .join("Cargo.toml");
    let cargo_toml_content = fs::read_to_string(cargo_toml_path)?;
    let parsed: Value = cargo_toml_content.parse::<Value>()?;

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
