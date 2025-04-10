use clap::{Arg, Command as ClapCommand};
use custom_lib::{CUSTOM_LIB, CUSTOM_TOML};
use reqwest::Client;
use reqwest::StatusCode;
use reqwest::multipart;
use std::env;
use std::{fs, path::Path, process::Command};
use toml::Value;
use uuid::Uuid;
mod custom_lib;

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
        .subcommand(ClapCommand::new("deploy").about("Deploy the Tilt project"));
    let matches = cmd.clone().get_matches();

    match matches.subcommand() {
        Some(("new", sub_matches)) => {
            let project_name = sub_matches.get_one::<String>("name").unwrap();
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
        Some(("deploy", _)) => {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(deploy()).unwrap();
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
    let toml_path = format!("{}/Cargo.toml", project_name);
    let custom_lib = CUSTOM_LIB;
    let custom_toml = CUSTOM_TOML.replace("{project_name}", project_name);

    // Add WebAssembly target
    let status = Command::new("rustup")
        .args(["target", "add", "wasm32-unknown-unknown"])
        .status()
        .expect("Failed to add wasm32-unknown-unknown target");

    if !status.success() {
        eprintln!("Failed to add WebAssembly target");
    }

    fs::write(&lib_path, custom_lib).expect("Failed to write lib.rs");
    fs::write(&toml_path, custom_toml).expect("Failed to write Cargo.toml");

    println!("Project '{}' created successfully!", project_name);
    println!("    cd ./{}", project_name);
    println!("    tilt test");
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

fn get_project_name() -> String {
    let output = Command::new("sh")
        .arg("-c")
        .arg("cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].name'")
        .output()
        .expect("Failed to execute shell command");

    String::from_utf8(output.stdout)
        .expect("Invalid UTF-8 in output")
        .trim()
        .to_string()
}

fn build_project() {
    let mut child = Command::new("cargo")
        .args(["build", "--target", "wasm32-unknown-unknown", "--release"])
        .spawn()
        .expect("Failed to execute build");

    let status = child.wait().expect("Failed to build project");
    let program_id = match check_program_id() {
        Some(id) => id,
        None => Uuid::new_v4().to_string(),
    };
    let toml_path = env::current_dir()
        .expect("failed to get current dir")
        .join("Cargo.toml");
    let toml_content = fs::read_to_string(&toml_path).unwrap();
    let replaced_toml = maybe_replace_program_id(&toml_content, &program_id);
    fs::write(toml_path, replaced_toml).unwrap();
    let package_name = get_project_name();
    let from_path = release_path(&package_name);
    let to_path = release_path(&program_id);

    if Path::new(&from_path).exists() {
        fs::rename(from_path, to_path).expect("Failed to rename .wasm file");
    } else {
        eprintln!("Expected .wasm file not found, skipping rename");
    }

    if !status.success() {
        eprintln!("Build failed");
    }
}

async fn deploy() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let url = "http://localhost:3000/upload_program"; // Replace the actual endpoint
    let package_name = get_project_name();
    let filename = release_path(&package_name);
    let file_path = Path::new(&filename);
    let file_bytes = std::fs::read(file_path)?;

    let part = multipart::Part::bytes(file_bytes)
        .file_name(filename)
        .mime_str("application/wasm")?;

    let form = multipart::Form::new().part("file", part);

    let response = client.post(url).multipart(form).send().await?;

    if response.status() == StatusCode::OK {
        println!("Program uploaded successfuly");
        println!("Response: {:?}", response.text().await?);
    } else {
        println!("Failed to upload program");
    }

    Ok(())
}

fn release_path(filename: &str) -> String {
    format!("./target/wasm32-unknown-unknown/release/{}.wasm", filename)
}

fn check_program_id() -> Option<String> {
    let cwd = env::current_dir().ok()?;
    let toml_path = cwd.join("Cargo.toml");
    let toml_content = fs::read_to_string(&toml_path).ok()?;
    let parsed: Value = toml_content.parse().ok()?;

    let program_id = parsed
        .get("package")?
        .get("metadata")?
        .get("mytool")?
        .get("program_id")?
        .as_str()?;

    if program_id.trim() == "{program_id}" {
        None
    } else {
        Some(program_id.to_string())
    }
}

fn maybe_replace_program_id(custom_toml: &str, program_id: &str) -> String {
    if custom_toml.contains("{program_id}") {
        custom_toml.replace("{program_id}", program_id)
    } else {
        custom_toml.to_string()
    }
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

        let wasm_dir = Path::new("./target/wasm32-unknown-unknown/release");

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

        let response = client
            .post("http://localhost:3000/upload_program")
            .multipart(form)
            .send()
            .await;

        assert!(
            response.is_ok(),
            "Deploy request should be sent successfully"
        );
    }
}
