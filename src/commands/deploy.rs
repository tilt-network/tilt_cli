use anyhow::Result;
use clap::Args;
use reqwest::{Client, multipart};
use std::path::Path;

use crate::{
    auth::load_auth_token,
    commands::{Run, build::Build},
    helpers::{get_package_metadata, release_path, url_from_env},
    organization::load_selected_organization_id,
};

/// Deploy your program to tilt network
#[derive(Debug, Args)]
pub struct Deploy;

impl Run for Deploy {
    async fn run(&self) -> Result<()> {
        Build.run().await?;

        let client = Client::new();
        let base_url = url_from_env();
        let url = format!("{base_url}/programs");

        let filename = release_path()?;
        let file_bytes = std::fs::read(Path::new(&filename))?;

        let part = multipart::Part::bytes(file_bytes)
            .file_name("program")
            .mime_str("application/wasm")?;
        let (name, description) = get_package_metadata()?;
        let organization_id = load_selected_organization_id()?;
        let token = load_auth_token()?;

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
