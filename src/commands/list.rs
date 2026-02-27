use anyhow::Result;
use clap::Args;
use reqwest::Client;

use crate::{
    auth::load_auth_token,
    commands::Run,
    entities::{program::Program, response::Response},
    helpers::url_from_env,
    organization::load_selected_organization_id,
};

/// List your tilt programs
#[derive(Debug, Args)]
pub struct List;

impl Run for List {
    async fn run(&self) -> Result<()> {
        let base_url = url_from_env();
        let url = format!("{base_url}/programs");
        let client = Client::new();
        let token = load_auth_token()?;
        let organization_id = load_selected_organization_id()?;

        let response = client
            .get(&url)
            .query(&[
                ("page", "1"),
                ("page_size", "100"),
                ("organization_id", &organization_id),
            ])
            .bearer_auth(&token)
            .send()
            .await?;

        let response = response.json::<Response<Vec<Program>>>().await?;

        let Some(data) = response.data else {
            println!("No programs found.");
            return Ok(());
        };

        print_table(data);
        Ok(())
    }
}

fn print_table(data: Vec<Program>) {
    let name_width = 20;
    println!("{:<name_width$} | Description", "Name");
    println!("{:-<name_width$}-+-{:-<50}", "", "");
    for item in data {
        let mut name = item.name.unwrap_or_else(|| "Unnamed".into());
        if name.chars().count() > name_width {
            name = name.chars().take(name_width - 3).collect::<String>() + "...";
        }
        let desc = item.description.unwrap_or_else(|| "-".into());
        println!("{name:<name_width$} | {desc}");
    }
}
