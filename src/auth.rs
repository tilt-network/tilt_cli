use std::fs::{create_dir_all, read_to_string, write};
use std::io::{self, ErrorKind};

use reqwest::Client;
use serde::Deserialize;

use crate::helpers::url_from_env;
use crate::organization::{fetch_and_save_organization_ids, save_selected_organization_id};

#[derive(Deserialize)]
struct SignInResponse {
    token: String,
}

fn save_auth_token(token: &str) -> io::Result<()> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Failed to get home directory"))?;

    let tilt_dir = home_dir.join(".tilt");
    let token_path = tilt_dir.join("auth_token");

    create_dir_all(&tilt_dir)?;
    write(token_path, token)?;

    Ok(())
}

fn create_tilt_directory() -> io::Result<()> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Failed to get home directory"))?;
    let tilt_dir = home_dir.join(".tilt");
    create_dir_all(tilt_dir)?;
    Ok(())
}

pub async fn sign_in(secret_key: &str) -> Result<String, Box<dyn std::error::Error>> {
    create_tilt_directory()?;
    let client = Client::new();
    let base_url = url_from_env();
    let response = client
        .post(format!("{base_url}/sign_in/api_key"))
        .json(&serde_json::json!({ "secret_key": secret_key }))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(format!("Sign-in failed with status: {}", response.status()).into());
    }

    let data: SignInResponse = response.json().await?;
    let orgs_id = fetch_and_save_organization_ids(data.token.clone()).await?;
    println!("Select an organization:");
    for (i, org) in orgs_id.iter().enumerate() {
        println!("{}: {} ({})", i + 1, org.name, org.id)m
    }

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let choice: usize = input.trim().parse().unwrap_or(0);
    if choice == 0 || choice > orgs_id.len() {
        return Err("Invalid choice".into());
    }
    let organization = &orgs_id[choice - 1];
    println!(
        "Selected organization: {} ({})",
        organization.name, organization.id
    );

    save_selected_organization_id(organization.id.clone())?;

    let response = client
        .post(format!("{base_url}/organizations/select",))
        .json(&serde_json::json!({ "organization_id": organization.id }))
        .bearer_auth(&data.token)
        .send()
        .await?;
    let data: SignInResponse = response.json().await?;
    save_auth_token(&data.token).unwrap();
    Ok(data.token)
}

pub fn load_auth_token() -> io::Result<String> {
    let path = dirs::home_dir()
        .map(|p| p.join(".tilt/auth_token"))
        .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "Home directory not found"))?;

    read_to_string(path).map(|s| s.trim().to_string())
}
