//! `marqdo wasm build` — compile `marqdo-wasm` and copy the `.wasm` artifact.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};

/// Build release wasm32 artifact and copy `marqdo_wasm.wasm` into `out_dir`.
pub fn build_wasm(out_dir: &Path) -> Result<()> {
    let status = Command::new("cargo")
        .args([
            "build",
            "-p",
            "marqdo-wasm",
            "--target",
            "wasm32-unknown-unknown",
            "--release",
        ])
        .status()
        .context("failed to spawn cargo (is it on PATH?)")?;
    if !status.success() {
        bail!("cargo build -p marqdo-wasm failed with {status}");
    }

    let artifact = find_wasm_artifact()?;
    std::fs::create_dir_all(out_dir)
        .with_context(|| format!("create {}", out_dir.display()))?;
    let dest = out_dir.join("marqdo_wasm.wasm");
    std::fs::copy(&artifact, &dest).with_context(|| {
        format!(
            "copy {} → {}",
            artifact.display(),
            dest.display()
        )
    })?;
    println!("wrote {}", dest.display());
    Ok(())
}

fn find_wasm_artifact() -> Result<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(td) = std::env::var("CARGO_TARGET_DIR") {
        candidates.push(
            PathBuf::from(td)
                .join("wasm32-unknown-unknown/release/marqdo_wasm.wasm"),
        );
    }
    candidates.push(PathBuf::from(
        "target/wasm32-unknown-unknown/release/marqdo_wasm.wasm",
    ));
    // Workspace may place target next to crates when invoked oddly — still try root.
    for c in &candidates {
        if c.is_file() {
            return Ok(c.clone());
        }
    }
    bail!(
        "built wasm not found; tried: {}",
        candidates
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )
}
