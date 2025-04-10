pub const CUSTOM_LIB: &str = r#"use serde::{Deserialize, Serialize};
use tilt_app as tilt;

#[derive(Deserialize)]
pub struct Request {
    pub arg: String,
}

#[derive(Serialize)]
pub struct Response {
    pub arg: String,
}

#[tilt::main]
fn main(req: Request) -> Response {
    Response { arg: req.arg }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_main() {
        let req = Request {
            arg: "test".to_string(),
        };
        let res = main(req);
        assert_eq!(res.arg, "test");
    }
}
"#;

pub const CUSTOM_TOML: &str = r#"[package]
name = "{project_name}"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
serde = { version = "1.0.219", features = ["derive"] }
serde_json = "1.0.140"
tilt_app = { git = "https://github.com/tilt-network/tilt_core.git", branch = "main" }

[package.metadata.tilt]
program_id = "{program_id}"
"#;
