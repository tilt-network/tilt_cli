use anyhow::{Result, anyhow};

pub enum ProjectKind {}

pub fn detect_project_kind() -> Result<ProjectKind> {
    Ok(ProjectKind::default())
}
