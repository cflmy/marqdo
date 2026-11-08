//! Execute the handbook itself, then serve the actual decomposition projects.
use serde_json::Value;
use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::{Duration, Instant},
};
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_marqdo")
}
struct Scratch(PathBuf);
impl Scratch {
    fn new(label: &str) -> Self {
        let p =
            std::env::temp_dir().join(format!("mq-web-handbook-{label}-{}", std::process::id()));
        fs::create_dir_all(&p).unwrap();
        Self(p)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn command(session: &Path) -> Command {
    let mut c = Command::new(bin());
    c.current_dir(root())
        .env("MARQDO_EXT", root().join("ext"))
        .env("MARQDO_LIB", root().join("lib"))
        .env(
            "MARQDO_WEB_PLUGIN",
            root()
                .join("plugins/web/build")
                .join(if cfg!(target_os = "macos") {
                    "libweb.dylib"
                } else if cfg!(windows) {
                    "web.dll"
                } else {
                    "libweb.so"
                }),
        )
        .env("MARQDO_CALL_SESSION_DIR", session);
    c
}
fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            files(&p, out)
        } else if p.to_string_lossy().ends_with(".mq.md") {
            out.push(p)
        }
    }
}
fn fences(src: &str, lang: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current = None;
    for l in src.lines() {
        if l == format!("```{lang}") {
            current = Some(String::new())
        } else if l == "```" {
            if let Some(b) = current.take() {
                out.push(b)
            }
        } else if let Some(b) = current.as_mut() {
            b.push_str(l);
            b.push('\n')
        }
    }
    assert!(current.is_none());
    out
}
fn html_structure(html: &str) {
    for marker in [
        "mq-row",
        "slot-footer",
        "slot-footer-brand",
        "slot-footer-links",
        "slot-footer-links-docs",
        "slot-footer-links-community",
    ] {
        assert!(html.contains(marker), "missing {marker}: {html}")
    }
    assert!(!html.contains("map["), "nested nodes were stringified");
    assert!(html.contains("<footer") && html.contains("<strong>"));
    assert!(html.contains("viewport"));
}
#[test]
fn source_handbook_examples_and_contracts_on_both_backends() {
    let scratch = Scratch::new("examples");
    let mut paths = Vec::new();
    files(&root().join("ext/web"), &mut paths);
    paths.sort();
    let mut calls = 0;
    fn contracts(fs: &[marqdo::ast::Function], path: &Path) {
        for f in fs {
            let c = f
                .contract
                .as_ref()
                .unwrap_or_else(|| panic!("{}: {} missing contract", path.display(), f.name));
            assert!(c.returns.is_some());
            assert_eq!(
                c.params.len(),
                f.params.len(),
                "{}: {}",
                path.display(),
                f.name
            );
            contracts(&f.children, path)
        }
    }
    for (i, p) in paths.iter().enumerate() {
        let source = fs::read_to_string(p).unwrap();
        let m = marqdo::load::load_module(p).unwrap();
        contracts(&m.functions, p);
        assert!(
            marqdo::check::check_path(p).unwrap().is_empty(),
            "{}",
            p.display()
        );
        assert!(
            marqdo::narrative_ratio::check_file(p).unwrap().is_empty(),
            "{}",
            p.display()
        );
        let blocks = fences(&source, "sh");
        assert!(
            !blocks.is_empty(),
            "{} needs a runnable recipe",
            p.display()
        );
        for backend in ["tree", "bytecode"] {
            let session = scratch.0.join(format!("{i}-{backend}"));
            fs::create_dir_all(&session).unwrap();
            for block in &blocks {
                for line in block.lines().filter(|l| !l.trim().is_empty()) {
                    assert!(line.starts_with("marqdo call "));
                    let invocation = line.replacen(
                        "marqdo call ",
                        &format!("\"{}\" call --backend {backend} ", bin()),
                        1,
                    );
                    let env = command(&session);
                    let out = Command::new("bash")
                        .args(["--noprofile", "--norc", "-ec", &invocation])
                        .current_dir(root())
                        .envs(env.get_envs().map(|(k, v)| (k, v.unwrap())))
                        .env_remove("BASH_ENV")
                        .output()
                        .unwrap();
                    assert!(
                        out.status.success(),
                        "{} [{backend}] {line}: {}\n{}",
                        p.display(),
                        String::from_utf8_lossy(&out.stdout),
                        String::from_utf8_lossy(&out.stderr)
                    );
                    let body: Value = serde_json::from_slice(&out.stdout).unwrap();
                    assert_eq!(body["ok"], true, "{body}");
                    if line.contains(" inspect ") || line.contains(" 检视 ") {
                        let mut routes: Vec<_> = body["result"]["declared"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|v| v["path"].as_str().unwrap())
                            .collect();
                        routes.sort();
                        assert_eq!(routes, vec!["/", "/about", "/api/ping"]);
                    }
                    if line.contains("preview_nodes") || line.contains("预览节点") {
                        let h = body["result"].as_str().unwrap();
                        assert!(h.contains("mq-row") && h.contains("<h2"));
                    }
                    calls += 1;
                }
            }
            for (j, block) in fences(&source, "marqdo").iter().enumerate() {
                let file = scratch.0.join(format!("author-{i}-{j}.mq.md"));
                fs::write(&file, block).unwrap();
                let out = command(&session)
                    .args(["run", file.to_str().unwrap(), "--backend", backend])
                    .output()
                    .unwrap();
                assert!(
                    out.status.success(),
                    "{}: {}",
                    p.display(),
                    String::from_utf8_lossy(&out.stderr)
                );
                let html = String::from_utf8(out.stdout).unwrap();
                assert!(
                    html.contains("slot-footer")
                        && html.contains("slot-brand")
                        && html.contains("mq-row")
                );
            }
        }
    }
    eprintln!("Executed {calls} commands directly from bilingual Web handbooks");
}
struct Server(Child);
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
fn get(port: u16, path: &str) -> std::io::Result<String> {
    let mut stream = TcpStream::connect_timeout(
        &format!("127.0.0.1:{port}").parse().unwrap(),
        Duration::from_millis(300),
    )?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    write!(
        stream,
        "GET {path} HTTP/1.0\r\nHost: localhost\r\nConnection: close\r\n\r\n"
    )?;
    let mut body = String::new();
    stream.read_to_string(&mut body)?;
    Ok(body)
}
#[test]
fn decomposition_projects_preview_and_serve_the_same_structure() {
    let scratch = Scratch::new("sites");
    for zh in [false, true] {
        for backend in ["tree", "bytecode"] {
            let site = root().join(if zh {
                "examples/web-layout-handbook-zh"
            } else {
                "examples/web-layout-handbook"
            });
            let mut source_files = Vec::new();
            files(&site, &mut source_files);
            for p in &source_files {
                let text = fs::read_to_string(p).unwrap();
                for forbidden in ["<div", "<style", "dom.raw", "json.set", "compose_"] {
                    assert!(
                        !text.contains(forbidden),
                        "{} uses {forbidden}",
                        p.display()
                    );
                }
            }
            for leaf in [
                "nav",
                "main",
                "footer/brand",
                "footer/links/docs",
                "footer/links/community",
            ] {
                let content = site.join("components").join(leaf).join("content.md");
                assert!(content.is_file());
                let markdown = fs::read_to_string(content).unwrap();
                assert!(
                    !regex::Regex::new(r"</?[A-Za-z][^>]*>")
                        .unwrap()
                        .is_match(&markdown),
                    "a terminal leaf contains HTML"
                );
            }
            let out = command(&scratch.0)
                .args([
                    "run",
                    site.join("preview.mq.md").to_str().unwrap(),
                    "--backend",
                    backend,
                ])
                .output()
                .unwrap();
            assert!(
                out.status.success(),
                "{}",
                String::from_utf8_lossy(&out.stderr)
            );
            let preview = String::from_utf8(out.stdout).unwrap();
            html_structure(&preview);
            let port = TcpListener::bind("127.0.0.1:0")
                .unwrap()
                .local_addr()
                .unwrap()
                .port();
            let logpath = scratch.0.join(format!("server-{zh}-{backend}.log"));
            let log = fs::File::create(&logpath).unwrap();
            let mut server = Server(
                command(&scratch.0)
                    .current_dir(&site)
                    .args([
                        "run",
                        "serve.mq.md",
                        "--backend",
                        backend,
                        "--bind",
                        &format!("{}={port}", if zh { "端口" } else { "port" }),
                    ])
                    .stdout(Stdio::from(log.try_clone().unwrap()))
                    .stderr(Stdio::from(log))
                    .spawn()
                    .unwrap(),
            );
            let deadline = Instant::now() + Duration::from_secs(8);
            let response = loop {
                if let Ok(h) = get(port, "/") {
                    break h;
                }
                assert!(
                    server.0.try_wait().unwrap().is_none() && Instant::now() < deadline,
                    "server failed: {}",
                    fs::read_to_string(&logpath).unwrap()
                );
                thread::sleep(Duration::from_millis(40));
            };
            assert!(response.starts_with("HTTP/1.0 200") || response.starts_with("HTTP/1.1 200"));
            let body = response.split_once("\r\n\r\n").unwrap().1;
            html_structure(body);
            assert_eq!(body.trim(), preview.trim(), "preview and live layout drift");
            let about = get(port, "/about").unwrap();
            assert!(about.contains("200 OK") && about.contains(if zh { "关于" } else { "About" }));
            let ping = get(port, "/api/ping").unwrap();
            assert!(ping.contains("200 OK"));
            let value: Value =
                serde_json::from_str(ping.split_once("\r\n\r\n").unwrap().1).unwrap();
            assert!(value.to_string().contains("pong"));
        }
    }
}

#[test]
fn metadata_only_endpoint_has_the_same_result_on_both_backends() {
    let source = "---\nmarkup: occlusion\ntype: endpoint\npath: /api/declaration\nmethod: POST\n---\n\nHandler declaration only.\n";
    let mut results = Vec::new();
    for backend in [marqdo::Backend::Tree, marqdo::Backend::Bytecode] {
        let options = marqdo::RunOptions {
            backend,
            ..Default::default()
        };
        let output = marqdo::run_source(source, &options).unwrap();
        let json = marqdo::browser_session::value_as_json(&output.value).unwrap();
        assert_eq!(json["ok"], true);
        assert_eq!(json["value"]["kind"], "endpoint");
        assert_eq!(json["value"]["path"], "/api/declaration");
        assert_eq!(json["value"]["method"], "POST");
        results.push(json);
        assert!(marqdo::run_source("## helper\n\n*1*\n", &options).is_err());
    }
    assert_eq!(results[0], results[1]);
}
