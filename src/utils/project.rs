use std::{env, fs};

use anyhow::{Context, Result, anyhow};
use toml::Value;

pub enum ProjectKind {
    Rust,
    Go,
}

pub fn detect_project_kind() -> Result<ProjectKind> {
    let dir = env::current_dir()?;
    match (dir.join("Cargo.toml").exists(), dir.join("go.mod").exists()) {
        (true, _) => Ok(ProjectKind::Rust),
        (false, true) => Ok(ProjectKind::Go),
        (false, false) => Err(anyhow!("No supported project kind found")),
    }
}

pub fn rust_package_metadata() -> Result<(String, String)> {
    let cargo_toml_path = env::current_dir()
        .context("error getting current directory")?
        .join("Cargo.toml");
    let cargo_toml_content = fs::read_to_string(cargo_toml_path)?;
    let parsed: Value = cargo_toml_content.parse::<Value>()?;

    let package = parsed
        .get("package")
        .and_then(|v| v.as_table())
        .ok_or_else(|| anyhow!("Missing [package] section"))?;

    let name = package
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'name' in [package]"))?
        .to_string();

    let description = package
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Ok((name, description))
}

pub fn go_package_metadata() -> Result<(String, String)> {
    let go_mod = fs::read_to_string("go.mod")?;
    let module_path = go_mod
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("module ")
                .map(str::trim)
                .map(str::to_string)
        })
        .ok_or_else(|| anyhow!("Module directive not found in go.mod"))?;

    let name = module_path
        .split('/')
        .next_back()
        .unwrap_or(&module_path)
        .to_string();

    Ok((name, module_path))
}
