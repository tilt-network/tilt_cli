use std::fs::{read_to_string, write};
use std::io::{self, ErrorKind};

use reqwest::Client;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Organization {
    pub id: String,
}

#[derive(Deserialize)]
pub struct OrganizationsResponse {
    pub data: Vec<Organization>,
}

fn save_all_organization_ids(ids: &[String]) -> io::Result<()> {
    let path = dirs::home_dir()
        .map(|p| p.join(".tilt/organization_id"))
        .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "Home directory not found"))?;

    let contents = ids.join("\n");
    write(path, contents)
}

pub async fn fetch_and_save_organization_ids(
    token: String,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();

    let res = client
        .get("https://production.tilt.rest/organizations")
        .bearer_auth(token)
        .send()
        .await?;

    if !res.status().is_success() {
        return Err(format!("Request failed: {}", res.status()).into());
    }

    let orgs: OrganizationsResponse = res.json().await?;
    if orgs.data.is_empty() {
        return Err("No organizations found".into());
    }
    let first_org_id = orgs.data[0].id.clone();

    let ids: Vec<String> = orgs.data.into_iter().map(|org| org.id).collect();
    save_all_organization_ids(&ids)?;
    println!("Saved organization IDs");

    Ok(first_org_id)
}

pub fn load_organization_id(index: usize) -> io::Result<String> {
    let path = dirs::home_dir()
        .map(|p| p.join(".tilt/organization_id"))
        .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "Home directory not found"))?;

    let contents = read_to_string(path)?;
    contents
        .lines()
        .nth(index)
        .map(|s| s.to_string())
        .ok_or_else(|| io::Error::new(ErrorKind::InvalidData, "No organization ID at given index"))
}
