use anyhow::{Context, Result};
use clap::Args;
use std::{fs, process::Command};

pub const CUSTOM_LIB: &str = include_str!("../../static/rust/lib.rs.template");
pub const TILT_BINDINGS: &str = include_str!("../../static/rust/tilt.rs.template");
pub const CUSTOM_TOML: &str = include_str!("../../static/rust/Cargo.toml.template");
pub const WIT_FILE: &str = include_str!("../../static/tilt_sdk.wit.template");
pub const GO_MOD: &str = include_str!("../../static/go/go.mod.template");
pub const GO_MAIN: &str = include_str!("../../static/go/main.go.template");
pub const GO_APP: &str = include_str!("../../static/go/app.go.template");
pub const GO_APP_TEST: &str = include_str!("../../static/go/app_test.go.template");
pub const GO_CABI: &str = include_str!("../../static/go/cabi.go.template");
pub const GO_WIT: &str = include_str!("../../static/go/component.wit.template");
pub const GO_WKG_LOCK: &str = include_str!("../../static/go/wkg.lock.template");
pub const PY_APP: &str = include_str!("../../static/python/app.py.template");
pub const PY_WIT: &str = include_str!("../../static/python/component.wit.template");
pub const PY_PYPROJECT: &str = include_str!("../../static/python/pyproject.toml.template");

#[derive(Debug, Args)]
pub struct New {
    #[arg(short = 'n', long)]
    name: String,
    #[arg(long, default_value = "rust", value_parser = ["rust", "go", "python"])]
    template: String,
}

impl New {
    pub async fn run(&self) -> Result<()> {
        let name = &self.name;
        match self.template.as_str() {
            "go" => self.new_go(name),
            "python" => self.new_python(name),
            _ => self.new_rust(name),
        }
    }

    fn new_rust(&self, name: &str) -> Result<()> {
        let output = Command::new("cargo")
            .args(["new", "--lib", name])
            .output()
            .context("Failed to create a new program. Do you have Rust installed?")?;

        if !output.status.success() {
            match output.status.code() {
                Some(101) => {
                    anyhow::bail!("Error: Project '{name}' already exists! Try something new.")
                }
                _ => eprintln!(
                    "Error: something went wrong while creating the project '{name}': {}. ",
                    String::from_utf8_lossy(&output.stderr)
                ),
            }
        }

        let custom_toml = CUSTOM_TOML.replace("{project_name}", name);

        let status = Command::new("rustup")
            .args(["target", "add", "wasm32-wasip2"])
            .status()
            .expect("Failed to add wasm32-wasip2 target");

        if !status.success() {
            eprintln!("Failed to add WebAssembly target");
        }

        fs::create_dir_all(format!("{name}/wit")).expect("Failed to create directory");
        fs::write(format!("{name}/src/lib.rs"), CUSTOM_LIB).expect("Failed to write lib.rs");
        fs::write(format!("{name}/Cargo.toml"), custom_toml).expect("Failed to write Cargo.toml");
        fs::write(format!("{name}/src/tilt.rs"), TILT_BINDINGS).expect("Failed to write tilt.rs");
        fs::write(format!("{name}/wit/tilt_sdk.wit"), WIT_FILE)
            .expect("Failed to write tilt_sdk.wit");

        println!("Project '{name}' created successfully!");
        println!("      cd ./{name}");
        println!("      tilt test");

        Ok(())
    }

    fn new_go(&self, name: &str) -> Result<()> {
        let go_mod = GO_MOD.replace("{module_name}", name);
        let go_main = GO_MAIN.replace("{module_name}", name);

        fs::create_dir_all(format!("{name}/wit")).context("Failed to create wit directory")?;
        fs::create_dir_all(format!("{name}/app")).context("Failed to create app directory")?;
        fs::write(format!("{name}/go.mod"), go_mod).context("Failed to write go.mod")?;
        fs::write(format!("{name}/main.go"), go_main).context("Failed to write main.go")?;
        fs::write(format!("{name}/app/app.go"), GO_APP).context("Failed to write app/app.go")?;
        fs::write(format!("{name}/app/app_test.go"), GO_APP_TEST)
            .context("Failed to write app/app_test.go")?;
        fs::write(format!("{name}/cabi.go"), GO_CABI).context("Failed to write cabi.go")?;
        fs::write(format!("{name}/wit/component.wit"), GO_WIT)
            .context("Failed to write component.wit")?;
        fs::write(format!("{name}/wkg.lock"), GO_WKG_LOCK).context("Failed to write wkg.lock")?;

        let tidy = Command::new("go")
            .args(["mod", "tidy"])
            .current_dir(name)
            .status()
            .context("Failed to run go mod tidy. Do you have Go installed?")?;

        if !tidy.success() {
            anyhow::bail!("go mod tidy failed");
        }

        let fetch = Command::new("wkg")
            .args(["wit", "fetch"])
            .current_dir(name)
            .status()
            .context(
                "Failed to run wkg wit fetch. Do you have wkg installed? Run: cargo install wkg",
            )?;

        if !fetch.success() {
            anyhow::bail!("wkg wit fetch failed");
        }

        let bindgen = Command::new("go")
            .args([
                "tool",
                "wit-bindgen-go",
                "generate",
                "--world",
                "tilt",
                "--out",
                "internal",
                "wit/",
            ])
            .current_dir(name)
            .status()
            .context("Failed to run wit-bindgen-go")?;

        if !bindgen.success() {
            anyhow::bail!("wit-bindgen-go generate failed");
        }

        patch_wasmexport_compat(name)?;

        println!("Project '{name}' created successfully!");
        println!("      cd ./{name}");
        println!("      tilt test");

        Ok(())
    }

    fn new_python(&self, name: &str) -> Result<()> {
        let pyproject = PY_PYPROJECT.replace("{project_name}", name);

        fs::create_dir_all(format!("{name}/wit")).context("Failed to create wit directory")?;

        fs::write(format!("{name}/app.py"), PY_APP).context("Failed to write app.py")?;
        fs::write(format!("{name}/wit/component.wit"), PY_WIT)
            .context("Failed to write component.wit")?;
        fs::write(format!("{name}/pyproject.toml"), pyproject)
            .context("Failed to write pyproject.toml")?;

        let bindings = Command::new("componentize-py")
            .args(["-d", "wit/", "-w", "tilt", "bindings", "."])
            .current_dir(name)
            .status()
            .context("Failed to run componentize-py. Do you have it installed? Run:pip install componentize-py")?;

        if !bindings.success() {
            anyhow::bail!("componentize-py bindings generation failed");
        }

        println!("Project {name} created successfully!");
        println!("      cd./{name}");
        println!("      tilt build");

        Ok(())
    }
}

// go:wasmexport in Go 1.24+ only allows unsafe.Pointer as a pointer return type.
// wit-bindgen-go generates *cm.Result[...] which the compiler rejects.
// Both are i32 in WASM linear memory, so the patch is ABI-compatible.
fn patch_wasmexport_compat(name: &str) -> Result<()> {
    let path = format!("{name}/internal/tilt/app/tilt/tilt.wasm.go");
    let content = fs::read_to_string(&path).context("Failed to read generated tilt.wasm.go")?;

    let patched = content
        .replace(
            "\t\"go.bytecodealliance.org/cm\"\n)",
            "\t\"go.bytecodealliance.org/cm\"\n\t\"unsafe\"\n)",
        )
        .replace(
            "\tresult = &result_\n\treturn\n}",
            "\treturn unsafe.Pointer(&result_)\n}",
        )
        .replace("//go:wasmexport execute\n", "//export execute\n");

    let patched = if let Some(start) = patched.find(") (result *cm.Result[") {
        if let Some(end) = patched[start..].find("]) {") {
            let before = &patched[..start];
            let after = &patched[start + end + 4..];
            format!("{before}) unsafe.Pointer {{{after}")
        } else {
            patched
        }
    } else {
        patched
    };

    fs::write(&path, patched).context("Failed to write patched tilt.wasm.go")?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_new_creates_project_files() {
        let temp_dir = tempdir().unwrap();
        let name = temp_dir.path().join("my_project");
        let name_str = name.to_str().unwrap().to_string();

        let cmd = New {
            name: name_str,
            template: "rust".to_string(),
        };
        cmd.run().await.unwrap();

        assert!(name.join("src/lib.rs").exists());
    }

    #[tokio::test]
    async fn test_new_toml_has_project_name() {
        let temp_dir = tempdir().unwrap();
        let name = temp_dir.path().join("my_project");
        let name_str = name.to_str().unwrap().to_string();

        let cmd = New {
            name: name_str.clone(),
            template: "rust".to_string(),
        };
        cmd.run().await.unwrap();

        let toml = fs::read_to_string(name.join("Cargo.toml")).unwrap();
        assert!(toml.contains(&name_str));
    }
}
