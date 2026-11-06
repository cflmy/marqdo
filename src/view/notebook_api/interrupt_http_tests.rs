use super::*;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

struct HttpServer {
    addr: SocketAddr,
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl HttpServer {
    fn start(root: &Path) -> Self {
        let root = Arc::new(crate::view::build_root(root).unwrap());
        let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
        let addr = server.server_addr().to_ip().unwrap();
        let stop = Arc::new(AtomicBool::new(false));
        let stopping = stop.clone();
        let thread = std::thread::spawn(move || {
            let mut workers = Vec::new();
            while !stopping.load(Ordering::SeqCst) {
                if let Some(request) = server.recv_timeout(Duration::from_millis(50)).unwrap() {
                    let root = root.clone();
                    workers.push(std::thread::spawn(move || {
                        crate::view::handle(&root, request)
                    }));
                }
            }
            for worker in workers {
                let _ = worker.join();
            }
        });
        Self {
            addr,
            stop,
            thread: Some(thread),
        }
    }
}
impl Drop for HttpServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
fn post(addr: SocketAddr, endpoint: &str, body: &serde_json::Value) -> serde_json::Value {
    let bytes = body.to_string();
    let mut stream = TcpStream::connect(addr).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(20)))
        .unwrap();
    write!(stream, "POST /api/session/{endpoint} HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{bytes}", bytes.len()).unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    let (_, body) = response.split_once("\r\n\r\n").unwrap();
    serde_json::from_str(body).unwrap_or_else(|e| panic!("{endpoint}: {e}: {response}"))
}

#[test]
fn real_http_interrupt_preserves_store_stops_batch_and_replays_control_input() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let server = HttpServer::start(&root);
    let repeats: usize = std::env::var("MARQDO_CANCEL_REPEATS")
        .map(|s| s.parse().unwrap())
        .unwrap_or(1);
    assert!(repeats > 0);
    for language in ["python", "node"] {
        for mode in ["cell/run", "cell/step", "run_all"] {
            for trial in 0..repeats {
                let id = format!(
                    "{}-{}-{language}",
                    std::process::id(),
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_nanos()
                );
                let relative = format!("target/http-cancel-{id}.mq.md");
                let path = root.join(&relative);
                let marker = root.join(format!("target/http-cancel-started-{id}"));
                let suffix = root.join(format!("target/http-cancel-suffix-{id}"));
                let marker_literal = serde_json::to_string(&marker.display().to_string()).unwrap();
                let suffix_literal = serde_json::to_string(&suffix.display().to_string()).unwrap();
                let (busy, after) = if language == "python" {
                    (format!("n = 99\nopen({marker_literal}, 'w').write('started')\nwhile True: pass"),
             format!("open({suffix_literal}, 'w').write('unexpected')\nn = 100"))
                } else {
                    (format!("n = 99; require('fs').writeFileSync({marker_literal}, 'started'); while (true) {{}}"),
             format!("require('fs').writeFileSync({suffix_literal}, 'unexpected'); n = 100;"))
                };
                let source = format!("---\nimport:\n  foreign: lib/foreign.mq.md\n---\n# main\n\n`kernels` =\n| 名称 | 语言 |\n|---|---|\n| worker | {language} |\n\n`units` =\n| 单元 | 内核 |\n|---|---|\n| busy | worker |\n| after | worker |\n\n`initial` =\n| 内核 | 名称 | 值 |\n|---|---|---|\n| worker | n | 1 |\n\n`会话` = > foreign.session kernels=`kernels` cells=`units` values=`initial`\n\n`busy` =\n```{language}\n{busy}\n```\n\n`after` =\n```{language}\n{after}\n```\n");
                std::fs::write(&path, &source).unwrap();
                let body = serde_json::json!({"path": relative});
                assert_eq!(post(server.addr, "open", &body)["ok"], true);
                let addr = server.addr;
                let mut running_body = body.clone();
                if mode == "cell/run" {
                    running_body["cell"] = "busy".into();
                }
                let runner = std::thread::spawn(move || post(addr, mode, &running_body));
                let deadline = Instant::now() + Duration::from_secs(3);
                while !marker.exists() && Instant::now() < deadline {
                    std::thread::sleep(Duration::from_millis(2));
                }
                assert!(
                    marker.exists(),
                    "{language}: evaluation never reached its side effect"
                );
                let slow_open = if language == "python" && mode == "run_all" && trial == 0 {
                    let slow_relative = format!("target/http-slow-open-{id}.mq.md");
                    let slow_path = root.join(&slow_relative);
                    let wrapper = root.join(format!("target/http-slow-wrapper-{id}.py"));
                    let started = root.join(format!("target/http-slow-started-{id}"));
                    let literal = serde_json::to_string(&started.display().to_string()).unwrap();
                    std::fs::write(&wrapper, format!("import os,sys,time\nfrom pathlib import Path\nPath({literal}).write_text('opening')\ntime.sleep(2)\nos.execv(sys.executable,[sys.executable]+sys.argv[1:])\n")).unwrap();
                    let command =
                        serde_json::to_string(&format!("python3 {}", wrapper.display())).unwrap();
                    std::fs::write(&slow_path, format!("---\nimport:\n  foreign: lib/foreign.mq.md\n---\n# main\n\n`kernels` =\n| 名称 | 语言 | 命令 |\n|---|---|---|\n| slow | python | {command} |\n\n`initial` =\n| 内核 | 名称 | 值 |\n|---|---|---|\n| slow | n | 1 |\n\n`会话` = > foreign.session kernels=`kernels` values=`initial`\n")).unwrap();
                    let slow_body = serde_json::json!({"path":slow_relative});
                    let opening_body = slow_body.clone();
                    let opener = std::thread::spawn(move || post(addr, "open", &opening_body));
                    let deadline = Instant::now() + Duration::from_secs(3);
                    while !started.exists() && Instant::now() < deadline {
                        std::thread::sleep(Duration::from_millis(2));
                    }
                    assert!(started.exists(), "unrelated slow kernel never started");
                    let start = Instant::now();
                    let pending = post(addr, "interrupt", &slow_body);
                    assert_eq!(pending["ok"], false, "{pending}");
                    assert!(
                        start.elapsed() < Duration::from_secs(1),
                        "control waited for initialization"
                    );
                    Some((opener, slow_body, slow_path, wrapper, started))
                } else {
                    None
                };
                std::thread::sleep(Duration::from_millis([0, 5, 25, 50][trial % 4]));
                let removed_source = language == "node" && mode == "cell/run" && trial == 0;
                if removed_source {
                    std::fs::remove_file(&path).unwrap();
                }
                let start = Instant::now();
                let interrupted = post(server.addr, "interrupt", &body);
                let latency = start.elapsed();
                assert!(
                    latency < Duration::from_secs(1),
                    "interrupt queued behind execution: {latency:?}"
                );
                assert_eq!(interrupted["ok"], true, "{interrupted}");
                assert_eq!(interrupted["in_flight"], true, "{interrupted}");
                assert_eq!(
                    interrupted["interrupted"]["results"][0]["active"], true,
                    "{interrupted}"
                );
                let run = runner.join().unwrap();
                let completion_latency = start.elapsed();
                assert!(
                    completion_latency < Duration::from_secs(1),
                    "cancel did not finish execution promptly: {completion_latency:?}"
                );
                if removed_source {
                    std::fs::write(&path, &source).unwrap();
                }
                if let Some((opener, slow_body, slow_path, wrapper, started)) = slow_open {
                    assert_eq!(opener.join().unwrap()["ok"], true);
                    assert_eq!(post(server.addr, "close", &slow_body)["ok"], true);
                    for file in [slow_path, wrapper, started] {
                        std::fs::remove_file(file).unwrap();
                    }
                }
                let record = if mode == "run_all" {
                    assert_eq!(run["summary"]["cancelled"], true, "{run}");
                    assert_eq!(run["summary"]["ran"], 1, "{run}");
                    assert_eq!(run["summary"]["remaining"], 1, "{run}");
                    &run["summary"]["records"][0]
                } else {
                    &run["record"]
                };
                assert_eq!(record["ok"], false, "{run}");
                assert_eq!(record["timed_out"], false, "{run}");
                assert_eq!(record["alive"], true, "{run}");
                assert!(
                    !suffix.exists(),
                    "cancelled batch executed its dependent suffix"
                );
                let pull = serde_json::json!({"path":relative,"kernel":"worker","name":"n"});
                assert_eq!(post(server.addr, "state/pull", &pull)["value"], 99);
                let exported = post(server.addr, "snapshot/export", &body);
                let snapshot = &exported["snapshot"];
                assert_eq!(snapshot["version"], 5);
                let controls: Vec<_> = snapshot["actions"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|a| a.get("controls").is_some())
                    .collect();
                assert_eq!(controls.len(), 1);
                assert_eq!(
                    controls[0]["kind"],
                    match mode {
                        "cell/run" => "cell",
                        "cell/step" => "step",
                        _ => "run_all",
                    }
                );
                assert_eq!(controls[0]["controls"].as_array().unwrap().len(), 1);
                let replay = post(
                    server.addr,
                    "snapshot/replay_check",
                    &serde_json::json!({"path":relative,"snapshot":snapshot}),
                );
                assert_eq!(replay["replay_check"]["ok"], true,
                    "{language}: expected snapshot={snapshot}; actual replay={replay}");
                assert!(!suffix.exists(), "replay executed a cancelled suffix");
                let compact = post(
                    server.addr,
                    "snapshot/export",
                    &serde_json::json!({"path":relative,"compact":true}),
                );
                assert_eq!(
                    compact["snapshot"]["format"], "marqdo.history.dag",
                    "{compact}"
                );
                let replay_compact = post(
                    server.addr,
                    "snapshot/replay_check",
                    &serde_json::json!({"path":relative,"snapshot":compact["snapshot"]}),
                );
                assert_eq!(
                    replay_compact["replay_check"]["ok"], true,
                    "expected snapshot={snapshot}; actual compact replay={replay_compact}"
                );
                let mut corrupt = compact["snapshot"].clone();
                corrupt["sha256"] = "bad".into();
                let rejected = post(
                    server.addr,
                    "snapshot/replay_check",
                    &serde_json::json!({"path":relative,"snapshot":corrupt}),
                );
                assert_eq!(rejected["ok"], false, "{rejected}");
                // A malformed cancellation suffix must fail before any replay executes.
                for mutation in ["time", "receipt", "duplicate", "version"] {
                    let mut bad = snapshot.clone();
                    let index = bad["actions"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .position(|a| a.get("controls").is_some())
                        .unwrap();
                    match mutation {
                        "time" => bad["actions"][index]["controls"][0]["after_us"] = (-1).into(),
                        "receipt" => {
                            bad["actions"][index]["controls"][0]["results"][0]["accepted"] =
                                false.into()
                        }
                        "duplicate" => {
                            let row = bad["actions"][index]["controls"][0]["results"][0].clone();
                            bad["actions"][index]["controls"][0]["results"]
                                .as_array_mut()
                                .unwrap()
                                .push(row);
                        }
                        _ => bad["version"] = 4.into(),
                    }
                    let rejected = post(
                        server.addr,
                        "snapshot/replay_check",
                        &serde_json::json!({"path":relative,"snapshot":bad}),
                    );
                    assert_eq!(rejected["ok"], false, "{mutation}: {rejected}");
                }
                assert_eq!(
                    post(server.addr, "state/pull", &pull)["value"],
                    99,
                    "checker disturbed warm state"
                );
                assert_eq!(post(server.addr, "close", &body)["ok"], true);
                println!("E11 {language} {mode} trial {trial}: real HTTP interrupt {:?}, retained n=99, skipped suffix, control-input replay and four malformed histories verified", latency);
                println!(
                    "CONTROL_METRIC:{}",
                    serde_json::json!({
                        "language": language, "mode": mode, "trial": trial,
                        "ack_ns": latency.as_nanos(), "completion_ns": completion_latency.as_nanos(),
                        "expected_store": 99, "plain_replay": true, "compact_replay": true,
                        "malformed_histories": 5, "correct": true
                    })
                );
                std::fs::remove_file(path).unwrap();
                std::fs::remove_file(marker).unwrap();
            }
        }
    }
}
