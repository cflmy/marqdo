//! Download official `ext/` L1 sources and native plugins.
//!
//! Enables `marqdo ext add web` without a local Rust toolchain or repo checkout.
//!
//! Disable with `MARQDO_EXT_NO_DOWNLOAD=1`. Override pack SemVer with
//! `MARQDO_EXT_VERSION` (independent of CLI `CARGO_PKG_VERSION`).
//!
//! Download order (first success wins):
//! 1. `MARQDO_EXT_DOWNLOAD_BASE` (optional override)
//! 2. **CDN** `https://ext.marqdo.com` (Cloudflare R2 custom domain)
//! 3. GitHub Releases `github.com/cflmy/marqdo`
//! 4. GitHub via reverse proxy `proxy.cflmy.top`

use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

use anyhow::{bail, Context, Result};

const REPO: &str = "cflmy/marqdo";
/// Public CDN for extension packs (R2 bucket `marqdo`, custom domain).
pub const EXT_CDN_BASE: &str = "https://ext.marqdo.com";
/// HTTPS reverse proxy used when github.com is slow or blocked.
const GITHUB_MIRROR_PREFIX: &str = "https://proxy.cflmy.top/github.com";
/// Packaged with the repo; may diverge from CLI SemVer for ext-only releases.
const EMBEDDED_EXT_VERSION: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ext/VERSION"));

pub fn downloads_enabled() -> bool {
    match env::var("MARQDO_EXT_NO_DOWNLOAD") {
        Ok(v) => {
            let v = v.trim();
            !(v == "1" || v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("yes"))
        }
        Err(_) => true,
    }
}

fn normalize_semver(raw: &str) -> Option<String> {
    let v = raw.trim().trim_start_matches('v').trim();
    if v.is_empty() {
        return None;
    }
    // one line only
    let v = v.lines().next()?.trim().trim_start_matches('v').trim();
    if v.is_empty() {
        return None;
    }
    Some(v.to_string())
}

/// SemVer of the **extension pack** (may differ from CLI version).
///
/// Resolution order:
/// 1. `MARQDO_EXT_VERSION`
/// 2. CDN `GET {EXT_CDN}/latest/VERSION` (short timeout; skipped if downloads disabled)
/// 3. Embedded `ext/VERSION` at build time
/// 4. CLI `CARGO_PKG_VERSION`
pub fn release_version() -> String {
    if let Ok(v) = env::var("MARQDO_EXT_VERSION") {
        if let Some(n) = normalize_semver(&v) {
            return n;
        }
    }
    if downloads_enabled() {
        if let Some(n) = fetch_cdn_latest_version() {
            return n;
        }
    }
    if let Some(n) = normalize_semver(EMBEDDED_EXT_VERSION) {
        return n;
    }
    env!("CARGO_PKG_VERSION").to_string()
}

fn fetch_cdn_latest_version() -> Option<String> {
    static CACHED: OnceLock<Option<String>> = OnceLock::new();
    CACHED
        .get_or_init(|| {
            let base = env::var("MARQDO_EXT_CDN")
                .ok()
                .map(|s| s.trim().trim_end_matches('/').to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| EXT_CDN_BASE.to_string());
            let url = format!("{base}/latest/VERSION");
            let agent = ureq::AgentBuilder::new()
                .timeout_connect(Duration::from_secs(5))
                .timeout_read(Duration::from_secs(10))
                .user_agent(&user_agent())
                .build();
            let resp = agent.get(&url).call().ok()?;
            if !(200..300).contains(&resp.status()) {
                return None;
            }
            let mut body = String::new();
            resp.into_reader().read_to_string(&mut body).ok()?;
            normalize_semver(&body)
        })
        .clone()
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

fn http_agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(Duration::from_secs(20))
        .timeout_read(Duration::from_secs(600))
        .user_agent(&user_agent())
        .build()
}

fn cdn_base() -> String {
    env::var("MARQDO_EXT_CDN")
        .ok()
        .map(|s| s.trim().trim_end_matches('/').to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| EXT_CDN_BASE.to_string())
}

/// Candidate download URLs for a release asset.
///
/// Order: optional `MARQDO_EXT_DOWNLOAD_BASE` → CDN → GitHub → GitHub proxy.
pub fn release_asset_urls(ver: &str, filename: &str) -> Vec<String> {
    let mut urls = Vec::new();
    if let Ok(base) = env::var("MARQDO_EXT_DOWNLOAD_BASE") {
        let base = base.trim().trim_end_matches('/');
        if !base.is_empty() {
            urls.push(format!("{base}/{REPO}/releases/download/v{ver}/{filename}"));
            urls.push(format!("{base}/releases/download/v{ver}/{filename}"));
            urls.push(format!("{base}/v{ver}/{filename}"));
            urls.push(format!("{base}/{filename}"));
        }
    }
    let cdn = cdn_base();
    // Preferred public layout on ext.marqdo.com / R2
    urls.push(format!("{cdn}/v{ver}/{filename}"));
    urls.push(format!("{cdn}/releases/v{ver}/{filename}"));
    urls.push(format!("{cdn}/{filename}"));
    urls.push(format!(
        "https://github.com/{REPO}/releases/download/v{ver}/{filename}"
    ));
    urls.push(format!(
        "{GITHUB_MIRROR_PREFIX}/{REPO}/releases/download/v{ver}/{filename}"
    ));
    let mut seen = std::collections::HashSet::new();
    urls.into_iter().filter(|u| seen.insert(u.clone())).collect()
}

fn download_to_file(url: &str, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = dest.with_extension("download-tmp");
    let resp = http_agent()
        .get(url)
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

fn download_first_ok(urls: &[String], dest: &Path) -> Result<()> {
    let mut errors: Vec<String> = Vec::new();
    for (i, url) in urls.iter().enumerate() {
        if i == 0 {
            println!("downloading ({url})…");
        } else {
            println!("retry ({url})…");
        }
        match download_to_file(url, dest) {
            Ok(()) => return Ok(()),
            Err(e) => {
                eprintln!("note: {e:#}");
                errors.push(format!("{url} → {e:#}"));
                let _ = fs::remove_file(dest.with_extension("download-tmp"));
            }
        }
    }
    bail!(
        "all download mirrors failed for {}:\n  - {}",
        dest.file_name().and_then(|s| s.to_str()).unwrap_or("asset"),
        errors.join("\n  - ")
    )
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

/// Ensure L1 `ext/` tree is available (repo checkout or downloaded Release zip).
/// Returns a directory that contains `web/web.mq.md`, `ai/…`, etc.
pub fn ensure_ext_source_tree() -> Result<PathBuf> {
    let ver = release_version();
    let cache = cache_root()?.join(format!("ext-src-v{ver}"));
    let marker = cache.join("web").join("web.mq.md");
    if marker.is_file() {
        return Ok(cache);
    }
    let nested = cache.join("ext").join("web").join("web.mq.md");
    if nested.is_file() {
        return Ok(cache.join("ext"));
    }
    if !downloads_enabled() {
        bail!("extension sources not found and downloads disabled (MARQDO_EXT_NO_DOWNLOAD)");
    }
    let zip_name = format!("marqdo-{ver}-ext.zip");
    let urls = release_asset_urls(&ver, &zip_name);
    let zip_path = cache_root()?.join(&zip_name);
    download_first_ok(&urls, &zip_path)?;
    let stage = cache_root()?.join(format!("ext-src-v{ver}-extract"));
    if stage.exists() {
        let _ = fs::remove_dir_all(&stage);
    }
    extract_zip(&zip_path, &stage)?;
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
    let urls = release_asset_urls(&ver, &zip_name);
    let cache = cache_root()?.join(format!("native-v{ver}-{triple}"));
    let lib_in_cache = cache.join("native").join(lib_name);
    if lib_in_cache.is_file() {
        return Ok(lib_in_cache);
    }
    let flat = cache.join(lib_name);
    if flat.is_file() {
        return Ok(flat);
    }
    let zip_path = cache_root()?.join(&zip_name);
    download_first_ok(&urls, &zip_path).with_context(|| {
        format!(
            "failed to download prebuilt plugins for {triple}. \
             Set MARQDO_EXT_CDN / MARQDO_EXT_DOWNLOAD_BASE, build locally, \
             or check https://github.com/{REPO}/releases and https://ext.marqdo.com/v{ver}/"
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
    fn asset_urls_prefer_cdn_then_github_then_proxy() {
        let u = release_asset_urls("1.0.2", "marqdo-1.0.2-ext.zip");
        assert!(u[0].starts_with("https://ext.marqdo.com/"));
        assert!(u.iter().any(|x| x.contains("github.com/cflmy/marqdo")));
        assert!(u
            .iter()
            .any(|x| x.contains("proxy.cflmy.top/github.com/cflmy/marqdo")));
        let gh = u
            .iter()
            .position(|x| x.starts_with("https://github.com/"))
            .unwrap();
        let proxy = u
            .iter()
            .position(|x| x.contains("proxy.cflmy.top"))
            .unwrap();
        assert!(gh < proxy);
    }

    #[test]
    fn embedded_ext_version_parses() {
        assert!(normalize_semver(EMBEDDED_EXT_VERSION).is_some());
    }

    #[test]
    fn triple_known_on_ci_hosts() {
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
