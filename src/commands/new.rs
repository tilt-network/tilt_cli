use anyhow::{Context, Result};
use clap::Args;
use std::{fs, process::Command};

use crate::{
    commands::Run,
    custom_lib::{CUSTOM_LIB, CUSTOM_TOML, TILT_BINDINGS, WIT_FILE},
};

/// Create a new tilt program
#[derive(Debug, Args)]
pub struct New {
    pub name: String,
}

impl Run for New {
    async fn run(&self) -> Result<()> {
        let name = &self.name;

        let output = Command::new("cargo")
            .args(["new", "--lib", name])
            .output()
            .context("Failed to create a new program. Do you have rust installed?")?;

        if !output.status.success() {
            match output.status.code() {
                Some(101) => {
                    anyhow::bail!("Error: Project '{name}' already exists! Try something new.")
                }
                _ => eprintln!(
                    "Error: something went wrong while creating the project '{name}': {}. ",
                    String::from_utf8_lossy(&output.stderr)
                ),
            }
        }

        let custom_toml = CUSTOM_TOML.replace("{project_name}", name);

        let status = Command::new("rustup")
            .args(["target", "add", "wasm32-wasip2"])
            .status()
            .expect("Failed to add wasm32-wasip2 target");

        if !status.success() {
            eprintln!("Failed to add WebAssembly target");
        }

        fs::create_dir_all(format!("{name}/wit")).expect("Failed to create directory");
        fs::write(format!("{name}/src/lib.rs"), CUSTOM_LIB).expect("Failed to write lib.rs");
        fs::write(format!("{name}/Cargo.toml"), custom_toml).expect("Failed to write Cargo.toml");
        fs::write(format!("{name}/src/tilt.rs"), TILT_BINDINGS).expect("Failed to write tilt.rs");
        fs::write(format!("{name}/wit/tilt_sdk.wit"), WIT_FILE)
            .expect("Failed to write tilt_sdk.wit");

        println!("Project '{name}' created successfully!");
        println!("      cd ./{name}");
        println!("      tilt test");

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_new_creates_project_files() {
        let temp_dir = tempdir().unwrap();
        let name = temp_dir.path().join("my_project");
        let name_str = name.to_str().unwrap().to_string();

        let cmd = New { name: name_str };
        cmd.run().await.unwrap();

        assert!(name.join("src/lib.rs").exists());
    }

    #[tokio::test]
    async fn test_new_toml_has_project_name() {
        let temp_dir = tempdir().unwrap();
        let name = temp_dir.path().join("my_project");
        let name_str = name.to_str().unwrap().to_string();

        let cmd = New {
            name: name_str.clone(),
        };
        cmd.run().await.unwrap();

        let toml = fs::read_to_string(name.join("Cargo.toml")).unwrap();
        assert!(toml.contains(&name_str));
    }
}
