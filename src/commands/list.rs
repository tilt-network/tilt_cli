use crate::utils;
use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::Args;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

#[derive(Debug, Args)]
pub struct List {}

impl List {
    pub async fn run(&self) -> Result<()> {
        let base_url = utils::url_from_env();
        let url = format!("{base_url}/programs");
        let client = Client::builder().timeout(Duration::from_secs(5)).build()?;
        let token = utils::load_auth_token()?;
        let organization_id = utils::load_selected_organization_id()?;

        let programs = client
            .get(&url)
            .query(&[
                ("page", "1"),
                ("page_size", "100"),
                ("organization_id", &organization_id),
            ])
            .bearer_auth(&token)
            .send()
            .await?
            .json::<ProgramList>()
            .await?;

        let Some(data) = programs.data else {
            println!("No programs found.");
            return Ok(());
        };

        print_table(&data);
        Ok(())
    }
}

/// The response from the Tilt API when listing programs.
#[derive(Default, PartialEq, Debug, Clone, Serialize, Deserialize)]
struct ProgramList {
    data: Option<Vec<Program>>,
    message: Option<String>,
    page: Option<u32>,
    page_size: Option<u32>,
    total_items: Option<u64>,
    total_pages: Option<u32>,
}

/// A Program as defined by the Tilt API.
#[derive(Default, PartialEq, Debug, Clone, Serialize, Deserialize)]
struct Program {
    id: Option<Uuid>,
    name: Option<String>,
    description: Option<String>,
    path: Option<String>,
    size: Option<i32>,
    organization_id: Option<Uuid>,
    updated_at: Option<DateTime<Utc>>,
    created_at: Option<DateTime<Utc>>,
}

fn print_table(data: &[Program]) {
    let max_name_width = 20;
    let desc_width = 50;
    println!("{:<max_name_width$} | Description", "Name");
    println!("{:-<max_name_width$}-+-{:-<desc_width$}", "", "");
    for item in data {
        let name = item.name.as_deref().unwrap_or("Unnamed");
        let name = &name[..name.len().min(max_name_width)];
        let desc = item.description.as_deref().unwrap_or("-");
        println!("{name:<max_name_width$} | {desc}");
    }
}
