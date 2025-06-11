use std::fs::{create_dir_all, read_to_string, write};
use std::io::{self, ErrorKind};

use reqwest::Client;
use serde::Deserialize;

use crate::organization::fetch_and_save_organization_ids;

#[derive(Deserialize)]
struct SignInResponse {
    token: String,
}

fn save_auth_token(token: &str) -> io::Result<()> {
    let home_dir = dirs::home_dir().ok_or_else(|| {
        io::Error::new(io::ErrorKind::NotFound, "Failed to get home directory")
    })?;

    let tilt_dir = home_dir.join(".tilt");
    let token_path = tilt_dir.join("auth_token");

    create_dir_all(&tilt_dir)?;
    write(token_path, token)?;

    Ok(())
}

pub async fn sign_in(
    email: &str,
    password: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let response = client
        .post("https://your.api.endpoint/auth/sign-in")
        .json(&serde_json::json!({ "email": email, "password": password }))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("Sign-in failed with status: {}", response.status()).into());
    }

    let data: SignInResponse = response.json().await?;
    fetch_and_save_organization_ids(data.token.clone()).await?;
    save_auth_token(&data.token).unwrap();
    Ok(data.token)
}

pub fn load_auth_token() -> io::Result<String> {
    let path = dirs::home_dir()
        .map(|p| p.join(".tilt/auth_token"))
        .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "Home directory not found"))?;

    read_to_string(path).map(|s| s.trim().to_string())
}

