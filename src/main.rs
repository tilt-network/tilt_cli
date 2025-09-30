mod auth;
mod entities;
mod helpers;
mod job;
mod organization;
mod task;

use clap::{Arg, Command as ClapCommand};
use custom_lib::{CUSTOM_LIB, CUSTOM_TOML};
use reqwest::Client;
use reqwest::Error;
use reqwest::StatusCode;
use reqwest::multipart;
use std::env;
use std::{fs, path::Path, process::Command};
// use toml::Value;
// use uuid::Uuid;
mod custom_lib;
use auth::sign_in;
use helpers::get_package_metadata;
// use helpers::get_project_name;
// use helpers::maybe_replace_program_id;
use helpers::release_path;
use helpers::url_from_env;
use organization::load_organization_id;

use crate::auth::load_auth_token;
use crate::custom_lib::TILT_BINDINGS;
use crate::custom_lib::WIT_FILE;
use crate::entities::program::Program;
use crate::entities::response::Response;
use crate::job::create_job;
use crate::organization::load_selected_organization_id;

fn main() {
    let mut cmd = ClapCommand::new("tilt")
        .about("Command Line Application for Tilt network")
        .subcommand(
            ClapCommand::new("new")
                .about("Creates a new Tilt project")
                .arg(Arg::new("name").required(true)),
        )
        .subcommand(ClapCommand::new("build").about("Build the Tilt project"))
        .subcommand(ClapCommand::new("test").about("Test the Tilt project"))
        .subcommand(ClapCommand::new("clean").about("Clean the Tilt project"))
        .subcommand(ClapCommand::new("list").about("List Tilt programs"))
        .subcommand(ClapCommand::new("deploy").about("Deploy the Tilt project"))
        .subcommand(ClapCommand::new("create-job").about("Create a job for the current project"))
        .subcommand(ClapCommand::new("organization").about("Select a Tilt organization"))
        .subcommand(
            ClapCommand::new("signin").about("Sign in to Tilt").arg(
                Arg::new("secret_key")
                    .long("secret_key")
                    .short('k')
                    .required(true),
            ),
        );

    let matches = cmd.clone().get_matches();

    match matches.subcommand() {
        Some(("new", sub_matches)) => {
            let project_name = match sub_matches.get_one::<String>("name") {
                Some(pn) => pn,
                None => {
                    eprintln!("Project name is required");
                    return;
                }
            };
            create_new_project(project_name);
        }
        Some(("test", _)) => {
            test_project();
        }
        Some(("clean", _)) => {
            clean_project();
        }
        Some(("build", _)) => {
            build_project();
        }
        Some(("list", _)) => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            if let Err(err) = rt.block_on(list_programs()) {
                println!("Error during listing: {}", err)
            }
        }
        Some(("deploy", _)) => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let res = rt.block_on(deploy());
            match res {
                Ok(_) => {}
                Err(e) => eprintln!("Error during deployment: {}", e),
            }
        }
        Some(("signin", sub_matches)) => {
            let secret_key = sub_matches.get_one::<String>("secret_key").unwrap();
            let rt = tokio::runtime::Runtime::new().unwrap();
            if let Err(err) = rt.block_on(sign_in(secret_key)) {
                println!("Error during sign in: {}", err);
            }
        }
        Some(("create-job", _)) => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(create_job()).unwrap();
        }
        _ => cmd.print_help().unwrap(),
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

    // Add WebAssembly target
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
    // let program_id = match check_program_id() {
    //     Some(id) => id,
    //     None => Uuid::new_v4().to_string(),
    // };
    let toml_path = env::current_dir()
        .expect("failed to get current dir")
        .join("Cargo.toml");
    let _toml_content = fs::read_to_string(&toml_path).unwrap();
    // let replaced_toml = maybe_replace_program_id(&toml_content, &program_id);
    // fs::write(toml_path, replaced_toml).unwrap();
    // let package_name = get_project_name();
    // let from_path = release_path(&package_name);
    // let to_path = release_path(&program_id);

    // if Path::new(&from_path).exists() {
    //     fs::rename(from_path, to_path).expect("Failed to rename .wasm file");
    // } else {
    //     eprintln!("Expected .wasm file not found, skipping rename");
    // }

    if !status.success() {
        eprintln!("Build failed");
    }
}

fn print_program_table(data: Vec<Program>) {
    // largura máxima da coluna Name
    let name_width = 20;
    // cabeçalho
    println!(
        "{:<name_width$} | {}",
        "Name",
        "Description",
        name_width = name_width
    );
    println!(
        "{:-<name_width$}-+-{:-<desc_width$}",
        "",
        "",
        name_width = name_width,
        desc_width = 50
    );
    // linhas
    for item in data {
        // pega ou default
        let mut name = item.name.clone().unwrap_or_else(|| "Unnamed".into());
        // trunca se ultrapassar
        if name.chars().count() > name_width {
            name = name.chars().take(name_width - 3).collect::<String>() + "...";
        }
        let desc = item.description.clone().unwrap_or_else(|| "-".into());
        // imprime: Name (padded à esquerda) | Description
        println!("{:<name_width$} | {}", name, desc, name_width = name_width);
    }
}

async fn list_programs() -> Result<(), Error> {
    // let global_url = String::from(url_from_env());
    let base_url = url_from_env();
    let url = format!("{}/programs", base_url);
    let client = reqwest::Client::new();
    let token = load_auth_token().unwrap();
    let organization_id = load_organization_id(0).unwrap();

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

    // if let Some(data) = response.get("data").and_then(|v| v.as_array()) {
    //     if data.is_empty() {
    //         println!("No programs found.");
    //     } else {
    //         for item in data {
    //             let name = item
    //                 .get("name")
    //                 .and_then(|v| v.as_str())
    //                 .unwrap_or("Unnamed");
    //             let description = item
    //                 .get("description")
    //                 .and_then(|v| v.as_str())
    //                 .unwrap_or("-");
    //             println!("{} — {}", name, description);
    //         }
    //     }
    // } else {
    //     println!("Unexpected response format.");
    // }

    Ok(())
}

async fn deploy() -> Result<(), Box<dyn std::error::Error>> {
    build_project();
    let client = Client::new();
    let base_url = url_from_env();
    let url = format!("{}/programs", base_url);
    // let program_id = match check_program_id() {
    //     Some(id) => id,
    //     None => panic!("Build program before deploying"),
    // };

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

    if response.status() == StatusCode::OK {
        println!("Program deployed successfuly");
        // println!("Response: {:?}", response.text().await?);
    } else {
        println!("Failed to upload program");
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

        let wasm_file = fs::read_dir(wasm_dir)
            .expect("Failed to read target directory")
            .filter_map(Result::ok)
            .find(|entry| entry.path().extension().map_or(false, |ext| ext == "wasm"));

        assert!(
            wasm_file.is_some(),
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
