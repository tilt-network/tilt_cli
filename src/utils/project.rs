use std::{env, fs};

use anyhow::{Result, anyhow};

pub enum ProjectKind {
    Rust,
    Go,
}

pub fn detect_project_kind() -> Result<ProjectKind> {
    let dir = env::current_dir()?;
    if dir.join("Cargo.toml").exists() {
        Ok(ProjectKind::Rust)
    } else if dir.join("go.mod").exists() {
        Ok(ProjectKind::Go)
    } else {
        Err(anyhow!("No supported project kind found"))
    }
}

pub fn go_package_metadata() -> Result<(String, String)> {
    let go_mod = fs::read_to_string("go.mod")?;
    let module_path = go_mod
        .lines()
        .find_map(|line| line.trim().strip_prefix("module ").map(str::trim).map(str::to_string))
        .ok_or_else(|| anyhow!("Module directive not found in go.mod"))?;

    let name = module_path.split('/').next_back().unwrap_or(&module_path).to_string();

    Ok((name, module_path))
}
