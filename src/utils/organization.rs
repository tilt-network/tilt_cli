use anyhow::{Context, Result};
use std::fs;

use crate::utils::tilt_dir;

/// Return the organization ID previously saved by the sign-in command.
pub fn load_selected_organization_id() -> Result<String> {
    let path = tilt_dir()?.join("organization_id_selected");
    let organization_id = fs::read_to_string(&path).unwrap_or_default();
    Ok(organization_id.trim().trim_matches('"').to_string())
}
