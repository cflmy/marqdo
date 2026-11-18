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
                        "call",
                        if zh { root().join("ext/web/网页.mq.md") } else { root().join("ext/web/web.mq.md") }.to_str().unwrap(),
                        if zh { "服务" } else { "serve" },
                        "--backend",
                        backend,
                        &format!("{}={}", if zh { "根" } else { "root" }, serde_json::to_string(&site).unwrap()),
                        &format!("{}={port}", if zh { "端口" } else { "port" }),
                        "--effects",
                        "plugin,fs.read,network.write",
                        "--fs-root",
                        site.to_str().unwrap(),
                        "--json",
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

fn browser_json(value: &marqdo::value::Value) -> Value {
    marqdo::browser_session::value_as_json(value).unwrap()
}
#[test]
fn spa_client_keeps_native_records_selection_history_and_text_effects() {
    let source = fs::read_to_string(root().join("examples/web-spa-app/static/client.mq.md")).unwrap();
    let (mut session, boot) = marqdo::browser_session::BrowserSession::boot(&source).unwrap();
    let boot = browser_json(&boot);
    assert_eq!(boot["render_list"]["#items"]["items"].as_array().unwrap().len(), 2);
    assert!(boot.get("navigate").is_none(), "boot must not push history: {boot}");
    let detail = browser_json(&session.call("open_item", r#"{"data_id":"1"}"#).unwrap());
    assert_eq!(detail["set_text"]["#title"], "Detail");
    assert_eq!(detail["set_text"]["#detail-body"], "Alpha");
    assert_eq!(detail["set_text"]["#log"], "/item");
    assert_eq!(detail["set_attr"]["#view-list"]["hidden"], true);
    let edit = browser_json(&session.call("go_edit", "{}").unwrap());
    assert_eq!(edit["set_value"]["#edit-title"], "Alpha");
    let saved = browser_json(&session.call("save_item", r#"{"value":"A <tag> \"quoted\""}"#).unwrap());
    assert_eq!(saved["set_text"]["#detail-body"], "A <tag> \"quoted\"");
    assert_eq!(saved["render_list"]["#items"]["items"][0]["attrs"]["data-id"], "1");
    assert!(saved.get("set_html").is_none(), "user text became HTML: {saved}");
    session.call("go_new", "{}").unwrap();
    let created = browser_json(&session.call("save_item", r#"{"value":"Gamma"}"#).unwrap());
    assert_eq!(created["render_list"]["#items"]["items"].as_array().unwrap().len(), 3);
    assert_eq!(created["render_list"]["#items"]["items"][2]["attrs"]["data-id"], "3");
    let pop = browser_json(&session.call("on_pop", r#"{"path":"/"}"#).unwrap());
    assert_eq!(pop["set_text"]["#title"], "Items");
    assert_eq!(pop["set_attr"]["#view-list"]["hidden"], false);
    assert!(pop.get("navigate").is_none(), "Back created another history entry: {pop}");
    let empty = browser_json(&session.call("save_item", r#"{"value":""}"#).unwrap());
    assert_eq!(empty["set_text"]["#log"], "title required");
}

#[test]
fn collaborative_client_keeps_the_next_local_edit_after_a_remote_snapshot() {
    let source=fs::read_to_string(root().join("examples/web-collab-doc/static/client.mq.md")).unwrap();
    let (mut session, boot)=marqdo::browser_session::BrowserSession::boot(&source).unwrap();
    assert_eq!(browser_json(&boot)["ws"]["url"],"/live/demo");
    let remote=browser_json(&session.call("on_msg",r#"{"data":"DOC\tremote"}"#).unwrap());
    assert_eq!(remote["set_value"]["#doc"],"remote");
    let local=browser_json(&session.call("on_edit",r#"{"value":"next local"}"#).unwrap());
    assert_eq!(local["ws"]["data"],"DOC\tnext local");
    let echoed=session.call("on_msg",r#"{"data":"DOC\tnext local"}"#).unwrap();
    assert_eq!(echoed,marqdo::value::Value::None);
    let empty=browser_json(&session.call("on_edit",r#"{"value":""}"#).unwrap());
    assert_eq!(empty["ws"]["data"],"DOC\t");
}

#[test]
fn callable_tenant_site_scopes_queries_and_preserves_data_across_restart() {
    let scratch = Scratch::new("tenant");
    fs::create_dir_all(scratch.0.join("db")).unwrap();
    for file in ["index.mq.md", "intro.md", "db/index.mq.md"] {
        fs::copy(root().join("examples/web-tenant-saas").join(file), scratch.0.join(file)).unwrap();
    }
    let insert = scratch.0.join("insert.mq.md");
    fs::write(&insert, "---\nmarkup: occlusion\nimport:\n  db: ext/data/db.mq.md\n---\n\n## add\n\nUse `url` for this owned database.\n\nWe **store = db.db(url=`url`)**.\n\n`row` =\n\n| title | tenant_id |\n|---|---|\n| User note | acme |\n\nReturn *[Insert a business record](> `store`.insert table=\"items\" rows=`row`)*.\n").unwrap();
    for backend in ["tree", "bytecode"] {
        let db_file = scratch.0.join(format!("{backend}.db"));
        for pass in 0..2 {
            let port = TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
            let logpath = scratch.0.join(format!("{backend}-{pass}.log"));
            let log = fs::File::create(&logpath).unwrap();
            let mut server = Server(command(&scratch.0).args([
                "call", "examples/web-tenant-saas/index.mq.md", "serve",
                "--backend", backend,
                &format!("root={}", serde_json::to_string(&scratch.0).unwrap()),
                &format!("port={port}"),
                &format!("url={}", serde_json::to_string(&format!("sqlite:{}", db_file.display())).unwrap()),
                "--effects", "plugin,fs.write,network.write",
                "--fs-root", scratch.0.to_str().unwrap(), "--json",
            ]).stdout(Stdio::from(log.try_clone().unwrap())).stderr(Stdio::from(log)).spawn().unwrap());
            let deadline = Instant::now() + Duration::from_secs(8);
            let home = loop {
                if let Ok(response) = get(port, "/") { break response; }
                assert!(server.0.try_wait().unwrap().is_none() && Instant::now() < deadline,
                    "tenant startup failed: {}", fs::read_to_string(&logpath).unwrap());
                thread::sleep(Duration::from_millis(30));
            };
            assert!(home.contains("200 OK") && home.contains("Two-tenant query demonstration"), "{home}");
            for (scope, count) in [("acme", pass + 1), ("beta", 1), ("unknown", 0)] {
                let response = get(port, &format!("/api/items?t={scope}")).unwrap();
                assert!(response.contains("200 OK"), "{response}");
                let body: Value = serde_json::from_str(response.split_once("\r\n\r\n").unwrap().1).unwrap();
                let rows = body["rows"].as_array().unwrap_or_else(|| panic!("unexpected JSON query body: {body}"));
                assert_eq!(rows.len(), count, "[{backend}] pass {pass}: {body}");
                assert!(rows.iter().all(|row| row["tenant_id"] == scope), "{body}");
            }
            let missing = get(port, "/api/items").unwrap();
            assert!(missing.contains("400 Bad Request") && missing.contains("tenant required"), "{missing}");
            server.0.kill().unwrap();
            server.0.wait().unwrap();
            if pass == 0 {
                let out = command(&scratch.0).args([
                    "call", insert.to_str().unwrap(), "add", "--backend", backend,
                    &format!("url={}", serde_json::to_string(&format!("sqlite:{}", db_file.display())).unwrap()),
                    "--effects", "plugin", "--fs-root", scratch.0.to_str().unwrap(), "--json",
                ]).output().unwrap();
                assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stdout));
            }
        }
    }
}

#[test]
fn recursive_network_site_retains_live_data_forms_tools_and_native_admin() {
    fn copy_sources(from: &Path, to: &Path) {
        fs::create_dir_all(to).unwrap();
        for entry in fs::read_dir(from).unwrap() {
            let entry=entry.unwrap(); let name=entry.file_name();
            if name.to_string_lossy().starts_with('.') || name=="data" || name=="generated" { continue; }
            let src=entry.path(); let dst=to.join(name);
            if src.is_dir() { copy_sources(&src,&dst); }
            else { fs::copy(src,dst).unwrap(); }
        }
    }
    fn request(port:u16,method:&str,path:&str,form:&str,cookie:&str)->String {
        let mut stream=TcpStream::connect(("127.0.0.1",port)).unwrap();
        stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
        write!(stream,"{method} {path} HTTP/1.0\r\nHost: localhost\r\nConnection: close\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\nCookie: {cookie}\r\n\r\n{form}",form.len()).unwrap();
        let mut out=String::new();stream.read_to_string(&mut out).unwrap();out
    }
    fn cookie(response:&str)->String {
        response.lines().find_map(|line|line.split_once(':')
            .filter(|(key,_)|key.eq_ignore_ascii_case("set-cookie"))
            .map(|(_,value)|value.trim().split(';').next().unwrap().to_string())).unwrap()
    }
    for (site, welcome, about_copy, required_copy, bytes_copy) in [
        ("web-net-site", "Welcome to the web-net site", "Each responsibility", "Title is required", "&lt;bytes&gt;"),
        ("web-net-site-zh", "网络扩展站", "各个职责", "标题不能为空", "&lt;字节&gt;"),
    ] {
    for backend in ["tree","bytecode"] {
        let scratch=Scratch::new(&format!("network-{site}-{backend}"));
        copy_sources(&root().join("examples").join(site),&scratch.0);
        fs::create_dir_all(scratch.0.join("public/generated")).unwrap();
        // HTTP behavior does not execute these browser assets. The real-browser
        // suite separately uses an actual current WASM build.
        for name in ["marqdo_wasm.wasm","marqdo-bridge.js"] {
            fs::write(scratch.0.join("public/generated").join(name),"HTTP-only fixture").unwrap();
        }
        let port=TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
        let logpath=scratch.0.join("server.log");
        let log=fs::File::create(&logpath).unwrap();
        let mut server=Server(command(&scratch.0).args([
            "call",&format!("examples/{site}/index.mq.md"),"serve","--backend",backend,
            &format!("root={}",serde_json::to_string(&scratch.0).unwrap()),
            &format!("port={port}"),"--effects","plugin,fs.write,network.write",
            "--fs-root",scratch.0.to_str().unwrap(),"--json",
        ]).stdout(Stdio::from(log.try_clone().unwrap())).stderr(Stdio::from(log)).spawn().unwrap());
        let deadline=Instant::now()+Duration::from_secs(10);
        let home=loop {
            if let Ok(body)=get(port,"/") { break body; }
            assert!(server.0.try_wait().unwrap().is_none() && Instant::now()<deadline,
                "network startup failed [{backend}]: {}",fs::read_to_string(&logpath).unwrap());
            thread::sleep(Duration::from_millis(30));
        };
        for expected in ["200 OK",welcome,"live-send","/static/client.mq.md"] {
            assert!(home.contains(expected),"missing {expected}: {home}");
        }
        let about=get(port,"/about").unwrap();
        assert!(about.contains("200 OK") && about.contains(about_copy),"{about}");
        let tools=get(port,"/tools").unwrap();
        for expected in ["200 OK","http_only","pic.png","image/png",bytes_copy] {
            assert!(tools.contains(expected),"missing {expected}: {tools}");
        }
        let denied=get(port,"/admin").unwrap();
        assert!(denied.contains("303 See Other") && denied.contains("/admin/login"),"{denied}");
        let new=get(port,"/new").unwrap();
        let guest=cookie(&new);
        let csrf=regex::Regex::new(r#"name="_csrf" value="([^"]+)""#).unwrap()
            .captures(&new).unwrap()[1].to_string();
        let invalid=request(port,"POST","/_form/article",&format!("_csrf={csrf}&title="),&guest);
        assert!(invalid.contains("200 OK") && invalid.contains(required_copy) &&
            invalid.contains(&format!("value=\"{csrf}\"")),"{invalid}");
        let created=request(port,"POST","/_form/article",
            &format!("_csrf={csrf}&title=Created+after+startup&body=Current+body"),&guest);
        assert!(created.contains("303 See Other"),"{created}");
        let current=get(port,"/").unwrap();
        assert!(current.contains("Created after startup"),"{current}");
        let fragment=get(port,"/_part/articles").unwrap();
        assert!(fragment.contains("200 OK") && fragment.contains("Created after startup") &&
            !fragment.contains("Try the capabilities") && !fragment.contains("<!DOCTYPE"),"{fragment}");
        let form_fragment=get(port,"/new/_part/article-form").unwrap();
        assert!(form_fragment.contains("200 OK") && form_fragment.contains("name=\"_csrf\"") &&
            !form_fragment.contains("Try the capabilities"),"{form_fragment}");
        let missing=get(port,"/_part/not-declared").unwrap();
        assert!(missing.contains("404 Not Found"),"{missing}");

        let login=request(port,"POST","/admin/login",
            &format!("_csrf={csrf}&username=admin&password=secret"),&guest);
        assert!(login.contains("303 See Other"),"{login}");
        let authenticated=cookie(&login);
        let admin=request(port,"GET","/admin/","",&authenticated);
        assert!(admin.contains("200 OK") && admin.contains("articles") && admin.contains("Created after startup"),"{admin}");
    }
    }
}
#[test]
fn callable_inspection_observes_later_resources_instead_of_the_prepare_snapshot() {
    let scratch=Scratch::new("inspect-resources");
    fs::write(scratch.0.join("index.mq.md"),"---\ntype: web\nroute: /\ntitle: Inspection fixture\n---\n\n# Metadata page\n").unwrap();
    let program=scratch.0.join("inspect.mq.md");
    fs::write(&program,"---\nmarkup: occlusion\nimport:\n  web: ext/web/web.mq.md\n  socket: ext/net/websocket.mq.md\n  form: ext/data/form.mq.md\n  auth: ext/security/auth.mq.md\n---\n\n## main\n\nUse `root` for the owned site.\n\nWe **app = web.prepare(root=`root`)** and **app = socket.route(app=`app`, path=\"/live\", mode=\"echo\", require_auth=False)** and **selected = form.form(table=\"articles\")** and **app = form.mount(app=`app`, id=\"article\", form=`selected`)**.\n\n`users` =\n\n| username | password | role |\n|---|---|---|\n| fixture | do-not-return-this-password | admin |\n\nWe **app = auth.attach(app=`app`, users=`users`, login_path=\"/members/login\")**.\nReturn *web.inspect(app=`app`)*.\n").unwrap();
    for backend in ["tree","bytecode"] {
        let output=command(&scratch.0).args(["call",program.to_str().unwrap(),"main",
            "--backend",backend,&format!("root={}",serde_json::to_string(&scratch.0).unwrap()),
            "--effects","plugin,fs.read","--fs-root",scratch.0.to_str().unwrap(),"--json"])
            .output().unwrap();
        assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stdout));
        let report:Value=serde_json::from_slice(&output.stdout).unwrap();
        let value=&report["result"];
        assert_eq!(value["scope"],"registered","{report}");
        let declared=value["declared"].as_array().unwrap();
        assert!(declared.iter().any(|row|row["path"]=="/live" && row["kind"]=="websocket"),"{report}");
        assert!(declared.iter().any(|row|row["path"]=="/" && row["file"]=="index.mq.md"),"{report}");
        let system=value["system"].as_array().unwrap();
        assert!(system.iter().any(|row|row["path"]=="/members/login" && row["method"]=="POST"),"{report}");
        assert!(system.iter().any(|row|row["path"]=="/_form/{id}" && row["method"]=="POST"),"{report}");
        assert!(!output.stdout.windows("do-not-return-this-password".len()).any(|w|w==b"do-not-return-this-password"));
    }
}

#[test]
fn pure_head_resources_match_preview_and_serving_on_both_backends() {
 let scratch=Scratch::new("pure-head");
 let entry=scratch.0.join("index.mq.md");
 fs::write(&entry,"---\nmarkup: occlusion\ntype: web\nview: main\nroute: /\ntitle: Actual head fixture\nimport:\n  component: ext/web/component/index.mq.md\n  head: ext/web/head.mq.md\n  zh: ext/web/页头.mq.md\n  web: ext/web/web.mq.md\n---\n\n## main\n\nThe head is pure; body remains ordinary Markdown nodes.\n\n`meta` =\n\n| key | value |\n|---|---|\n| description | Real <literal> \"value\" |\n| og:title | Head resource |\n\n`links` =\n\n| rel | href | crossorigin |\n|---|---|---|\n| preconnect | https://fonts.example.test | anonymous |\n| stylesheet | /theme.css | |\n\nWe **resource = head.resource(meta=`meta`, links=`links`)** and **declared = component.resource(value=`resource`)** and **content = component.markdown(text=\"# Actual content\", slot=\"content\")**.\nReturn *component.merge(a=`declared`, b=`content`)*.\n\n## preview\n\nPreview requires the real native renderer but no network.\n\nWe **nodes = main()**.\nReturn *web.render(nodes=`nodes`, title=\"Actual head fixture\")*.\n").unwrap();
 for backend in ["tree","bytecode"] {
  let output=command(&scratch.0).args(["call",entry.to_str().unwrap(),"preview","--backend",backend,
   "--effects","plugin","--fs-root",scratch.0.to_str().unwrap(),"--json"]).output().unwrap();
  assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stdout));
  let value:Value=serde_json::from_slice(&output.stdout).unwrap();
  let preview=value["result"].as_str().unwrap();
  let (head,body)=preview.split_once("</head>").unwrap();
  for expected in ["Real &lt;literal&gt; &quot;value&quot;","og:title","fonts.example.test","/theme.css"] {
   assert!(head.contains(expected),"{preview}");
  }
  assert!(body.contains("Actual content") && !body.contains("fonts.example") && !body.contains("slot-head"),"{preview}");
  let port=TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
  let logpath=scratch.0.join(format!("head-{backend}.log"));
  let log=fs::File::create(&logpath).unwrap();
  let mut server=Server(command(&scratch.0).args(["call","ext/web/web.mq.md","serve","--backend",backend,
   &format!("root={}",serde_json::to_string(&scratch.0).unwrap()),&format!("port={port}"),
   "--effects","plugin,fs.read,network.write","--fs-root",scratch.0.to_str().unwrap(),"--json"])
   .stdout(Stdio::from(log.try_clone().unwrap())).stderr(Stdio::from(log)).spawn().unwrap());
  let deadline=Instant::now()+Duration::from_secs(8);
  let served=loop {
   if let Ok(response)=get(port,"/") {break response;}
   assert!(server.0.try_wait().unwrap().is_none() && Instant::now()<deadline,
    "head serving failed: {}",fs::read_to_string(&logpath).unwrap());
   thread::sleep(Duration::from_millis(30));
  };
  let html=served.split_once("\r\n\r\n").unwrap().1;
  assert_eq!(html,preview);
  let fragment=get(port,"/_part/content").unwrap();
  assert!(fragment.contains("200 OK") && fragment.contains("Actual content") &&
   !fragment.contains("fonts.example") && !fragment.contains("description"),"{fragment}");
 }
}

#[test]
fn installed_web_package_contains_the_actual_public_resource_closure() {
 let scratch=Scratch::new("install");
 let installed=scratch.0.join("installed");
 let out=command(&scratch.0).env("MARQDO_EXT",&installed).env("MARQDO_EXT_SOURCE",root().join("ext"))
  .args(["ext","add","web"]).output().unwrap();
 assert!(out.status.success(),"{} {}",String::from_utf8_lossy(&out.stdout),String::from_utf8_lossy(&out.stderr));
 let mut maintained=Vec::new();
 for directory in ["web","data","security","net"] {files(&root().join("ext").join(directory),&mut maintained);}
 for path in maintained {
  let relative=path.strip_prefix(root().join("ext")).unwrap();
  let target=installed.join(relative);
  assert!(target.is_file(),"installer omitted {}",relative.display());
  assert_eq!(fs::read(&target).unwrap(),fs::read(&path).unwrap(),"installed source drift: {}",relative.display());
 }
 for (file,function,args) in [
  ("web/head.mq.md","resource",r#"{"meta":[{"key":"description","value":"Installed"}]}"#),
  ("web/页头.mq.md","声明",r#"{"元数据":[{"key":"description","value":"已安装"}]}"#),
  ("data/表单.mq.md","引用",r#"{"标识":"article"}"#),
  ("data/后台.mq.md","启用",r#"{"应用":{}}"#),
 ] {
  let mut child=command(&scratch.0);
  child.current_dir(&scratch.0).env("MARQDO_EXT",&installed)
   .args(["call",installed.join(file).to_str().unwrap(),function,"--args-json","-","--json"])
   .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
  let mut child=child.spawn().unwrap();
  child.stdin.take().unwrap().write_all(args.as_bytes()).unwrap();
  let output=child.wait_with_output().unwrap();
  assert!(output.status.success(),"installed {file}: {} {}",String::from_utf8_lossy(&output.stdout),String::from_utf8_lossy(&output.stderr));
 }
}

#[test]
fn recursive_literary_sites_preserve_theme_detail_forms_sessions_and_custom_desk() {
 fn copy_sources(from:&Path,to:&Path) {
  fs::create_dir_all(to).unwrap();
  for entry in fs::read_dir(from).unwrap() {
   let entry=entry.unwrap();let name=entry.file_name();
   if name.to_string_lossy().starts_with('.') || name=="data" || name=="generated" {continue;}
   if entry.path().is_dir() {copy_sources(&entry.path(),&to.join(name));}
   else {fs::copy(entry.path(),to.join(name)).unwrap();}
  }
 }
 fn request(port:u16,method:&str,path:&str,body:&str,cookie:&str)->String {
  let mut stream=TcpStream::connect(("127.0.0.1",port)).unwrap();
  stream.set_read_timeout(Some(Duration::from_secs(3))).unwrap();
  write!(stream,"{method} {path} HTTP/1.0\r\nHost: localhost\r\nConnection: close\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\nCookie: {cookie}\r\n\r\n{body}",body.len()).unwrap();
  let mut response=String::new();stream.read_to_string(&mut response).unwrap();response
 }
 fn cookie(response:&str)->String {
  response.lines().find_map(|line|line.split_once(':').filter(|(key,_)|key.eq_ignore_ascii_case("set-cookie"))
   .map(|(_,value)|value.trim().split(';').next().unwrap().to_string())).unwrap()
 }
 for (site,required_copy) in [("web-site","Title is required"),("web-site-zh","标题不能为空")] {
  for backend in ["tree","bytecode"] {
   let scratch=Scratch::new(&format!("literary-{site}-{backend}"));
   copy_sources(&root().join("examples").join(site),&scratch.0);
   // Exercise the actual executable preservation tool on an owned site copy.
   let output=command(&scratch.0).args(["call",scratch.0.join("verify.mq.md").to_str().unwrap(),"theme_values",
    "--backend",backend,"--effects","plugin,fs.read","--fs-root",scratch.0.to_str().unwrap(),"--json"]).output().unwrap();
   assert!(output.status.success(),"{}",String::from_utf8_lossy(&output.stdout));
   let checked:Value=serde_json::from_slice(&output.stdout).unwrap();
   assert_eq!(checked["result"]["ok"],true,"{checked}");
   assert_eq!(checked["result"]["value"]["preserved_rows"],412);
   assert_eq!(checked["result"]["value"]["original_sha256"],"5889305c79210754b36a1ec858d57361f73a23e983f6a091568f12584fb8b8c9");
   let port=TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
   let logpath=scratch.0.join("server.log");let log=fs::File::create(&logpath).unwrap();
   let mut server=Server(command(&scratch.0).args(["call",&format!("examples/{site}/index.mq.md"),"serve","--backend",backend,
    &format!("root={}",serde_json::to_string(&scratch.0).unwrap()),&format!("port={port}"),
    "--effects","plugin,fs.write,network.write","--fs-root",scratch.0.to_str().unwrap(),"--json"])
    .stdout(Stdio::from(log.try_clone().unwrap())).stderr(Stdio::from(log)).spawn().unwrap());
   let deadline=Instant::now()+Duration::from_secs(10);
   let home=loop {
    if let Ok(body)=get(port,"/") {break body;}
    assert!(server.0.try_wait().unwrap().is_none() && Instant::now()<deadline,
     "{site} [{backend}] startup: {}",fs::read_to_string(&logpath).unwrap());
    thread::sleep(Duration::from_millis(30));
   };
   assert!(home.contains("200 OK"),"{home}");
   assert_eq!(home.matches("<article class=\"card\">").count(),8,"{home}");
   let (head,body)=home.split_once("</head>").unwrap();
   assert!(head.contains("og:type") && head.contains("fonts.googleapis.com") && head.contains("/static/generated/theme.css"),"{home}");
   assert!(!body.contains("document-head") && body.contains("site-layout has-sidebar"),"{body}");
   assert!(body.contains("href=\"/login\"") && !body.contains("href=\"/admin\""),"{body}");
   let about=get(port,"/about").unwrap();
   assert!(about.contains("200 OK") && !about.contains("property=\"og:type\""),"{about}");
   let css=get(port,"/static/generated/theme.css").unwrap();
   for expected in ["--ink: #0a0809","--gold: #d4af37","@keyframes particleDrift","(max-width: 860px)",".site-layout.has-sidebar",".main-intro h1"] {
    assert!(css.contains(expected),"missing {expected}: {css}");
   }
   let denied=get(port,"/admin").unwrap();
   assert!(denied.contains("303 See Other") && denied.contains("/login?next="),"{denied}");
   let new=get(port,"/new").unwrap();let guest=cookie(&new);
   let token=regex::Regex::new(r#"name="_csrf" value="([^"]+)""#).unwrap().captures(&new).unwrap()[1].to_string();
   let invalid=request(port,"POST","/_form/article",&format!("_csrf={token}&title=&slug="),&guest);
   assert!(invalid.contains(required_copy) && invalid.contains(&format!("value=\"{token}\"")),"{invalid}");
   let created=request(port,"POST","/_form/article",&format!("_csrf={token}&title=After+startup&slug=current&summary=Short+summary&body=%23%23+Full+body%0A%0AMarkdown+content"),&guest);
   assert!(created.contains("303 See Other"),"{created}");
   let current=get(port,"/").unwrap();
   assert_eq!(current.matches("<article class=\"card\">").count(),9,"{current}");
   assert!(current.contains("After startup") && current.contains("Short summary") && !current.contains("Full body"),"{current}");
   let detail=get(port,"/post/current?slug=wrong").unwrap();
   assert!(detail.contains("200 OK") && detail.contains("Full body</h2>") && detail.contains("Markdown content") && !detail.contains("Short summary"),"{detail}");
   let fragment=get(port,"/_part/articles").unwrap();
   assert!(fragment.contains("After startup") && !fragment.contains("<!DOCTYPE") && !fragment.contains("fonts.googleapis.com"),"{fragment}");
   let login=request(port,"POST","/login",&format!("_csrf={token}&username=admin&password=demo"),&guest);
   assert!(login.contains("303 See Other"),"{login}");let signed=cookie(&login);
   let desk=request(port,"GET","/admin","",&signed);
   assert!(desk.contains("200 OK") && desk.contains("is-desk") && desk.contains("After startup") &&
    desk.contains("action=\"/_form/article\"") && !desk.contains("<aside"),"{desk}");
   let signed_home=request(port,"GET","/","",&signed);
   assert!(signed_home.contains("href=\"/admin\"") && !signed_home.contains("href=\"/login\""),"{signed_home}");
   let logout=request(port,"GET","/admin/logout","",&signed);assert!(logout.contains("303 See Other"),"{logout}");
   let after_logout=request(port,"GET","/admin","",&signed);
   assert!(after_logout.contains("303 See Other") && after_logout.contains("/login"),"{after_logout}");
  }
 }
}

#[test]
fn ordinary_blog_is_callable_and_resolves_default_sqlite_under_its_granted_site() {
 for backend in ["tree","bytecode"] {
  let scratch=Scratch::new(&format!("blog-{backend}"));
  for directory in ["pages","api","db","public"] {fs::create_dir_all(scratch.0.join(directory)).unwrap();}
  for file in ["index.mq.md","pages/about.mq.md","pages/post.mq.md","api/ping.mq.md","api/echo.mq.md","db/posts.tables.md"] {
   fs::copy(root().join("examples/marqdo-blog").join(file),scratch.0.join(file)).unwrap();
  }
  let port=TcpListener::bind("127.0.0.1:0").unwrap().local_addr().unwrap().port();
  let logpath=scratch.0.join("server.log");let log=fs::File::create(&logpath).unwrap();
  let mut server=Server(command(&scratch.0).args(["call","examples/marqdo-blog/serve.mq.md","main",
   "--backend",backend,&format!("root={}",serde_json::to_string(&scratch.0).unwrap()),&format!("port={port}"),
   "--effects","plugin,fs.write,network.write","--fs-root",scratch.0.to_str().unwrap(),"--json"])
   .stdout(Stdio::from(log.try_clone().unwrap())).stderr(Stdio::from(log)).spawn().unwrap());
  let deadline=Instant::now()+Duration::from_secs(8);
  let home=loop {
   if let Ok(response)=get(port,"/") {break response;}
   assert!(server.0.try_wait().unwrap().is_none() && Instant::now()<deadline,"blog startup: {}",fs::read_to_string(&logpath).unwrap());
   thread::sleep(Duration::from_millis(30));
  };
  assert!(home.contains("200 OK") && home.contains("Hello Marqdo") && home.contains("Artifact Web"),"{home}");
  assert!(scratch.0.join("data/blog.db").is_file(),"default database escaped the granted root");
  let detail=get(port,"/post/hello").unwrap();
  assert!(detail.contains("200 OK") && detail.contains("executable documents"),"{detail}");
  for path in ["/about","/api/ping"] {let response=get(port,path).unwrap();assert!(response.contains("200 OK"),"{response}");}
 }
}

#[test]
fn browser_effects_keep_typed_children_duplicate_identity_and_observed_results() {
    let source = fs::read_to_string(root().join("examples/browser-app/static/app.mq.md")).unwrap();
    let (mut session, boot) = marqdo::browser_session::BrowserSession::boot(&source).unwrap();
    let boot = browser_json(&boot);
    assert!(boot.get("navigate").is_none());
    let literal = "A <tag> \"quoted\" & data\nsecond line";
    for id in ["1", "2"] {
        let added = browser_json(&session.call("add", &serde_json::json!({"value":literal}).to_string()).unwrap());
        let rows = added["render_list"]["#items"]["items"].as_array().unwrap();
        let last = rows.last().unwrap();
        assert_eq!(last["children"][0]["text"], literal);
        assert_eq!(last["children"][1]["attrs"]["data-id"], id);
        assert!(last.get("html").is_none());
    }
    let remaining = browser_json(&session.call("on_item", r#"{"data_id":"1"}"#).unwrap());
    let rows = remaining["render_list"]["#items"]["items"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["children"][1]["attrs"]["data-id"], "2");
    assert_eq!(session.call("on_item", r#"{"data_id":"missing"}"#).unwrap(), marqdo::value::Value::None);
    let third = browser_json(&session.call("add", r#"{"value":"third"}"#).unwrap());
    assert_eq!(third["render_list"]["#items"]["items"][1]["children"][1]["attrs"]["data-id"], "3");
    let about = browser_json(&session.call("go_about", "{}").unwrap());
    assert_eq!(about["navigate"]["url"], "/about");
    assert_eq!(about["set_attr"]["#panel-about"]["hidden"], false);
    let pop = browser_json(&session.call("on_pop", r#"{"path":"/net?from=history"}"#).unwrap());
    assert!(pop.get("navigate").is_none());
    assert_eq!(pop["set_text"]["#path"], "path: /net");
    assert_eq!(pop["set_attr"]["#panel-net"]["hidden"], false);
    let failed = browser_json(&session.call("on_draft", r#"{"ok":false,"found":false}"#).unwrap());
    assert_eq!(failed["set_text"]["#log"], "draft read failed");
    assert!(failed.get("set_value").is_none());
    let empty = browser_json(&session.call("on_draft", r#"{"ok":true,"found":true,"value":""}"#).unwrap());
    assert_eq!(empty["set_value"]["#draft"], "");
    let save = browser_json(&session.call("save_draft", r#"{"value":"saved draft"}"#).unwrap());
    assert_eq!(save["storage"]["then"], "on_saved");
    assert!(save.get("set_text").is_none(), "request was prematurely described as completion");
    let observed = serde_json::json!({"ok":true,"results":[
        {"ok":true,"status":200,"body":"actual-success"},
        {"ok":false,"status":503,"body":"actual-failure"}
    ]});
    let batch = browser_json(&session.call("on_fetch_all", &observed.to_string()).unwrap());
    let output = batch["set_text"]["#net-out"].as_str().unwrap();
    assert!(output.contains("actual-success") && output.contains("503") && output.contains("actual-failure"), "{output}");
    let copy = browser_json(&session.call("copy_log", "{}").unwrap());
    assert_eq!(copy["clipboard"]["text"], format!("{literal}\nthird"));
    assert_eq!(session.call("on_tick", r#"{"ok":false,"id":"tick"}"#).unwrap(), marqdo::value::Value::None);
    let tick = browser_json(&session.call("on_tick", r#"{"ok":true,"id":"tick"}"#).unwrap());
    assert_eq!(tick["set_text"]["#tick"], "1");
}
