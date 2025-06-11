use serde::Serialize;
use reqwest::Client;

use crate::{helpers::{get_project_name, url_from_env}, organization::load_organization_id};

#[derive(Serialize)]
struct JobRequest {
    organization_id: String,
    name: String,
    status: String,
    total_tokens: u32,
}

pub async fn create_job() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = format!("{}/jobs", url_from_env());

    let name = get_project_name();
    let organization_id = load_organization_id(0)?;

    let payload = JobRequest {
        organization_id,
        name,
        status: "pending".to_string(),
        total_tokens: 0,
    };

    let response = client.post(&url).json(&payload).send().await?;

    if response.status().is_success() {
        println!("Job created successfully");
        println!("Response: {:?}", response.text().await?);
    } else {
        println!("Failed to create job: {}", response.status());
        println!("Response: {:?}", response.text().await?);
    }

    Ok(())
}

