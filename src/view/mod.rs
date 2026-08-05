//! `marqdo view` — local HTML browser for `.mq.md` structure + output.

mod html;
mod output;
mod render;

pub use output::{write_static, OutputOptions};

use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

use crate::input_feed::{effective_stdin, split_stdin_text};
use crate::view::html::{escape, page_file, page_index, LinkMode};
use crate::view::render::{collect_input_prompts, render_module_structure, FileViewModel};
use crate::{run_file_capture, RunOptions};

pub struct ViewOptions {
    pub path: PathBuf,
    pub host: String,
    pub port: u16,
    pub open_browser: bool,
}

pub(crate) struct ViewRoot {
    pub root: PathBuf,
    pub only_file: Option<PathBuf>,
    pub files: Vec<PathBuf>,
}

/// Block serving until Ctrl+C (process kill).
pub fn serve(opts: ViewOptions) -> Result<()> {
    let root_info = build_root(&opts.path)?;
    let addr: SocketAddr = format!("{}:{}", opts.host, opts.port)
        .parse()
        .context("invalid host/port")?;
    let server = Server::http(addr).map_err(|e| anyhow::anyhow!("listen {addr}: {e}"))?;
    let url = format!("http://{}:{}/", opts.host, opts.port);
    eprintln!("marqdo view: {url}");
    eprintln!("root: {}", root_info.root.display());
    eprintln!("{} .mq.md file(s). Ctrl+C to stop.", root_info.files.len());

    if opts.open_browser {
        let _ = open_url(&url);
    }

    for request in server.incoming_requests() {
        if let Err(err) = handle(&root_info, request) {
            eprintln!("view request error: {err:#}");
        }
    }
    Ok(())
}

pub(crate) fn build_root(path: &Path) -> Result<ViewRoot> {
    let path = if path.exists() {
        path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
    } else {
        bail!("path not found: {}", path.display());
    };

    if path.is_file() {
        if !is_mq_md(&path) {
            bail!("not a .mq.md file: {}", path.display());
        }
        let root = path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));
        let rel = PathBuf::from(path.file_name().unwrap());
        return Ok(ViewRoot {
            root,
            only_file: Some(rel.clone()),
            files: vec![rel],
        });
    }

    if !path.is_dir() {
        bail!("not a file or directory: {}", path.display());
    }

    let mut files = Vec::new();
    collect_mq_md(&path, &path, &mut files)?;
    files.sort();
    Ok(ViewRoot {
        root: path,
        only_file: None,
        files,
    })
}

fn collect_mq_md(root: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("read_dir {}", dir.display()))? {
        let entry = entry?;
        let p = entry.path();
        if p.is_dir() {
            collect_mq_md(root, &p, out)?;
        } else if is_mq_md(&p) {
            let rel = p.strip_prefix(root).unwrap_or(&p).to_path_buf();
            out.push(rel);
        }
    }
    Ok(())
}

fn is_mq_md(path: &Path) -> bool {
    path.file_name()
        .and_then(|s| s.to_str())
        .map(|s| s.ends_with(".mq.md"))
        .unwrap_or(false)
}

fn handle(root: &ViewRoot, request: Request) -> Result<()> {
    let url = request.url().to_string();
    let method = request.method().clone();

    if method == Method::Post {
        let (path_part, _) = split_url(&url);
        if path_part == "/api/foreign-run" {
            return api_foreign_run(root, request);
        }
        let resp = Response::from_string("method not allowed").with_status_code(StatusCode(405));
        let _ = request.respond(resp);
        return Ok(());
    }

    if method != Method::Get {
        let resp = Response::from_string("method not allowed").with_status_code(StatusCode(405));
        let _ = request.respond(resp);
        return Ok(());
    }

    let (path_part, query) = split_url(&url);
    match path_part {
        "/" | "/index.html" => {
            // Default: open the first file in the folder (sorted), no welcome pick screen.
            if let Some(first) = root.files.first() {
                let rel = first.to_string_lossy().replace('\\', "/");
                let resolved = resolve_rel(root, &rel)?;
                let vm = build_file_view(root, &resolved, &rel, &[], true)?;
                let body = page_file(&root.files, &rel, &vm, &LinkMode::Live);
                respond_html(request, body)
            } else {
                let body = page_index(&root.files, None, &LinkMode::Live);
                respond_html(request, body)
            }
        }
        "/file" => {
            let rel = query_param(query, "path").unwrap_or_default();
            let stdin_raw = query_param(query, "stdin").unwrap_or_default();
            let stdin_lines = split_stdin_text(&stdin_raw);
            let resolved = resolve_rel(root, &rel)?;
            let vm = build_file_view(root, &resolved, &rel, &stdin_lines, true)?;
            let body = page_file(&root.files, &rel, &vm, &LinkMode::Live);
            respond_html(request, body)
        }
        "/api/tree" => {
            let json = files_json(&root.files);
            respond_json(request, json)
        }
        _ => {
            let resp = Response::from_string("not found").with_status_code(StatusCode(404));
            let _ = request.respond(resp);
            Ok(())
        }
    }
}

pub(crate) fn build_file_view(
    root: &ViewRoot,
    abs: &Path,
    rel: &str,
    stdin_lines: &[String],
    live: bool,
) -> Result<FileViewModel> {
    let source = fs::read_to_string(abs).with_context(|| format!("read {}", abs.display()))?;
    // Structure shows this file only — do not merge imported lib bodies into the tree.
    let (structure, input_prompts) = match crate::parse::parse_source(&source) {
        Ok(module) => (
            render_module_structure(&module, &source),
            collect_input_prompts(&module),
        ),
        Err(e) => (
            format!(
                "<div class=\"err\">parse/load error: {}</div>",
                escape(&tidy_user_error(&format!("{e:#}"), abs, rel))
            ),
            Vec::new(),
        ),
    };
    let effective = effective_stdin(&source, stdin_lines);
    // Same host caps as `marqdo run`. Soft exit + sleep clamp only.
    let mut opts = RunOptions::default();
    opts.stdin_lines = effective.clone();
    opts.sleep_limit_ms = if live { Some(30_000) } else { Some(0) };
    // Optional: sandbox to the whole view tree so sibling folders are reachable.
    opts.fs_root = Some(root.root.clone());
    let (stdout, stderr, ok, plots) = match run_file_capture(abs, &opts) {
        Ok(cap) => (
            cap.stdout,
            String::new(),
            true,
            cap.plots.into_iter().map(|p| p.svg).collect(),
        ),
        Err(e) => (
            String::new(),
            tidy_user_error(&format!("{e:#}"), abs, rel),
            false,
            Vec::new(),
        ),
    };
    Ok(FileViewModel {
        rel_path: rel.to_string(),
        source,
        structure_html: structure,
        stdout,
        stderr,
        ok,
        preset_stdin: effective.join("\n"),
        input_prompts,
        plots,
    })
}

/// Prefer view-relative paths and strip Windows `\\?\` noise in error text.
fn tidy_user_error(msg: &str, abs: &Path, rel: &str) -> String {
    use crate::diagnostics::display_path;
    let mut s = msg.to_string();
    let abs_disp = abs.display().to_string();
    let abs_clean = display_path(abs);
    for candidate in [
        abs_disp.as_str(),
        abs_clean.as_str(),
        &abs_disp.replace('/', "\\"),
        &abs_clean.replace('/', "\\"),
        &abs_disp.replace('\\', "/"),
        &abs_clean.replace('\\', "/"),
    ] {
        if !candidate.is_empty() {
            s = s.replace(candidate, rel);
        }
    }
    // Leftover extended prefixes anywhere in the message
    s = s.replace(r"\\?\", "");
    s = s.replace("//?/", "");
    s
}

fn resolve_rel(root: &ViewRoot, rel: &str) -> Result<PathBuf> {
    if rel.is_empty() {
        bail!("missing path");
    }
    let rel_path = PathBuf::from(rel.replace('\\', "/"));
    if rel_path
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        bail!("path escapes root");
    }
    if let Some(only) = &root.only_file {
        if &rel_path != only {
            bail!("file not in view scope");
        }
    } else if !root.files.iter().any(|f| f == &rel_path) {
        // also allow forward-slash normalized compare
        let norm = rel_path.to_string_lossy().replace('\\', "/");
        if !root
            .files
            .iter()
            .any(|f| f.to_string_lossy().replace('\\', "/") == norm)
        {
            bail!("unknown file: {rel}");
        }
    }
    let abs = root.root.join(&rel_path);
    let canon = abs.canonicalize().unwrap_or(abs.clone());
    let root_canon = root.root.canonicalize().unwrap_or_else(|_| root.root.clone());
    if !canon.starts_with(&root_canon) {
        bail!("path escapes root");
    }
    if !is_mq_md(&canon) {
        bail!("not a .mq.md file");
    }
    Ok(canon)
}

fn split_url(url: &str) -> (&str, &str) {
    match url.split_once('?') {
        Some((p, q)) => (p, q),
        None => (url, ""),
    }
}

fn query_param<'a>(query: &'a str, key: &str) -> Option<String> {
    for pair in query.split('&') {
        let mut kv = pair.splitn(2, '=');
        let k = kv.next()?;
        let v = kv.next().unwrap_or("");
        if k == key {
            return Some(urlencoding_decode(v));
        }
    }
    None
}

fn urlencoding_decode(s: &str) -> String {
    // Minimal decode for %XX and + (UTF-8 byte sequences, not Latin-1 chars).
    let mut out = Vec::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                let hex = &s[i + 1..i + 3];
                if let Ok(v) = u8::from_str_radix(hex, 16) {
                    out.push(v);
                    i += 3;
                } else {
                    out.push(b'%');
                    i += 1;
                }
            }
            c => {
                out.push(c);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn files_json(files: &[PathBuf]) -> String {
    let list: Vec<String> = files
        .iter()
        .map(|f| format!("\"{}\"", escape_json(&f.to_string_lossy().replace('\\', "/"))))
        .collect();
    format!("{{\"files\":[{}]}}", list.join(","))
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn api_foreign_run(root: &ViewRoot, mut request: Request) -> Result<()> {
    let mut body = Vec::new();
    std::io::Read::read_to_end(&mut request.as_reader(), &mut body)?;
    let body = String::from_utf8_lossy(&body);
    let v: serde_json::Value = serde_json::from_str(&body).unwrap_or(serde_json::Value::Null);
    let lang = v
        .get("lang")
        .and_then(|x| x.as_str())
        .unwrap_or("python");
    let source = v.get("source").and_then(|x| x.as_str()).unwrap_or("");
    let cmd = v
        .get("cmd")
        .and_then(|x| x.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let json = if source.is_empty() {
        serde_json::json!({ "ok": false, "error": "missing source" }).to_string()
    } else {
        match crate::host::foreign::run_with_cmd_override(
            &root.root,
            lang,
            source,
            cmd,
        ) {
            Ok(stdout) => serde_json::json!({ "ok": true, "stdout": stdout }).to_string(),
            Err(error) => serde_json::json!({ "ok": false, "error": error }).to_string(),
        }
    };
    respond_json(request, json)
}

fn respond_html(request: Request, body: String) -> Result<()> {
    let header = Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..])
        .map_err(|_| anyhow::anyhow!("header"))?;
    let resp = Response::from_string(body).with_header(header);
    request.respond(resp).context("respond")?;
    Ok(())
}

fn respond_json(request: Request, body: String) -> Result<()> {
    let header = Header::from_bytes(&b"Content-Type"[..], &b"application/json; charset=utf-8"[..])
        .map_err(|_| anyhow::anyhow!("header"))?;
    let resp = Response::from_string(body).with_header(header);
    request.respond(resp).context("respond")?;
    Ok(())
}

fn open_url(url: &str) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", url])
            .spawn()
            .context("open browser")?;
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(url).spawn().context("open browser")?;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open")
            .arg(url)
            .spawn()
            .context("open browser")?;
    }
    Ok(())
}
