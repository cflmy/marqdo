//! `marqdo view` — local HTML browser for `.mq.md` structure + output.
//! `marqdo debug` — separate debugger host (tree-walk breakpoints).

mod debug_api;
mod debug_page;
mod html;
mod output;
mod render;

pub use output::{write_static, OutputOptions};

use std::fs;
use std::io::{self, Read};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{Receiver, RecvTimeoutError};
use std::time::Duration;

use anyhow::{bail, Context, Result};
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

use crate::input_feed::{effective_stdin, extract_frontmatter_stdin, split_stdin_text};
use crate::view::debug_page::{build_debug_model, page_debug};
use crate::view::html::{escape, page_file, page_index, LinkMode};
use crate::view::render::{collect_input_prompts, render_module_structure, FileViewModel};
use crate::{run_file_capture, RunOptions};

pub struct ViewOptions {
    pub path: PathBuf,
    pub host: String,
    pub port: u16,
    pub open_browser: bool,
}

/// Options for `marqdo debug` (separate from view).
pub struct DebugOptions {
    pub path: PathBuf,
    pub host: String,
    pub port: u16,
    pub open_browser: bool,
}

#[derive(Clone)]
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

    // Per-request threads so SSE long-poll and `/api/run` do not block each other.
    for request in server.incoming_requests() {
        let root = root_info.clone();
        std::thread::spawn(move || {
            if let Err(err) = handle(&root, request) {
                eprintln!("view request error: {err:#}");
            }
        });
    }
    Ok(())
}

/// Live debugger UI (distinct from `view`). Default port usually 7430.
pub fn serve_debug(opts: DebugOptions) -> Result<()> {
    let root_info = build_root(&opts.path)?;
    let addr: SocketAddr = format!("{}:{}", opts.host, opts.port)
        .parse()
        .context("invalid host/port")?;
    let server = Server::http(addr).map_err(|e| anyhow::anyhow!("listen {addr}: {e}"))?;
    let url = format!("http://{}:{}/", opts.host, opts.port);
    eprintln!("marqdo debug: {url}");
    eprintln!("root: {}", root_info.root.display());
    eprintln!("{} .mq.md file(s). Ctrl+C to stop.", root_info.files.len());

    if opts.open_browser {
        let _ = open_url(&url);
    }

    for request in server.incoming_requests() {
        if let Err(err) = handle_debug(&root_info, request) {
            eprintln!("debug request error: {err:#}");
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
        match path_part {
            "/api/foreign-run" => return api_foreign_run(root, request),
            "/api/run" => return api_run(root, request),
            _ => {
                let resp =
                    Response::from_string("method not allowed").with_status_code(StatusCode(405));
                let _ = request.respond(resp);
                return Ok(());
            }
        }
    }

    if method != Method::Get {
        let resp = Response::from_string("method not allowed").with_status_code(StatusCode(405));
        let _ = request.respond(resp);
        return Ok(());
    }

    let (path_part, query) = split_url(&url);
    match path_part {
        "/" | "/index.html" => {
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
        "/api/events" => api_events(request),
        _ => {
            let resp = Response::from_string("not found").with_status_code(StatusCode(404));
            let _ = request.respond(resp);
            Ok(())
        }
    }
}

fn handle_debug(root: &ViewRoot, request: Request) -> Result<()> {
    let url = request.url().to_string();
    let method = request.method().clone();

    if method == Method::Post {
        let (path_part, _) = split_url(&url);
        match path_part {
            "/api/foreign-run" => return api_foreign_run(root, request),
            "/api/debug/start" => {
                return api_debug_post(request, |body| {
                    debug_api::api_debug_start(&root.root, body)
                })
            }
            "/api/debug/continue" => {
                return api_debug_post(request, |body| {
                    debug_api::api_debug_action(body, crate::debug::DebugAction::Continue)
                })
            }
            "/api/debug/step" => {
                return api_debug_post(request, |body| {
                    debug_api::api_debug_action(body, crate::debug::DebugAction::Step)
                })
            }
            "/api/debug/stop" => {
                return api_debug_post(request, |body| debug_api::api_debug_stop(body))
            }
            "/api/debug/breakpoints" => {
                return api_debug_post(request, |body| {
                    debug_api::api_debug_set_breakpoints(body)
                })
            }
            _ => {
                let resp =
                    Response::from_string("method not allowed").with_status_code(StatusCode(405));
                let _ = request.respond(resp);
                return Ok(());
            }
        }
    }

    if method != Method::Get {
        let resp = Response::from_string("method not allowed").with_status_code(StatusCode(405));
        let _ = request.respond(resp);
        return Ok(());
    }

    let (path_part, query) = split_url(&url);
    match path_part {
        "/" | "/index.html" => {
            if let Some(first) = root.files.first() {
                let rel = first.to_string_lossy().replace('\\', "/");
                let resolved = resolve_rel(root, &rel)?;
                let source = fs::read_to_string(&resolved)
                    .with_context(|| format!("read {}", resolved.display()))?;
                let vm = build_debug_model(&resolved, &rel, &source);
                let body = page_debug(&root.files, &rel, &vm);
                respond_html(request, body)
            } else {
                let body = page_index(&root.files, None, &LinkMode::Live);
                respond_html(request, body)
            }
        }
        "/file" => {
            let rel = query_param(query, "path").unwrap_or_default();
            let resolved = resolve_rel(root, &rel)?;
            let source = fs::read_to_string(&resolved)
                .with_context(|| format!("read {}", resolved.display()))?;
            let vm = build_debug_model(&resolved, &rel, &source);
            let body = page_debug(&root.files, &rel, &vm);
            respond_html(request, body)
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
    let mut source = fs::read_to_string(abs).with_context(|| format!("read {}", abs.display()))?;
    // Structure shows this file only — do not merge imported lib bodies into the tree.
    let (mut structure, mut outline, input_prompts) = match crate::parse::parse_source(&source) {
        Ok(module) => (
            render_module_structure(&module, &source),
            crate::view::render::render_function_outline(&module),
            collect_input_prompts(&module),
        ),
        Err(e) => (
            format!(
                "<div class=\"err\">parse/load error: {}</div>",
                escape(&tidy_user_error(&format!("{e:#}"), abs, rel))
            ),
            String::new(),
            Vec::new(),
        ),
    };
    let effective = effective_stdin(&source, stdin_lines);
    let awaiting_input = awaiting_preset_input(&input_prompts, stdin_lines, &source);
    // Same host caps as `marqdo run`. Soft exit + sleep clamp only.
    let mut opts = RunOptions::default();
    opts.stdin_lines = effective.clone();
    opts.sleep_limit_ms = if live { Some(30_000) } else { Some(0) };
    // Optional: sandbox to the whole view tree so sibling folders are reachable.
    opts.fs_root = Some(root.root.clone());
    let (stdout, stderr, ok, plots) = if awaiting_input {
        (String::new(), String::new(), true, Vec::new())
    } else {
        match run_file_capture(abs, &opts) {
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
        }
    };
    // Re-read after run so writeback slots / workbook edits show in Structure.
    if !awaiting_input {
        if let Ok(next) = fs::read_to_string(abs) {
            source = next;
            if let Ok(module) = crate::parse::parse_source(&source) {
                structure = render_module_structure(&module, &source);
                outline = crate::view::render::render_function_outline(&module);
            }
        }
    }
    outline.push_str(&crate::view::render::render_writeback_outline(&source));
    Ok(FileViewModel {
        rel_path: rel.to_string(),
        source,
        structure_html: structure,
        outline_html: outline,
        stdout,
        stderr,
        ok,
        preset_stdin: effective.join("\n"),
        input_prompts,
        awaiting_input,
        plots,
    })
}

/// Defer execution until the view preset-input form supplies lines (no query stdin, no FM stdin).
pub(crate) fn awaiting_preset_input(
    input_prompts: &[String],
    stdin_lines: &[String],
    source: &str,
) -> bool {
    !input_prompts.is_empty()
        && stdin_lines.is_empty()
        && extract_frontmatter_stdin(source).is_empty()
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

fn api_debug_post(
    mut request: Request,
    f: impl FnOnce(&str) -> serde_json::Value,
) -> Result<()> {
    let mut body = Vec::new();
    std::io::Read::read_to_end(&mut request.as_reader(), &mut body)?;
    let body = String::from_utf8_lossy(&body);
    respond_json(request, f(&body).to_string())
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

/// SSE: long-lived `text/event-stream` of EventBus JSON maps (`data: …\n\n`).
fn api_events(request: Request) -> Result<()> {
    let rx = crate::host::event_bus::EventBus::global().subscribe();
    let body = SseBody::new(rx);
    let ct = Header::from_bytes(&b"Content-Type"[..], &b"text/event-stream; charset=utf-8"[..])
        .map_err(|_| anyhow::anyhow!("header"))?;
    let cc = Header::from_bytes(&b"Cache-Control"[..], &b"no-cache"[..])
        .map_err(|_| anyhow::anyhow!("header"))?;
    let conn = Header::from_bytes(&b"Connection"[..], &b"keep-alive"[..])
        .map_err(|_| anyhow::anyhow!("header"))?;
    let resp = Response::empty(StatusCode(200))
        .with_header(ct)
        .with_header(cc)
        .with_header(conn)
        .with_data(body, None)
        .with_chunked_threshold(0);
    request.respond(resp).context("respond sse")?;
    Ok(())
}

/// Background `run_file` for the live Stream panel; events go to the EventBus.
fn api_run(root: &ViewRoot, mut request: Request) -> Result<()> {
    let mut body = Vec::new();
    std::io::Read::read_to_end(&mut request.as_reader(), &mut body)?;
    let body = String::from_utf8_lossy(&body);
    let v: serde_json::Value = serde_json::from_str(&body).unwrap_or(serde_json::Value::Null);
    let rel = v.get("path").and_then(|x| x.as_str()).unwrap_or("").to_string();
    let stdin_raw = v.get("stdin").and_then(|x| x.as_str()).unwrap_or("");
    let stdin_lines = split_stdin_text(stdin_raw);
    if rel.is_empty() {
        return respond_json(
            request,
            serde_json::json!({ "ok": false, "error": "missing path" }).to_string(),
        );
    }
    let abs = match resolve_rel(root, &rel) {
        Ok(p) => p,
        Err(e) => {
            return respond_json(
                request,
                serde_json::json!({ "ok": false, "error": format!("{e:#}") }).to_string(),
            );
        }
    };
    let root_path = root.root.clone();
    let rel_clone = rel.clone();
    std::thread::spawn(move || {
        let start = serde_json::json!({
            "type": "run_start",
            "path": rel_clone,
        });
        crate::host::event_bus::EventBus::global().publish_json(&start.to_string());
        let source = fs::read_to_string(&abs).unwrap_or_default();
        let effective = effective_stdin(&source, &stdin_lines);
        let mut opts = RunOptions::default();
        opts.stdin_lines = effective;
        opts.sleep_limit_ms = Some(30_000);
        opts.fs_root = Some(root_path);
        match run_file_capture(&abs, &opts) {
            Ok(cap) => {
                let done = serde_json::json!({
                    "type": "done",
                    "path": rel_clone,
                    "ok": true,
                    "result": cap.stdout,
                });
                crate::host::event_bus::EventBus::global().publish_json(&done.to_string());
            }
            Err(e) => {
                let err = serde_json::json!({
                    "type": "error",
                    "path": rel_clone,
                    "message": format!("{e:#}"),
                });
                crate::host::event_bus::EventBus::global().publish_json(&err.to_string());
            }
        }
    });
    respond_json(
        request,
        serde_json::json!({ "ok": true, "started": true, "path": rel }).to_string(),
    )
}

struct SseBody {
    rx: Receiver<String>,
    buf: Vec<u8>,
    pos: usize,
}

impl SseBody {
    fn new(rx: Receiver<String>) -> Self {
        Self {
            rx,
            buf: Vec::new(),
            pos: 0,
        }
    }
}

impl Read for SseBody {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        loop {
            if self.pos < self.buf.len() {
                let n = (self.buf.len() - self.pos).min(out.len());
                out[..n].copy_from_slice(&self.buf[self.pos..self.pos + n]);
                self.pos += n;
                return Ok(n);
            }
            match self.rx.recv_timeout(Duration::from_secs(15)) {
                Ok(json) => {
                    self.buf = format!("data: {json}\n\n").into_bytes();
                    self.pos = 0;
                }
                Err(RecvTimeoutError::Timeout) => {
                    self.buf = b": ping\n\n".to_vec();
                    self.pos = 0;
                }
                Err(RecvTimeoutError::Disconnected) => return Ok(0),
            }
        }
    }
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

#[cfg(test)]
mod tests {
    use super::awaiting_preset_input;

    #[test]
    fn awaiting_input_when_no_stdin_source() {
        let src = include_str!("../../tests/keywords/input.mq.md");
        assert!(awaiting_preset_input(&["Name: ".into()], &[], src));
        assert!(!awaiting_preset_input(&["Name: ".into()], &["Ada".into()], src));
    }

    #[test]
    fn not_awaiting_when_frontmatter_stdin() {
        let src = "---\nstdin: Ada\n---\n\n# main\n\n*`n` = > input*\n";
        assert!(!awaiting_preset_input(&["p".into()], &[], src));
    }
}
