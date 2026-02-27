use crate::utils;
use crate::utils::tilt_dir;
use anyhow::{Result, bail};
use clap::Args;
use reqwest::Client;
use serde::Deserialize;
use std::fs;
use std::time::Duration;

#[derive(Debug, Args)]
pub struct Signin {
    /// Your tilt private key. You can find it in the Tilt Console at right after choosing your organization in User -> Settings.
    #[arg(short = 'k', long)]
    secret_key: String,
}

impl Signin {
    pub async fn run(&self) -> Result<()> {
        let client = Client::builder().timeout(Duration::from_secs(5)).build()?;
        let base_url = utils::url_from_env();
        let response = client
            .post(format!("{base_url}/sign_in/api_key"))
            .json(&serde_json::json!({ "secret_key": self.secret_key }))
            .send()
            .await?;

        if !response.status().is_success() {
            bail!("Sign-in failed with status: {}", response.status());
        }

        let data = response.json::<SignInResponse>().await?;
        save_selected_organization_id(&data.organization.id)?;
        save_auth_token(&data.token)?;
        println!("Authenticated successfully.");
        Ok(())
    }
}

#[derive(Deserialize)]
struct OrganizationId {
    id: String,
}

#[derive(Deserialize)]
struct SignInResponse {
    token: String,
    organization: OrganizationId,
}

/// Saves the authentication token to a file in the Tilt directory.
///
/// Only the sign-in command should be able to call this function, as it is responsible for
/// authenticating the user and obtaining the token.
fn save_auth_token(token: &str) -> Result<()> {
    let dir = tilt_dir()?;
    fs::create_dir_all(&dir)?;
    fs::write(dir.join("auth_token"), token)?;
    Ok(())
}

/// Saves the selected organization ID to a file in the Tilt directory.
///
/// Only the sign-in command should be able to call this function, as it is responsible for
/// authenticating the user and obtaining the organization ID.
fn save_selected_organization_id(id: &str) -> Result<()> {
    let path = tilt_dir()?.join("organization_id_selected");
    fs::write(path, id)?;
    Ok(())
}
