mod auth;
mod entities;
mod helpers;
mod organization;
mod task;

use anyhow::Result;
use clap::{Parser, Subcommand};
use custom_lib::{CUSTOM_LIB, CUSTOM_TOML};
use reqwest::Client;
use reqwest::StatusCode;
use reqwest::multipart;
use std::{fs, path::Path, process::Command};

mod custom_lib;

use auth::sign_in;
use helpers::get_package_metadata;
use helpers::release_path;
use helpers::url_from_env;

use crate::auth::load_auth_token;
use crate::custom_lib::TILT_BINDINGS;
use crate::custom_lib::WIT_FILE;
use crate::entities::program::Program;
use crate::entities::response::Response;
use crate::organization::load_selected_organization_id;

#[derive(Debug, Parser)]
#[command(name = "tilt", about = "Command Line Application for Tilt network")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    New { name: String },
    Build,
    Test,
    Clean,
    List,
    Deploy,
    Signin {
        #[arg(long = "secret_key", short = 'k')]
        secret_key: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::New { name }) => create_new_project(&name),
        Some(Commands::Test) => test_project(),
        Some(Commands::Clean) => clean_project(),
        Some(Commands::Build) => build_project(),
        Some(Commands::List) => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            if let Err(err) = rt.block_on(list_programs()) {
                eprintln!("Error during listing: {err}");
            }
        }
        Some(Commands::Deploy) => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            if let Err(e) = rt.block_on(deploy()) {
                eprintln!("Error during deployment: {e}");
            }
        }
        Some(Commands::Signin { secret_key }) => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            if let Err(err) = rt.block_on(sign_in(&secret_key)) {
                eprintln!("Error during sign in: {err}");
            }
        }
        None => {
            use clap::CommandFactory;
            Cli::command().print_help().unwrap();
        }
    }
}

fn create_new_project(project_name: &String) {
    let output = Command::new("cargo")
        .args(["new", "--lib", project_name])
        .output()
        .expect("Failed to execute cargo new");

    if !output.status.success() {
        if let Some(101) = output.status.code() {
            eprintln!("Error: Project '{}' already exists!", project_name);
        } else {
            eprintln!(
                "Error creating project '{}': {}",
                project_name,
                String::from_utf8_lossy(&output.stderr)
            );
        }
        return;
    }

    let lib_path = format!("{}/src/lib.rs", project_name);
    let tilt_bindings_path = format!("{}/src/tilt.rs", project_name);
    let toml_path = format!("{}/Cargo.toml", project_name);
    let wit_path = format!("{}/wit/tilt_sdk.wit", project_name);
    let custom_lib = CUSTOM_LIB;
    let tilt_bindings = TILT_BINDINGS;
    let custom_toml = CUSTOM_TOML.replace("{project_name}", project_name);
    let wit_file = WIT_FILE;

    let status = Command::new("rustup")
        .args(["target", "add", "wasm32-wasip2"])
        .status()
        .expect("Failed to add wasm32-wasip2 target");

    if !status.success() {
        eprintln!("Failed to add WebAssembly target");
    }

    let wit_dir = format!("{}/wit", project_name);
    fs::create_dir_all(&wit_dir).expect("Failed to create wit directory");

    fs::write(&lib_path, custom_lib).expect("Failed to write lib.rs");
    fs::write(&toml_path, custom_toml).expect("Failed to write Cargo.toml");
    fs::write(&tilt_bindings_path, tilt_bindings).expect("Failed to write tilt.rs");
    fs::write(&wit_path, wit_file).expect("Failed to write tilt.rs");

    println!("Project '{}' created successfully!", project_name);
    println!("    cd ./{}", project_name);
    println!("    tilt-cli test");
}

fn test_project() {
    let mut child = Command::new("cargo")
        .arg("test")
        .spawn()
        .expect("Failed to execute test");

    let status = child.wait().expect("Failed to wait on test process");

    if !status.success() {
        eprintln!("Tests failed");
    }
}

fn clean_project() {
    let mut child = Command::new("cargo")
        .arg("clean")
        .spawn()
        .expect("Failed to execute test");

    let status = child.wait().expect("Failed to wait on test process");

    if !status.success() {
        eprintln!("Tests failed");
    }
}

fn build_project() {
    let mut child = Command::new("cargo")
        .args(["build", "--target", "wasm32-wasip2", "--release"])
        .spawn()
        .expect("Failed to execute build");

    let status = child.wait().expect("Failed to build project");

    if !status.success() {
        eprintln!("Build failed");
    }
}

fn print_program_table(data: Vec<Program>) {
    let name_width = 20;
    println!("{:<name_width$} | Description", "Name", name_width = name_width);
    println!(
        "{:-<name_width$}-+-{:-<desc_width$}",
        "",
        "",
        name_width = name_width,
        desc_width = 50
    );
    for item in data {
        let mut name = item.name.clone().unwrap_or_else(|| "Unnamed".into());
        if name.chars().count() > name_width {
            name = name.chars().take(name_width - 3).collect::<String>() + "...";
        }
        let desc = item.description.clone().unwrap_or_else(|| "-".into());
        println!("{:<name_width$} | {}", name, desc, name_width = name_width);
    }
}

async fn list_programs() -> Result<()> {
    let base_url = url_from_env();
    let url = format!("{}/programs", base_url);
    let client = reqwest::Client::new();
    let token = load_auth_token().unwrap();
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

    print_program_table(data);

    Ok(())
}

async fn deploy() -> Result<()> {
    build_project();
    let client = Client::new();
    let base_url = url_from_env();
    let url = format!("{}/programs", base_url);

    let program = "program";

    let filename = release_path()?;
    let file_path = Path::new(&filename);
    let file_bytes = std::fs::read(file_path)?;

    let part = multipart::Part::bytes(file_bytes)
        .file_name(program)
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
        .post(url)
        .bearer_auth(&token)
        .multipart(form)
        .send()
        .await?;

    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    if status == StatusCode::OK {
        println!("Program deployed successfuly");
    } else {
        println!("Failed to upload program (status: {status})");
        println!("Response body: {body}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_create_new_project() {
        let temp_dir = tempdir().unwrap();
        let project_name = temp_dir.path().join("test_project");
        let project_name_str = project_name.to_str().unwrap().to_string();

        create_new_project(&project_name_str);

        assert!(
            project_name.join("src/lib.rs").exists(),
            "lib.rs should exist"
        );
        assert!(
            project_name.join("Cargo.toml").exists(),
            "Cargo.toml should exist"
        );
    }

    #[test]
    fn test_test_project() {
        let temp_dir = tempdir().unwrap();
        let project_name = temp_dir.path().join("test_project");
        let project_name_str = project_name.to_str().unwrap().to_string();

        create_new_project(&project_name_str);

        assert!(
            project_name.join("src/lib.rs").exists(),
            "lib.rs should exist"
        );
        assert!(
            project_name.join("Cargo.toml").exists(),
            "Cargo.toml should exist"
        );
    }

    #[test]
    fn test_build_project() {
        build_project();

        let wasm_dir = Path::new("./target/wasm32-wasip2/release");

        assert!(
            fs::read_dir(wasm_dir)
                .expect("Failed to read target directory")
                .filter_map(Result::ok)
                .any(|entry| entry.path().extension().is_some_and(|ext| ext == "wasm")),
            "Expected a .wasm file in the release dir"
        );
    }

    #[tokio::test]
    async fn test_deploy() {
        let client = Client::new();
        let temp_dir = tempdir().unwrap();
        let wasm_file = temp_dir.path().join("test.wasm");

        fs::write(&wasm_file, b"\x00asm\x01\x00\x00\x00").expect("Failed to write test wasm file");

        let part = multipart::Part::bytes(fs::read(&wasm_file).unwrap())
            .file_name("test.wasm")
            .mime_str("application/wasm")
            .unwrap();

        let form = multipart::Form::new().part("file", part);

        let base_url = url_from_env();
        let response = client
            .post(format!("{base_url}/programs"))
            .multipart(form)
            .send()
            .await;

        assert!(
            response.is_ok(),
            "Deploy request should be sent successfully"
        );
    }
}
