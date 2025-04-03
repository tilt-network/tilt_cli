use std::{env, fs, process::Command};
use clap::{Arg, Command as ClapCommand};

fn main() {
    let matches = ClapCommand::new("tilt")
        .about("Command Line Application for Tilt network")
        .subcommand(
            ClapCommand::new("new")
                .about("Creates a new Tilt project")
                .arg(Arg::new("name").required(true)),
        )
        .get_matches();

    if let Some(sub_matches) = matches.subcommand_matches("new") {
        let project_name = sub_matches.get_one::<String>("name").unwrap();

        // Run `cargo new <project_name>`
        let status = Command::new("cargo")
            .arg("new")
            .arg(project_name)
            .status()
            .expect("Failed to execute cargo new");

        if !status.success() {
            eprintln!("Failed to create project {}", project_name);
            return;
        }

        // Modify main.rs
        let main_path = format!("{}/src/main.rs", project_name);
        let custom_main = r#"fn main() {
    println!("Hello from tilt");
}"#;
        
        fs::write(&main_path, custom_main).expect("Failed to write main.rs");

        println!("Project '{}' created successfully!", project_name);
    }
}

