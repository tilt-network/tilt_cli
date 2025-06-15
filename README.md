Tilt CLI

A command-line tool for working with the Tilt network, enabling developers to create, build, test, and deploy WebAssembly programs to the Tilt platform.

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)
- WebAssembly target: `rustup target add wasm32-unknown-unknown`

### Installation

```bash
cargo install tilt-cli
```

## Usage

### Available Commands

```bash
USAGE:
    tilt [SUBCOMMAND]

SUBCOMMANDS:
    new          Creates a new Tilt project
    build        Build the Tilt project
    test         Test the Tilt project
    clean        Clean the Tilt project
    list         List Tilt programs
    deploy       Deploy the Tilt project
    signin       Sign in to Tilt
    create-job   Create a job for the current project
    help         Print this message or the help of the given subcommand(s)
```

### Creating a New Program

```bash
tilt new my-tilt-program
```

This creates a new Rust project configured for WebAssembly compilation and the Tilt platform.

### Building Your Project

```bash
cd my-tilt-project
tilt build
```

This compiles your project to WebAssembly and prepares it for deployment.

### Testing Your Project

```bash
tilt test
```

Runs unit tests for your project.

### Signing Into Tilt

```bash
tilt signin --email your.email@example.com --password your-password
```

Authenticates with the Tilt platform and stores your credentials locally.

### Deploying Your Project

```bash
tilt deploy
```

### Listing Your Programs

```bash
tilt list
```

Shows a list of your programs deployed to the Tilt platform.

## Program Template

New projects use a simple template that handles requests and responses:

```rust
use serde::{Deserialize, Serialize};
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
```

## Configuration

The `Cargo.toml` file includes Tilt-specific metadata:

```toml
[package.metadata.tilt]
program_id = "..."  # Unique identifier for your program, do not change this value
```
