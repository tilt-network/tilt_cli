use crate::utils::tilt_dir;
use anyhow::Result;
use std::fs;

/// Return the authentication token previously saved by the sign-in command.
pub fn load_auth_token() -> Result<String> {
    let token = fs::read_to_string(tilt_dir()?.join("auth_token"))?;
    Ok(token.trim().to_string())
}
