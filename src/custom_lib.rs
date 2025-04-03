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
"#;

pub const CUSTOM_TOML: &str = r#"[package]
name = "app"
version = "0.1.0"
edition = "2024"

[net]
git-fetch-with-cli = true

[lib]
crate-type = ["cdylib"]

[dependencies]
serde = { version = "1.0.219", features = ["derive"] }
serde_json = "1.0.140"
tilt_app = { git = "ssh://git@github.com/tilt-network/tilt_core.git", branch = "main" }
"#;
