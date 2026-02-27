use std::{env, fs, path::PathBuf};

use anyhow::{Context, Result, anyhow};
use toml::Value;

pub fn url_from_env() -> &'static str {
    match env::var("USE_TILT_STAGING").as_deref() {
        Ok("true") | Ok("1") => "https://staging.tilt.rest",
        _ => "https://production.tilt.rest",
    }
}

pub fn release_path() -> Result<String> {
    let (name, _) = get_package_metadata()?;
    Ok(format!(
        "./target/wasm32-wasip2/release/{}.wasm",
        name.replace("-", "_")
    ))
}

pub fn get_package_metadata() -> Result<(String, String)> {
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

pub fn tilt_dir() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|p| p.join(".tilt"))
        .ok_or_else(|| anyhow!("Home directory not found"))
}
