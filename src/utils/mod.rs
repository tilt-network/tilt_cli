mod auth;
mod organization;

use anyhow::{Result, anyhow};
use std::{env, path::PathBuf};

pub use auth::*;
pub use organization::*;

/// Returns the base URL to be used for API requests.
pub fn url_from_env() -> &'static str {
    match env::var("ENVIRONMENT").as_deref() {
        Ok("staging") => "https://staging.tilt.rest",
        Ok("production") => "https://production.tilt.rest",
        Ok("local") => "http://localhost:3000",
        _ => "https://production.tilt.rest",
    }
}

pub fn dashboard_url_from_env() -> &'static str {
    match env::var("ENVIRONMENT").as_deref() {
        Ok("staging") => "https://staging.tilt.rest",
        Ok("production") => "https://production.tilt.rest",
        Ok("local") => "http://localhost:8923",
        _ => "https://production.tilt.rest",
    }
}

/// Returns the path to the Tilt configuration directory.
pub fn tilt_dir() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|p| p.join(".tilt"))
        .ok_or_else(|| anyhow!("Home directory not found"))
}
