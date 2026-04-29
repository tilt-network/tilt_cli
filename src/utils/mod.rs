mod auth;
mod organization;
mod project;

use anyhow::{Result, anyhow};
use std::{env, path::PathBuf, time::Duration};

pub use auth::*;
pub use organization::*;
pub use project::*;

/// Returns the base URL to be used for API requests.
pub fn url_from_env() -> &'static str {
    match env::var("ENVIRONMENT").as_deref() {
        Ok("staging") => "https://staging.tilt.rest",
        Ok("production") => "https://production.tilt.rest",
        Ok("local") => "http://localhost:3000",
        _ => "https://production.tilt.rest",
    }
}

/// Returns the HTTP timeout used for API requests.
pub fn http_timeout_from_env() -> Duration {
    let secs = env::var("TILT_HTTP_TIMEOUT_SECS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(30);
    Duration::from_secs(secs)
}

/// Returns the path to the Tilt configuration directory.
pub fn tilt_dir() -> Result<PathBuf> {
    dirs::home_dir()
        .map(|p| p.join(".tilt"))
        .ok_or_else(|| anyhow!("Home directory not found"))
}
