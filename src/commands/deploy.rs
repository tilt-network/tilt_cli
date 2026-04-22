use crate::commands::build::Build;
use crate::utils::{self, ProjectKind, detect_project_kind, go_package_metadata, rust_package_metadata};
use anyhow::Result;
use clap::Args;
use reqwest::{Client, multipart};
use std::{fs, path::Path, time::Duration};

#[derive(Debug, Args)]
pub struct Deploy {}

impl Deploy {
    pub async fn run(&self) -> Result<()> {
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
            ProjectKind::Rust => rust_package_metadata()?,
            ProjectKind::Go => {
                let (name, _) = go_package_metadata()?;
                (name, String::new())
            }
        };

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
            println!("Failed to deploy program: {status}");
        }

        Ok(())
    }
}

fn release_path() -> Result<String> {
    match detect_project_kind()? {
        ProjectKind::Rust => {
            let (name, _) = rust_package_metadata()?;
            Ok(format!(
                "./target/wasm32-wasip2/release/{}.wasm",
                name.replace('-', "_")
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
