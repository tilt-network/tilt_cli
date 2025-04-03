use clap::{Arg, Command as ClapCommand};
use custom_lib::{CUSTOM_LIB, CUSTOM_TOML};
use std::{fs, process::Command};
mod custom_lib;

fn main() {
    let matches = ClapCommand::new("tilt")
        .about("Command Line Application for Tilt network")
        .subcommand(
            ClapCommand::new("new")
                .about("Creates a new Tilt project")
                .arg(Arg::new("name").required(true)),
        )
        .subcommand(ClapCommand::new("build").about("Build the Tilt project"))
        .get_matches();

    match matches.subcommand() {
        Some(("new", sub_matches)) => {
            let project_name = sub_matches.get_one::<String>("name").unwrap();
            create_new_project(project_name);
        }
        Some(("build", _)) => {
            build_project();
        }
        _ => unreachable!(), // Clap ensures a valid subcommand is provided
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
    let custom_toml = CUSTOM_TOML;

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
}

fn build_project() {
    let output = Command::new("cargo")
        .args(["build", "--target", "wasm32-unknown-unknown", "--release"])
        .output()
        .expect("Failed to execute build");

    if !output.status.success() {
        eprintln!("Error building project: {:?}", output);
        return;
    }
}
