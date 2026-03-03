# Tilt CLI

A command-line tool for working with the Tilt network, enabling developers to create, build, test, and deploy WebAssembly programs to the Tilt platform.

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) and Cargo
- WebAssembly target: `rustup target add wasm32-wasip2`

### From crates.io

```bash
cargo install tilt-cli
```

### From source

```bash
git clone https://github.com/your-org/tilt_cli
cd tilt_cli
make install
```

## Usage

### Available Commands

```bash
USAGE:
    tilt [SUBCOMMAND]

SUBCOMMANDS:
    new       Create a new Tilt program
    build     Build the program to WebAssembly
    test      Run the program's tests
    list      List your deployed programs
    deploy    Deploy the program to the Tilt network
    signin    Sign in to your Tilt account
    help      Print this message or the help of the given subcommand(s)
```

### Creating a New Program

```bash
tilt new --name my-tilt-program
```

This scaffolds a new Rust project pre-configured for WebAssembly compilation and the Tilt platform. The generated structure includes:

```
my-tilt-program/
├── src/
│   ├── lib.rs       # Your program's entry point
│   └── tilt.rs      # Tilt bindings (do not edit)
├── wit/
│   └── tilt_sdk.wit # WIT interface definition
└── Cargo.toml
```

### Building Your Program

```bash
cd my-tilt-program
tilt build
```

Compiles the project to WebAssembly (`wasm32-wasip2`) in release mode, generating the `.wasm` binary under `target/wasm32-wasip2/release/`.

### Testing Your Program

```bash
tilt test
```

Runs the unit tests via `cargo test`.

### Signing In

```bash
tilt signin --secret-key <your_secret_key>
```

You can find your secret key in the **Tilt Console under** **User → Settings**, right after selecting your organization.

### Deploying Your Program

```bash
tilt deploy
```

Builds the project (if not already built) and uploads the `.wasm` binary to the Tilt network under your authenticated organization. Requires a prior `tilt signin`.

### Listing Your Programs

```bash
tilt list
```

Shows all programs deployed to your organization on the Tilt platform.

## Program Template

New projects start with a simple template that receives and returns raw bytes:

```rust
mod tilt;

use tilt::*;

struct App;

impl Tilt for App {
    fn execute(req: Vec<u8>) -> Result<Vec<u8>, Error> {
        // Your logic here.
        Ok(req)
    }
}

export!(App with_types_in tilt);
```

## Configuration

The generated `Cargo.toml` includes Tilt-specific metadata used during deployment:

```toml
[package.metadata.tilt]
program_id = "..."  # Unique identifier for your program — do not change this value

[package.metadata.component]
package = "tilt:sdk"
```
