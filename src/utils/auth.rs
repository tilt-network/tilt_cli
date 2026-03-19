use crate::utils::{tilt_dir, url_from_env};
use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;
use std::{collections::HashMap, fs, time::Duration};

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: Option<String>,
}

/// Return the authentication token previously saved by the sign-in command.
pub fn load_auth_token() -> Result<String> {
    let path = tilt_dir()?.join("auth_token");
    let token = fs::read_to_string(&path).with_context(|| {
        format!(
            "Failed to read auth token from {:?}. Please run `tilt signin`.",
            path
        )
    })?;
    Ok(token.trim().to_string())
}

/// Return the refresh token previously saved by the sign-in command.
pub fn load_refresh_token() -> Result<String> {
    let path = tilt_dir()?.join("refresh_token");
    let token = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read refresh token from {:?}.", path))?;
    Ok(token.trim().to_string())
}

/// Saves the authentication token to a file.
pub fn save_auth_token(token: &str) -> Result<()> {
    let dir = tilt_dir()?;
    fs::create_dir_all(&dir)?;
    fs::write(dir.join("auth_token"), token)?;
    Ok(())
}

/// Saves the refresh token to a file.
pub fn save_refresh_token(token: &str) -> Result<()> {
    let dir = tilt_dir()?;
    fs::create_dir_all(&dir)?;
    fs::write(dir.join("refresh_token"), token)?;
    Ok(())
}

/// Clears the authentication files.
pub fn clear_auth_files() -> Result<()> {
    let dir = tilt_dir()?;
    let _ = fs::remove_file(dir.join("auth_token"));
    let _ = fs::remove_file(dir.join("refresh_token"));
    let _ = fs::remove_file(dir.join("organization_id_selected"));
    Ok(())
}

/// Refreshes the authentication token using the refresh token,
/// saving the new tokens and returning the new access token.
/// Clears the auth files if the refresh fails.
pub async fn refresh_auth_token() -> Result<String> {
    let refresh_token = match load_refresh_token() {
        Ok(t) => t,
        Err(_) => {
            clear_auth_files()?;
            anyhow::bail!("No refresh token found. Please sign in again.");
        }
    };

    let mut payload = HashMap::new();
    payload.insert("grant_type", "refresh_token");
    payload.insert("refresh_token", &refresh_token);
    payload.insert("client_id", "tilt-cli");
    payload.insert("client_secret", "");

    let client = Client::builder().timeout(Duration::from_secs(15)).build()?;
    let res = client
        .post(format!("{}/oauth/token", url_from_env()))
        .form(&payload)
        .send()
        .await?;

    if res.status().is_success() {
        let token_resp: TokenResponse = res.json().await?;
        save_auth_token(&token_resp.access_token)?;
        if let Some(rt) = token_resp.refresh_token {
            save_refresh_token(&rt)?;
        }
        Ok(token_resp.access_token)
    } else {
        clear_auth_files()?;
        anyhow::bail!("Session expired. Please sign in again.");
    }
}
