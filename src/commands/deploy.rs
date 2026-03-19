use crate::commands::build::Build;
use crate::utils;
use anyhow::{Context, Result, anyhow};
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
        let file_bytes = fs::read(Path::new(&filename))
            .with_context(|| format!("Failed to read compiled WASM file at {}", filename))?;

        let (name, description) = get_package_metadata()?;
        let organization_id = utils::load_selected_organization_id()?;
        let mut token = utils::load_auth_token()?;

        let part = multipart::Part::bytes(file_bytes.clone())
            .file_name("program")
            .mime_str("application/wasm")?;

        let mut form = multipart::Form::new()
            .text("name", name.clone())
            .text("description", description.clone())
            .part("program", part);

        if !organization_id.is_empty() {
            form = form.text("organization_id", organization_id.clone());
        }

        let mut response = client
            .post(&url)
            .bearer_auth(&token)
            .multipart(form)
            .send()
            .await?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            println!("Token expired, refreshing...");
            token = utils::refresh_auth_token().await?;

            let part = multipart::Part::bytes(file_bytes)
                .file_name("program")
                .mime_str("application/wasm")?;

            let mut form = multipart::Form::new()
                .text("name", name)
                .text("description", description)
                .part("program", part);

            if !organization_id.is_empty() {
                form = form.text("organization_id", organization_id);
            }

            response = client
                .post(&url)
                .bearer_auth(&token)
                .multipart(form)
                .send()
                .await?;
        }

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
    let (name, _) = get_package_metadata()?;
    Ok(format!(
        "./target/wasm32-wasip2/release/{}.wasm",
        name.replace("-", "_")
    ))
}

fn get_package_metadata() -> Result<(String, String)> {
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
