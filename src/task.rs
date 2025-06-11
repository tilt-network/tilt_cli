use reqwest::Client;
use reqwest::StatusCode;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

use crate::helpers::url_from_env;

pub async fn create_task(job_id: &str, segment_index: u32) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = format!("{}/tasks", url_from_env());

    let body = json!({
        "job_id": job_id,
        "segment_index": segment_index,
        "status": "pending"
    });

    let response = client.post(&url).json(&body).send().await?;

    if response.status() == StatusCode::OK || response.status() == StatusCode::CREATED {
        println!("Task created successfully");
    } else {
        let status = response.status();
        let text = response.text().await?;
        eprintln!("Failed to create task ({}): {}", status, text);
    }

    Ok(())
}

pub async fn run_tasks_for_dir(dir_path: &str, job_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let files: Vec<PathBuf> = fs::read_dir(dir_path)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect();

    for (index, _) in files.iter().enumerate() {
        create_task(job_id, index as u32).await?;
    }

    Ok(())
}

