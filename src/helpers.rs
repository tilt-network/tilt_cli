use std::{env, fs};

use anyhow::{Result, anyhow};
use toml::Value;

pub fn url_from_env() -> &'static str {
    let prod_url = "https://production.tilt.rest";
    let stg_url = "https://staging.tilt.rest";
    match env::var("USE_TILT_STAGING") {
        Ok(val) => {
            let val = val.to_ascii_lowercase();
            if val == "true" || val == "1" {
                return stg_url;
            }
        }
        Err(env::VarError::NotPresent) => return prod_url,
        Err(_) => return prod_url,
    }
    prod_url
}

pub fn release_path() -> Result<String, anyhow::Error> {
    let md = get_package_metadata()?;
    let package_name = md.0;
    Ok(format!(
        "./target/wasm32-wasip2/release/{}.wasm",
        package_name.replace("-", "_")
    ))
}

pub fn _maybe_replace_program_id(custom_toml: &str, program_id: &str) -> String {
    if custom_toml.contains("{program_id}") {
        custom_toml.replace("{program_id}", program_id)
    } else {
        custom_toml.to_string()
    }
}

pub fn get_package_metadata() -> Result<(String, String), anyhow::Error> {
    let cargo_toml_path = env::current_dir()
        .expect("error getting current directory")
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
