use anyhow::{Context, Result};
use clap::Args;
use std::{fs, path::Path, process::Command};

pub const CUSTOM_LIB: &str = include_str!("../../static/lib.rs.template");
pub const TILT_BINDINGS: &str = include_str!("../../static/tilt.rs.template");
pub const CUSTOM_TOML: &str = include_str!("../../static/Cargo.toml.template");
pub const WIT_FILE: &str = include_str!("../../static/tilt_sdk.wit.template");

pub const JS_PACKAGE_JSON: &str = include_str!("../static/js-template/package.json.template");
pub const JS_GITIGNORE: &str = include_str!("../static/js-template/.gitignore.template");
pub const JS_INDEX: &str = include_str!("../static/js-template/src/index.js.template");
pub const JS_TILT_BINDINGS: &str = include_str!("../static/js-template/src/tilt.js.template");
pub const JS_WIT_FILE: &str = include_str!("../static/js-template/wit/tilt_sdk.wit.template");

#[derive(Debug, Args)]
pub struct New {
    #[arg(short = 'n', long)]
    name: String,

    /// Create a JavaScript template project instead of Rust
    #[arg(long, default_value_t = false)]
    js: bool,
}

impl New {
    pub async fn run(&self) -> Result<()> {
        if self.js {
            self.create_js_project()
        } else {
            self.create_rust_project()
        }
    }

    fn create_rust_project(&self) -> Result<()> {
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
                _ => {
                    anyhow::bail!(
                        "Error: something went wrong while creating the project '{name}': {}",
                        String::from_utf8_lossy(&output.stderr)
                    )
                }
            }
        }

        let custom_toml = CUSTOM_TOML.replace("{project_name}", name);

        let status = Command::new("rustup")
            .args(["target", "add", "wasm32-wasip2"])
            .status()
            .context("Failed to run rustup target add wasm32-wasip2")?;

        if !status.success() {
            eprintln!("Failed to add WebAssembly target wasm32-wasip2");
        }

        fs::create_dir_all(format!("{name}/wit")).context("Failed to create wit directory")?;
        fs::write(format!("{name}/src/lib.rs"), CUSTOM_LIB).context("Failed to write lib.rs")?;
        fs::write(format!("{name}/Cargo.toml"), custom_toml)
            .context("Failed to write Cargo.toml")?;
        fs::write(format!("{name}/src/tilt.rs"), TILT_BINDINGS)
            .context("Failed to write tilt.rs")?;
        fs::write(format!("{name}/wit/tilt_sdk.wit"), WIT_FILE)
            .context("Failed to write tilt_sdk.wit")?;

        println!("Project '{name}' created successfully!");
        println!("      cd ./{name}");
        println!("      tilt test");

        Ok(())
    }

    fn create_js_project(&self) -> Result<()> {
        let name = &self.name;
        let root = Path::new(name);

        if root.exists() {
            anyhow::bail!("Error: Project '{name}' already exists! Try something new.");
        }

        fs::create_dir_all(root.join("src")).context("Failed to create src directory")?;
        fs::create_dir_all(root.join("wit")).context("Failed to create wit directory")?;

        let package_json = JS_PACKAGE_JSON.replace("{project_name}", name);

        fs::write(root.join("package.json"), package_json)
            .context("Failed to write package.json")?;
        fs::write(root.join(".gitignore"), JS_GITIGNORE).context("Failed to write .gitignore")?;
        fs::write(root.join("src/index.js"), JS_INDEX).context("Failed to write src/index.js")?;
        fs::write(root.join("src/tilt.js"), JS_TILT_BINDINGS)
            .context("Failed to write src/tilt.js")?;
        fs::write(root.join("wit/tilt_sdk.wit"), JS_WIT_FILE)
            .context("Failed to write wit/tilt_sdk.wit")?;

        println!("JavaScript project '{name}' created successfully!");
        println!("      cd ./{name}");
        println!("      pnpm install");
        println!("      pnpm run build");
        println!("      tilt deploy");

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_new_creates_rust_project_files() {
        let temp_dir = tempdir().unwrap();
        let name = temp_dir.path().join("my_project");
        let name_str = name.to_str().unwrap().to_string();

        let cmd = New {
            name: name_str.clone(),
            js: false,
        };

        cmd.run().await.unwrap();

        assert!(name.join("src/lib.rs").exists());
        assert!(name.join("Cargo.toml").exists());
        assert!(name.join("src/tilt.rs").exists());
        assert!(name.join("wit/tilt_sdk.wit").exists());
    }

    #[tokio::test]
    async fn test_new_rust_toml_has_project_name() {
        let temp_dir = tempdir().unwrap();
        let name = temp_dir.path().join("my_project");
        let name_str = name.to_str().unwrap().to_string();

        let cmd = New {
            name: name_str.clone(),
            js: false,
        };

        cmd.run().await.unwrap();

        let toml = fs::read_to_string(name.join("Cargo.toml")).unwrap();
        assert!(toml.contains(&name_str));
    }

    #[tokio::test]
    async fn test_new_creates_js_project_files() {
        let temp_dir = tempdir().unwrap();
        let name = temp_dir.path().join("my_js_project");
        let name_str = name.to_str().unwrap().to_string();

        let cmd = New {
            name: name_str.clone(),
            js: true,
        };

        cmd.run().await.unwrap();

        assert!(name.join("package.json").exists());
        assert!(name.join(".gitignore").exists());
        assert!(name.join("src/index.js").exists());
        assert!(name.join("src/tilt.js").exists());
        assert!(name.join("wit/tilt_sdk.wit").exists());
    }
}
