use anyhow::Result;
use std::fs::{self, read_to_string};

use crate::helpers::tilt_dir;

pub fn save_selected_organization_id(id: &str) -> Result<()> {
    let path = tilt_dir()?.join("organization_id_selected");
    fs::write(path, id)?;
    Ok(())
}

pub fn load_selected_organization_id() -> Result<String> {
    let path = tilt_dir()?.join("organization_id_selected");
    let organization_id = read_to_string(path)?;
    Ok(organization_id.trim().to_string())
}
