//! MR-01 gold: dual 60s fake SSE execute workers + short reads (ADR 0009).
//! MR-02 (same file): third execute rejected while A continues cleanly.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

fn ensure_plugins() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let status = Command::new("bash")
        .arg(root.join("scripts/build-web-plugin.sh"))
        .current_dir(&root)
        .status()
        .expect("build web plugin");
    assert!(status.success(), "web plugin build failed");
    let status = Command::new("cargo")
        .args(["build", "-p", "marqdo_plugin_llm", "--locked"])
        .current_dir(&root)
        .status()
        .expect("build llm plugin");
    assert!(status.success(), "llm plugin build failed");
    let web = root.join("plugins/web/build/libweb.so");
    let llm = root.join("target/debug/libllm.so");
    assert!(web.is_file(), "missing {web:?}");
    assert!(llm.is_file(), "missing {llm:?}");
    // SAFETY: test process setup before spawning children.
    std::env::set_var("MARQDO_WEB_PLUGIN", &web);
    std::env::set_var("MARQDO_LLM_PLUGIN", &llm);
    std::env::set_var("MARQDO_EXT", root.join("ext"));
}

fn spawn_fake_sse(prefix: &'static str, secs: u64, hz: u64) -> (String, Arc<AtomicBool>, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind fake sse");
    listener.set_nonblocking(true).expect("nonblocking");
    let addr = listener.local_addr().unwrap();
    let stop = Arc::new(AtomicBool::new(false));
    let stop_bg = stop.clone();
    let handle = thread::spawn(move || {
        // Poll accept so a never-connected upstream (e.g. 429 before dial) can stop.
        let mut stream = loop {
            if stop_bg.load(Ordering::Relaxed) {
                return;
            }
            match listener.accept() {
                Ok((s, _)) => break s,
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(20));
                    continue;
                }
                Err(_) => return,
            }
        };
        stream.set_nonblocking(false).ok();
        let mut raw = Vec::new();
        let mut buf = [0u8; 4096];
        // Read request headers (+ body if Content-Length).
        loop {
            let n = stream.read(&mut buf).unwrap_or(0);
            if n == 0 {
                break;
            }
            raw.extend_from_slice(&buf[..n]);
            if let Some(pos) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
                let header = String::from_utf8_lossy(&raw[..pos]);
                let cl = header
                    .lines()
                    .find_map(|l| l.strip_prefix("Content-Length:"))
                    .and_then(|v| v.trim().parse::<usize>().ok())
                    .unwrap_or(0);
                let body_start = pos + 4;
                while raw.len() < body_start + cl {
                    let n = stream.read(&mut buf).unwrap_or(0);
                    if n == 0 {
                        break;
                    }
                    raw.extend_from_slice(&buf[..n]);
                }
                break;
            }
            if raw.len() > 1 << 20 {
                break;
            }
        }
        let _ = stream.write_all(
            b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n",
        );
        let _ = stream.flush();
        let interval = Duration::from_millis(1000 / hz.max(1));
        let deadline = Instant::now() + Duration::from_secs(secs);
        let mut i = 0u64;
        while Instant::now() < deadline && !stop_bg.load(Ordering::Relaxed) {
            let token = format!("{prefix}{i}");
            let chunk = format!(
                "data: {{\"choices\":[{{\"delta\":{{\"content\":\"{token}\"}}}}]}}\n\n"
            );
            if stream.write_all(chunk.as_bytes()).is_err() {
                break;
            }
            let _ = stream.flush();
            i += 1;
            thread::sleep(interval);
        }
        let _ = stream.write_all(b"data: [DONE]\n\n");
        let _ = stream.flush();
    });
    (format!("http://{addr}/v1"), stop, handle)
}

fn wait_http_ok(url: &str, timeout: Duration) -> bool {
    let start = Instant::now();
    while start.elapsed() < timeout {
        let out = Command::new("curl")
            .args(["-s", "-o", "/dev/null", "-w", "%{http_code}", url])
            .output();
        if let Ok(o) = out {
            if String::from_utf8_lossy(&o.stdout) == "200" {
                return true;
            }
        }
        thread::sleep(Duration::from_millis(200));
    }
    false
}

fn collect_sse_deltas(url: &str, body: &str, base_url: &str) -> (u16, String, Vec<String>) {
    let payload = body.replace("__BASE__", base_url);
    let out = Command::new("curl")
        .args([
            "-s",
            "-N",
            "--max-time",
            "180",
            "-w",
            "\n__HTTP__%{http_code}",
            "-H",
            "Content-Type: application/json",
            "-d",
            &payload,
            url,
        ])
        .output()
        .expect("curl sse");
    let text = String::from_utf8_lossy(&out.stdout);
    let (body_text, code_s) = text
        .rsplit_once("\n__HTTP__")
        .unwrap_or((text.as_ref(), "0"));
    let code: u16 = code_s.trim().parse().unwrap_or(0);
    let mut deltas = Vec::new();
    for line in body_text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("data: ") {
            if rest == "[DONE]" {
                continue;
            }
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(rest) {
                if let Some(d) = v
                    .pointer("/payload/delta")
                    .and_then(|x| x.as_str())
                    .or_else(|| v.pointer("/payload/text").and_then(|x| x.as_str()))
                {
                    deltas.push(d.to_string());
                }
            }
        }
    }
    (code, body_text.to_string(), deltas)
}

fn count_call_lib_path_workers(site_pid: u32) -> usize {
    // Walk /proc for children whose cmdline contains --call-lib-path.
    let Ok(entries) = std::fs::read_dir("/proc") else {
        return 0;
    };
    let mut n = 0usize;
    for ent in entries.flatten() {
        let name = ent.file_name();
        let pid: u32 = match name.to_str().and_then(|s| s.parse().ok()) {
            Some(p) => p,
            None => continue,
        };
        let cmdline = std::fs::read(format!("/proc/{pid}/cmdline")).unwrap_or_default();
        let cmd = String::from_utf8_lossy(&cmdline);
        if !cmd.contains("--call-lib-path") {
            continue;
        }
        // Prefer workers whose parent is the site (or grandparent via supervisor).
        let ppid = std::fs::read_to_string(format!("/proc/{pid}/stat"))
            .ok()
            .and_then(|s| {
                let rparen = s.rfind(')')?;
                let rest = s[rparen + 2..].split_whitespace().nth(1)?;
                rest.parse::<u32>().ok()
            })
            .unwrap_or(0);
        if ppid == site_pid || cmd.contains("isolated-execute-site") || cmd.contains("serve.mq.md")
        {
            n += 1;
        } else {
            // Count any call-lib-path marqdo under this test tree.
            n += 1;
        }
    }
    n
}

#[test]
fn mr01_dual_isolated_execute_and_short_reads() {
    if TcpListener::bind("127.0.0.1:0").is_err() {
        eprintln!("skip MR-01: local sockets unavailable");
        return;
    }
    ensure_plugins();

    let secs: u64 = std::env::var("MARQDO_MR01_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(60);
    let hz: u64 = 20;

    let (base_a, stop_a, join_a) = spawn_fake_sse("A", secs, hz);
    let (base_b, stop_b, join_b) = spawn_fake_sse("B", secs, hz);

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let site = root.join("tests/ext/isolated-execute-site");
    let runtime_dir = std::env::temp_dir().join(format!("mq-mr01-rt-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&runtime_dir);

    let bin = env!("CARGO_BIN_EXE_marqdo");
    let mut child: Child = Command::new(bin)
        .args(["run", "serve.mq.md"])
        .current_dir(&site)
        .env("MARQDO_EXT", root.join("ext"))
        .env("MARQDO_WEB_PLUGIN", std::env::var_os("MARQDO_WEB_PLUGIN").unwrap())
        .env("MARQDO_LLM_PLUGIN", std::env::var_os("MARQDO_LLM_PLUGIN").unwrap())
        .env("MARQDO_RUNTIME_DIR", &runtime_dir)
        .env("MARQDO_SITE_PORT", "18220")
        .env("OPENAI_API_KEY", "mr01-key")
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn site");
    let site_pid = child.id();

    let base = "http://127.0.0.1:18220";
    if !wait_http_ok(&format!("{base}/v1/models"), Duration::from_secs(30)) {
        let _ = child.kill();
        let err = child
            .stderr
            .take()
            .map(|mut s| {
                let mut t = String::new();
                let _ = s.read_to_string(&mut t);
                t
            })
            .unwrap_or_default();
        panic!("site did not start; stderr={err}");
    }

    // Capability honesty: isolated execution is default-on.
    let caps = marqdo::workbench::probe_capabilities(&serde_json::json!({}));
    assert_eq!(
        caps["features"]["runtime.isolated_execution"],
        1,
        "caps={caps}"
    );

    let chat_url = format!("{base}/v1/chat/completions");
    let body_tmpl = r#"{"model":"mr01","stream":true,"base_url":"__BASE__","messages":[{"role":"user","content":"hi"}]}"#;

    let deltas_a = Arc::new(Mutex::new(Vec::new()));
    let deltas_b = Arc::new(Mutex::new(Vec::new()));
    let code_a = Arc::new(Mutex::new(0u16));
    let code_b = Arc::new(Mutex::new(0u16));
    let raw_a = Arc::new(Mutex::new(String::new()));
    let raw_b = Arc::new(Mutex::new(String::new()));

    let da = deltas_a.clone();
    let ca = code_a.clone();
    let ra = raw_a.clone();
    let base_a_c = base_a.clone();
    let chat_a = chat_url.clone();
    let body_a = body_tmpl.to_string();
    let t_a = thread::spawn(move || {
        let (code, raw, deltas) = collect_sse_deltas(&chat_a, &body_a, &base_a_c);
        *ca.lock().unwrap() = code;
        *ra.lock().unwrap() = raw;
        *da.lock().unwrap() = deltas;
    });

    let db = deltas_b.clone();
    let cb = code_b.clone();
    let rb = raw_b.clone();
    let base_b_c = base_b.clone();
    let chat_b = chat_url.clone();
    let body_b = body_tmpl.to_string();
    let t_b = thread::spawn(move || {
        let (code, raw, deltas) = collect_sse_deltas(&chat_b, &body_b, &base_b_c);
        *cb.lock().unwrap() = code;
        *rb.lock().unwrap() = raw;
        *db.lock().unwrap() = deltas;
    });

    // Short reads during the dual stream.
    let short_ok = Arc::new(Mutex::new(0u32));
    let short_fail = Arc::new(Mutex::new(0u32));
    let short_stop = Arc::new(AtomicBool::new(false));
    let short_ok_c = short_ok.clone();
    let short_fail_c = short_fail.clone();
    let short_stop_c = short_stop.clone();
    let models_url = format!("{base}/v1/models");
    let t_short = thread::spawn(move || {
        while !short_stop_c.load(Ordering::Relaxed) {
            let out = Command::new("curl")
                .args(["-s", "-o", "/dev/null", "-w", "%{http_code}", &models_url])
                .output();
            match out {
                Ok(o) if String::from_utf8_lossy(&o.stdout) == "200" => {
                    *short_ok_c.lock().unwrap() += 1;
                }
                _ => {
                    *short_fail_c.lock().unwrap() += 1;
                }
            }
            thread::sleep(Duration::from_millis(50)); // ~20/s
        }
    });

    // Give workers time to spawn, then assert two --call-lib-path children.
    thread::sleep(Duration::from_secs(2));
    let workers = count_call_lib_path_workers(site_pid);
    assert!(
        workers >= 2,
        "expected ≥2 --call-lib-path workers, got {workers}"
    );

    // MR-02 / slot full: third execute must be rejected while A/B hold slots.
    let (base_c, stop_c, join_c) = spawn_fake_sse("C", 5, 10);
    let third = Command::new("curl")
        .args([
            "-s",
            "-o",
            "/tmp/mq-mr01-third.out",
            "-w",
            "%{http_code}",
            "-H",
            "Content-Type: application/json",
            "-d",
            &format!(
                r#"{{"model":"mr01","stream":true,"base_url":"{base_c}","messages":[{{"role":"user","content":"x"}}]}}"#
            ),
            &chat_url,
        ])
        .output()
        .expect("third curl");
    let third_code = String::from_utf8_lossy(&third.stdout).trim().to_string();
    assert!(
        third_code == "429" || third_code == "503",
        "third execute should be rejected, got {third_code}"
    );
    stop_c.store(true, Ordering::Relaxed);
    let _ = join_c.join();

    let _ = t_a.join();
    let _ = t_b.join();
    short_stop.store(true, Ordering::Relaxed);
    let _ = t_short.join();
    stop_a.store(true, Ordering::Relaxed);
    stop_b.store(true, Ordering::Relaxed);
    let _ = join_a.join();
    let _ = join_b.join();

    let _ = child.kill();
    let _ = child.wait();

    let ca = *code_a.lock().unwrap();
    let cb = *code_b.lock().unwrap();
    let da = deltas_a.lock().unwrap().clone();
    let db = deltas_b.lock().unwrap().clone();
    let ra = raw_a.lock().unwrap().clone();
    let rb = raw_b.lock().unwrap().clone();

    assert_eq!(ca, 200, "stream A http={ca} raw={ra}");
    assert_eq!(cb, 200, "stream B http={cb} raw={rb}");
    assert!(
        !ra.contains("interpreter busy") && !ra.contains("sse bridge in use"),
        "A busy: {ra}"
    );
    assert!(
        !rb.contains("interpreter busy") && !rb.contains("sse bridge in use"),
        "B busy: {rb}"
    );
    assert!(!da.is_empty(), "A produced no deltas; raw={ra}");
    assert!(!db.is_empty(), "B produced no deltas; raw={rb}");
    assert!(
        da.iter().all(|t| t.starts_with('A')),
        "A mixed foreign tokens: {da:?}"
    );
    assert!(
        db.iter().all(|t| t.starts_with('B')),
        "B mixed foreign tokens: {db:?}"
    );
    assert!(
        da.iter().all(|t| !t.starts_with('B')),
        "A saw B tokens"
    );
    assert!(
        db.iter().all(|t| !t.starts_with('A')),
        "B saw A tokens"
    );

    let sok = *short_ok.lock().unwrap();
    let sfail = *short_fail.lock().unwrap();
    assert!(sok > 10, "expected many short 200s, got ok={sok} fail={sfail}");
    assert_eq!(sfail, 0, "short reads failed: {sfail}");

    assert_eq!(
        caps["features"]["runtime.read_lane"],
        1,
        "read_lane must be on; caps={caps}"
    );

    let _ = std::fs::remove_dir_all(&runtime_dir);
}

/// Observer disconnect must not cancel the execute task (default).
#[test]
fn disconnect_keeps_running_and_replay() {
    if TcpListener::bind("127.0.0.1:0").is_err() {
        eprintln!("skip disconnect-keeps-running: local sockets unavailable");
        return;
    }
    ensure_plugins();

    let secs: u64 = 6;
    let (base_a, stop_a, join_a) = spawn_fake_sse("A", secs, 10);

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let site = root.join("tests/ext/isolated-execute-site");
    let runtime_dir = std::env::temp_dir().join(format!("mq-disc-rt-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&runtime_dir);

    let bin = env!("CARGO_BIN_EXE_marqdo");
    let mut child: Child = Command::new(bin)
        .args(["run", "serve.mq.md"])
        .current_dir(&site)
        .env("MARQDO_EXT", root.join("ext"))
        .env("MARQDO_WEB_PLUGIN", std::env::var_os("MARQDO_WEB_PLUGIN").unwrap())
        .env("MARQDO_LLM_PLUGIN", std::env::var_os("MARQDO_LLM_PLUGIN").unwrap())
        .env("MARQDO_RUNTIME_DIR", &runtime_dir)
        .env("MARQDO_SITE_PORT", "18221")
        .env("OPENAI_API_KEY", "mr01-key")
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn site");

    let base = "http://127.0.0.1:18221";
    if !wait_http_ok(&format!("{base}/v1/models"), Duration::from_secs(45)) {
        let _ = child.kill();
        let err = child
            .stderr
            .take()
            .map(|mut s| {
                let mut t = String::new();
                let _ = s.read_to_string(&mut t);
                t
            })
            .unwrap_or_default();
        panic!("site did not start for disconnect test; stderr={err}");
    }

    let chat_url = format!("{base}/v1/chat/completions");
    let body = format!(
        r#"{{"model":"mr01","stream":true,"base_url":"{base_a}","messages":[{{"role":"user","content":"hi"}}]}}"#
    );
    // Observer disconnects after ~1.2s; task must keep running.
    let out = Command::new("curl")
        .args([
            "-s",
            "-N",
            "--max-time",
            "1",
            "-H",
            "Content-Type: application/json",
            "-d",
            &body,
            &chat_url,
        ])
        .output()
        .expect("curl disconnect");
    let text = String::from_utf8_lossy(&out.stdout);
    let mut task_id = String::new();
    for line in text.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("data: ") {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(rest) {
                if v.get("type").and_then(|t| t.as_str()) == Some("run.started") {
                    if let Some(tid) = v.pointer("/payload/task_id").and_then(|x| x.as_str()) {
                        task_id = tid.to_string();
                    }
                }
            }
        }
    }
    assert!(!task_id.is_empty(), "missing task_id in early SSE; raw={text}");

    // Wait until task finishes (observer already gone).
    let mut terminal = false;
    let deadline = Instant::now() + Duration::from_secs(secs + 8);
    while Instant::now() < deadline {
        let get_url = format!("{base}/_runtime/tasks/{task_id}");
        let out = Command::new("curl")
            .args(["-s", &get_url])
            .output()
            .expect("runtime get");
        let body = String::from_utf8_lossy(&out.stdout);
        if body.contains("\"completed\"")
            || body.contains("\"succeeded\"")
            || body.contains("\"failed\"")
            || body.contains("\"cancelled\"")
        {
            assert!(
                !body.contains("\"cancelled\""),
                "disconnect must not cancel task: {body}"
            );
            terminal = true;
            break;
        }
        thread::sleep(Duration::from_millis(200));
    }
    assert!(terminal, "task {task_id} did not finish after observer disconnect");

    // Replay via events (JSON after_seq=0).
    let ev_url = format!("{base}/_runtime/tasks/{task_id}/events?after_seq=0");
    let ev = Command::new("curl")
        .args(["-s", &ev_url])
        .output()
        .expect("events");
    let ev_body = String::from_utf8_lossy(&ev.stdout);
    assert!(
        ev_body.contains('A') || ev_body.contains("delta") || ev_body.contains("event"),
        "replay empty: {ev_body}"
    );

    stop_a.store(true, Ordering::Relaxed);
    let _ = join_a.join();
    let _ = child.kill();
    let _ = child.wait();
    let _ = std::fs::remove_dir_all(&runtime_dir);
}

/// Control + short reads stay 200 while two execute streams hold slots.
#[test]
fn control_under_dual_execute() {
    if TcpListener::bind("127.0.0.1:0").is_err() {
        eprintln!("skip control-under-dual-execute: local sockets unavailable");
        return;
    }
    ensure_plugins();

    let secs: u64 = 8;
    let (base_a, stop_a, join_a) = spawn_fake_sse("A", secs, 10);
    let (base_b, stop_b, join_b) = spawn_fake_sse("B", secs, 10);

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let site = root.join("tests/ext/isolated-execute-site");
    let runtime_dir = std::env::temp_dir().join(format!("mq-ctl-rt-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&runtime_dir);

    let bin = env!("CARGO_BIN_EXE_marqdo");
    let mut child: Child = Command::new(bin)
        .args(["run", "serve.mq.md"])
        .current_dir(&site)
        .env("MARQDO_EXT", root.join("ext"))
        .env("MARQDO_WEB_PLUGIN", std::env::var_os("MARQDO_WEB_PLUGIN").unwrap())
        .env("MARQDO_LLM_PLUGIN", std::env::var_os("MARQDO_LLM_PLUGIN").unwrap())
        .env("MARQDO_RUNTIME_DIR", &runtime_dir)
        .env("MARQDO_SITE_PORT", "18222")
        .env("OPENAI_API_KEY", "mr01-key")
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn site");

    let base = "http://127.0.0.1:18222";
    if !wait_http_ok(&format!("{base}/v1/models"), Duration::from_secs(45)) {
        let _ = child.kill();
        let err = child
            .stderr
            .take()
            .map(|mut s| {
                let mut t = String::new();
                let _ = s.read_to_string(&mut t);
                t
            })
            .unwrap_or_default();
        panic!("site did not start for control test; stderr={err}");
    }

    let chat_url = format!("{base}/v1/chat/completions");
    let body_tmpl = r#"{"model":"mr01","stream":true,"base_url":"__BASE__","messages":[{"role":"user","content":"hi"}]}"#;
    let t_a = {
        let u = chat_url.clone();
        let b = body_tmpl.to_string();
        let ba = base_a.clone();
        thread::spawn(move || {
            let _ = collect_sse_deltas(&u, &b, &ba);
        })
    };
    let t_b = {
        let u = chat_url.clone();
        let b = body_tmpl.to_string();
        let bb = base_b.clone();
        thread::spawn(move || {
            let _ = collect_sse_deltas(&u, &b, &bb);
        })
    };

    thread::sleep(Duration::from_secs(1));
    let mut ok_n = 0u32;
    let mut fail_n = 0u32;
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(3) {
        for path in ["/v1/models", "/v1/ping"] {
            let url = format!("{base}{path}");
            let t0 = Instant::now();
            let out = Command::new("curl")
                .args(["-s", "-o", "/dev/null", "-w", "%{http_code}", &url])
                .output();
            let elapsed = t0.elapsed();
            match out {
                Ok(o) if String::from_utf8_lossy(&o.stdout) == "200" => {
                    assert!(
                        elapsed < Duration::from_millis(1500),
                        "{path} p95 budget; took {elapsed:?}"
                    );
                    ok_n += 1;
                }
                _ => fail_n += 1,
            }
        }
        thread::sleep(Duration::from_millis(80));
    }
    assert!(ok_n > 4, "expected control/short 200s, ok={ok_n} fail={fail_n}");
    assert_eq!(fail_n, 0, "control/short failed under dual execute: {fail_n}");

    let _ = t_a.join();
    let _ = t_b.join();
    stop_a.store(true, Ordering::Relaxed);
    stop_b.store(true, Ordering::Relaxed);
    let _ = join_a.join();
    let _ = join_b.join();
    let _ = child.kill();
    let _ = child.wait();
    let _ = std::fs::remove_dir_all(&runtime_dir);
}
