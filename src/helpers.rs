use std::{env, fs, process::Command};

use toml::Value;

pub fn get_project_name() -> String {
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

pub fn url_from_env() -> &'static str {
    let prod_url = "https://production.tilt.rest/";
    let stg_url = "https://production.tilt.rest/";
    match env::var("USE_TILT_STAGING") {
        Ok(val) => {
            let val = val.to_ascii_lowercase();
            if val == "true" || val == "1" {
                return stg_url;
            }
        }
        Err(env::VarError::NotPresent) => return prod_url,
        Err(_) => return prod_url, // Covers VarError::NotUnicode
    }
    prod_url
}

pub fn release_path(filename: &str) -> String {
    format!("./target/wasm32-unknown-unknown/release/{}.wasm", filename)
}

pub fn check_program_id() -> Option<String> {
    let cwd = env::current_dir().ok()?;
    let toml_path = cwd.join("Cargo.toml");
    let toml_content = fs::read_to_string(&toml_path).ok()?;
    let parsed: Value = toml_content.parse().ok()?;

    let program_id = parsed
        .get("package")?
        .get("metadata")?
        .get("tilt")?
        .get("program_id")?
        .as_str()?;

    if program_id.trim() == "{program_id}" {
        None
    } else {
        Some(program_id.to_string())
    }
}

pub fn maybe_replace_program_id(custom_toml: &str, program_id: &str) -> String {
    if custom_toml.contains("{program_id}") {
        custom_toml.replace("{program_id}", program_id)
    } else {
        custom_toml.to_string()
    }
}

pub fn get_package_metadata() -> Result<(String, String), Box<dyn std::error::Error>> {
    let cargo_toml_path = env::current_dir()
        .expect("error getting current directory")
        .join("Cargo.toml");
    let cargo_toml_content = fs::read_to_string(cargo_toml_path)?;
    let parsed: Value = cargo_toml_content.parse::<Value>()?;

    let package = parsed
        .get("package")
        .and_then(|v| v.as_table())
        .ok_or("Missing [package] section")?;

    let name = package
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or("Missing 'name' in [package]")?
        .to_string();

    let description = package
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Ok((name, description))
}
