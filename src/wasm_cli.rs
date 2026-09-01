//! `marqdo wasm build` — compile `marqdo-wasm` and copy artifacts for the browser host.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};

/// Build size-optimized wasm32 artifact and copy into `out_dir`:
/// - `marqdo_wasm.wasm`
/// - `marqdo-bridge.js` (canonical host glue)
pub fn build_wasm(out_dir: &Path) -> Result<()> {
    let status = Command::new("cargo")
        .args([
            "build",
            "-p",
            "marqdo-wasm",
            "--target",
            "wasm32-unknown-unknown",
            "--profile",
            "release-wasm",
        ])
        .status()
        .context("failed to spawn cargo (is it on PATH?)")?;
    if !status.success() {
        bail!("cargo build -p marqdo-wasm --profile release-wasm failed with {status}");
    }

    let mut artifact = find_wasm_artifact()?;
    std::fs::create_dir_all(out_dir)
        .with_context(|| format!("create {}", out_dir.display()))?;

    if let Some(optimized) = try_wasm_opt(&artifact)? {
        artifact = optimized;
    }

    let dest_wasm = out_dir.join("marqdo_wasm.wasm");
    std::fs::copy(&artifact, &dest_wasm).with_context(|| {
        format!(
            "copy {} → {}",
            artifact.display(),
            dest_wasm.display()
        )
    })?;
    let bytes = std::fs::metadata(&dest_wasm)
        .map(|m| m.len())
        .unwrap_or(0);
    println!(
        "wrote {} ({:.1} KiB)",
        dest_wasm.display(),
        bytes as f64 / 1024.0
    );

    let bridge_src = find_bridge_js()?;
    let dest_bridge = out_dir.join("marqdo-bridge.js");
    std::fs::copy(&bridge_src, &dest_bridge).with_context(|| {
        format!(
            "copy {} → {}",
            bridge_src.display(),
            dest_bridge.display()
        )
    })?;
    println!("wrote {}", dest_bridge.display());
    Ok(())
}

fn find_bridge_js() -> Result<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        candidates.push(
            PathBuf::from(manifest).join("crates/marqdo-wasm/js/marqdo-bridge.js"),
        );
    }
    candidates.push(PathBuf::from("crates/marqdo-wasm/js/marqdo-bridge.js"));
    // When running from installed binary, try next to current_exe ../../…
    if let Ok(exe) = std::env::current_exe() {
        if let Some(root) = exe.parent().and_then(|p| p.parent()).and_then(|p| p.parent()) {
            candidates.push(root.join("crates/marqdo-wasm/js/marqdo-bridge.js"));
        }
    }
    for c in &candidates {
        if c.is_file() {
            return Ok(c.clone());
        }
    }
    bail!(
        "marqdo-bridge.js not found; tried: {}",
        candidates
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn try_wasm_opt(input: &Path) -> Result<Option<PathBuf>> {
    let which = Command::new("wasm-opt").arg("--version").output();
    let Ok(out) = which else {
        return Ok(None);
    };
    if !out.status.success() {
        return Ok(None);
    }
    let tmp = input.with_extension("opt.wasm");
    let status = Command::new("wasm-opt")
        .args(["-Oz", "--enable-bulk-memory"])
        .arg(input)
        .arg("-o")
        .arg(&tmp)
        .status()
        .context("wasm-opt spawn failed")?;
    if !status.success() {
        let status2 = Command::new("wasm-opt")
            .args(["-Oz"])
            .arg(input)
            .arg("-o")
            .arg(&tmp)
            .status()
            .context("wasm-opt spawn failed")?;
        if !status2.success() {
            eprintln!("warning: wasm-opt failed; keeping cargo artifact");
            return Ok(None);
        }
    }
    println!("wasm-opt: {}", tmp.display());
    Ok(Some(tmp))
}

fn find_wasm_artifact() -> Result<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(td) = std::env::var("CARGO_TARGET_DIR") {
        candidates.push(
            PathBuf::from(&td).join("wasm32-unknown-unknown/release-wasm/marqdo_wasm.wasm"),
        );
        candidates.push(
            PathBuf::from(td).join("wasm32-unknown-unknown/release/marqdo_wasm.wasm"),
        );
    }
    candidates.push(PathBuf::from(
        "target/wasm32-unknown-unknown/release-wasm/marqdo_wasm.wasm",
    ));
    candidates.push(PathBuf::from(
        "target/wasm32-unknown-unknown/release/marqdo_wasm.wasm",
    ));
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
