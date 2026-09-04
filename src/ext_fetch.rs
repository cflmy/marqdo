//! Download official `ext/` L1 sources and native plugins from GitHub Releases.
//!
//! Enables `marqdo ext add web` without a local Rust toolchain or repo checkout.
//! Disable with `MARQDO_EXT_NO_DOWNLOAD=1`. Override tag with `MARQDO_EXT_VERSION=0.3.4`.

use std::env;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

const REPO: &str = "cflmy/marqdo";

pub fn downloads_enabled() -> bool {
    match env::var("MARQDO_EXT_NO_DOWNLOAD") {
        Ok(v) => {
            let v = v.trim();
            !(v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
        }
        Err(_) => true,
    }
}

/// SemVer without leading `v` (matches release asset names).
pub fn release_version() -> String {
    if let Ok(v) = env::var("MARQDO_EXT_VERSION") {
        let v = v.trim().trim_start_matches('v');
        if !v.is_empty() {
            return v.to_string();
        }
    }
    env!("CARGO_PKG_VERSION").to_string()
}

pub fn host_target_triple() -> Option<&'static str> {
    if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        Some("x86_64-pc-windows-msvc")
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        Some("x86_64-unknown-linux-gnu")
    } else if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        Some("aarch64-apple-darwin")
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        Some("x86_64-apple-darwin")
    } else {
        None
    }
}

fn cache_root() -> Result<PathBuf> {
    let home = env::var_os("USERPROFILE")
        .or_else(|| env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let dir = home.join(".marqdo").join("cache");
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn user_agent() -> String {
    format!("marqdo/{}", env!("CARGO_PKG_VERSION"))
}

fn download_to_file(url: &str, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = dest.with_extension("download-tmp");
    let resp = ureq::get(url)
        .set("User-Agent", &user_agent())
        .set("Accept", "application/octet-stream")
        .call()
        .with_context(|| format!("GET {url}"))?;
    if !(200..300).contains(&resp.status()) {
        bail!("download {url} returned HTTP {}", resp.status());
    }
    let mut reader = resp.into_reader();
    let mut out = File::create(&tmp).with_context(|| format!("create {}", tmp.display()))?;
    io::copy(&mut reader, &mut out).with_context(|| format!("write {}", tmp.display()))?;
    out.flush()?;
    drop(out);
    if dest.exists() {
        let _ = fs::remove_file(dest);
    }
    fs::rename(&tmp, dest).with_context(|| format!("rename {}", dest.display()))?;
    Ok(())
}

fn extract_zip(zip_path: &Path, dest_dir: &Path) -> Result<()> {
    fs::create_dir_all(dest_dir)?;
    let file = File::open(zip_path).with_context(|| format!("open {}", zip_path.display()))?;
    let mut archive = zip::ZipArchive::new(file)
        .with_context(|| format!("read zip {}", zip_path.display()))?;
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .with_context(|| format!("zip entry {i}"))?;
        let name = entry.name().to_string();
        if name.is_empty() {
            continue;
        }
        if Path::new(&name)
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
        {
            bail!("refusing zip entry with ..: {name}");
        }
        let out_path = dest_dir.join(Path::new(&name));
        if entry.is_dir() || name.ends_with('/') {
            fs::create_dir_all(&out_path)?;
            continue;
        }
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut outfile =
            File::create(&out_path).with_context(|| format!("create {}", out_path.display()))?;
        io::copy(&mut entry, &mut outfile)
            .with_context(|| format!("extract {name}"))?;
    }
    Ok(())
}

fn release_asset_url(ver: &str, filename: &str) -> String {
    format!("https://github.com/{REPO}/releases/download/v{ver}/{filename}")
}

/// Ensure L1 `ext/` tree is available (repo checkout or downloaded Release zip).
/// Returns a directory that contains `web/web.mq.md`, `ai/…`, etc.
pub fn ensure_ext_source_tree() -> Result<PathBuf> {
    let ver = release_version();
    let cache = cache_root()?.join(format!("ext-src-v{ver}"));
    let marker = cache.join("web").join("web.mq.md");
    if marker.is_file() {
        return Ok(cache);
    }
    // Zip layout from CI: top-level `ext/…`
    let nested = cache.join("ext").join("web").join("web.mq.md");
    if nested.is_file() {
        return Ok(cache.join("ext"));
    }
    if !downloads_enabled() {
        bail!("extension sources not found and downloads disabled (MARQDO_EXT_NO_DOWNLOAD)");
    }
    let zip_name = format!("marqdo-{ver}-ext.zip");
    let url = release_asset_url(&ver, &zip_name);
    let zip_path = cache_root()?.join(&zip_name);
    println!("downloading official ext sources ({url})…");
    download_to_file(&url, &zip_path)?;
    let stage = cache_root()?.join(format!("ext-src-v{ver}-extract"));
    if stage.exists() {
        let _ = fs::remove_dir_all(&stage);
    }
    extract_zip(&zip_path, &stage)?;
    // Prefer `ext/` child if present
    let src = if stage.join("ext").join("web").join("web.mq.md").is_file() {
        stage.join("ext")
    } else if stage.join("web").join("web.mq.md").is_file() {
        stage.clone()
    } else {
        bail!(
            "downloaded {zip_name} but could not find web/web.mq.md inside; check release assets"
        );
    };
    if cache.exists() {
        let _ = fs::remove_dir_all(&cache);
    }
    // Move/copy tree into stable cache path
    copy_dir_recursive(&src, &cache)?;
    let _ = fs::remove_dir_all(&stage);
    if !cache.join("web").join("web.mq.md").is_file() {
        bail!("failed to cache ext sources under {}", cache.display());
    }
    println!("cached ext sources at {}", cache.display());
    Ok(cache)
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src).with_context(|| format!("read {}", src.display()))? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let to = dest.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &to)?;
        } else if ty.is_file() {
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &to)
                .with_context(|| format!("copy {} → {}", entry.path().display(), to.display()))?;
        }
    }
    Ok(())
}

/// Download prebuilt native plugins zip for this host; return path to `lib` file for `short`.
pub fn download_native_plugin(short: &str, lib_name: &str) -> Result<PathBuf> {
    if !downloads_enabled() {
        bail!("native plugin not found and downloads disabled (MARQDO_EXT_NO_DOWNLOAD)");
    }
    let Some(triple) = host_target_triple() else {
        bail!(
            "no prebuilt native plugins for this platform; install Rust and run `cargo build -p marqdo_plugin_{short}`"
        );
    };
    let ver = release_version();
    let zip_name = format!("marqdo-{ver}-native-{triple}.zip");
    let url = release_asset_url(&ver, &zip_name);
    let cache = cache_root()?.join(format!("native-v{ver}-{triple}"));
    let lib_in_cache = cache.join("native").join(lib_name);
    if lib_in_cache.is_file() {
        return Ok(lib_in_cache);
    }
    // Also accept flat layout
    let flat = cache.join(lib_name);
    if flat.is_file() {
        return Ok(flat);
    }
    let zip_path = cache_root()?.join(&zip_name);
    println!("downloading prebuilt native plugins ({url})…");
    download_to_file(&url, &zip_path).with_context(|| {
        format!(
            "failed to download prebuilt plugins for {triple}. \
             Build locally with `cargo build --release -p marqdo_plugin_{short}`, \
             or check https://github.com/{REPO}/releases/tag/v{ver}"
        )
    })?;
    if cache.exists() {
        let _ = fs::remove_dir_all(&cache);
    }
    fs::create_dir_all(&cache)?;
    extract_zip(&zip_path, &cache)?;
    if lib_in_cache.is_file() {
        println!("cached {}", lib_in_cache.display());
        return Ok(lib_in_cache);
    }
    if flat.is_file() {
        return Ok(flat);
    }
    // Search recursively for the lib name
    if let Some(found) = find_named_file(&cache, lib_name)? {
        return Ok(found);
    }
    bail!(
        "downloaded {zip_name} but missing {lib_name}; expected native/{lib_name} in the archive"
    )
}

fn find_named_file(root: &Path, name: &str) -> Result<Option<PathBuf>> {
    fn walk(dir: &Path, name: &str, out: &mut Option<PathBuf>) -> Result<()> {
        if out.is_some() {
            return Ok(());
        }
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let p = entry.path();
            if entry.file_type()?.is_dir() {
                walk(&p, name, out)?;
            } else if entry.file_name() == name {
                *out = Some(p);
            }
        }
        Ok(())
    }
    let mut found = None;
    walk(root, name, &mut found)?;
    Ok(found)
}

pub fn cargo_available() -> bool {
    std::process::Command::new("cargo")
        .arg("-V")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_url_shape() {
        let u = release_asset_url("0.3.4", "marqdo-0.3.4-ext.zip");
        assert_eq!(
            u,
            "https://github.com/cflmy/marqdo/releases/download/v0.3.4/marqdo-0.3.4-ext.zip"
        );
    }

    #[test]
    fn triple_known_on_ci_hosts() {
        // At least one of these builds is what we ship.
        let t = host_target_triple();
        if cfg!(any(
            all(target_os = "linux", target_arch = "x86_64"),
            all(target_os = "windows", target_arch = "x86_64"),
            all(target_os = "macos", target_arch = "aarch64"),
            all(target_os = "macos", target_arch = "x86_64"),
        )) {
            assert!(t.is_some());
        }
    }
}
