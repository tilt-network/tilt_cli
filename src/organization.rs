use std::fs;
use std::io::{self, ErrorKind};

pub fn save_selected_organization_id(id: String) -> io::Result<()> {
    let path = dirs::home_dir()
        .map(|p| p.join(".tilt/organization_id_selected"))
        .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "Home directory not found"))?;

    fs::write(path, id)
}

pub fn load_selected_organization_id() -> io::Result<String> {
    let path = dirs::home_dir()
        .map(|p| p.join(".tilt/organization_id_selected"))
        .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "Home directory not found"))?;

    fs::read_to_string(path).map(|s| s.trim().to_string())
}
