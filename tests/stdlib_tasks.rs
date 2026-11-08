//! Exercise public documents on both runtimes; no private host-only shortcuts.
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU64, Ordering},
};
static SEQ: AtomicU64 = AtomicU64::new(0);
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "mq-stdlib-{}-{}",
            std::process::id(),
            SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&p).unwrap();
        Self(p)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn call(
    backend: &str,
    module: &str,
    function: &str,
    args: &[&str],
    effects: &str,
    dir: Option<&Path>,
) -> Value {
    let mut c = Command::new(env!("CARGO_BIN_EXE_marqdo"));
    c.current_dir(root())
        .args(["call", "--json", "--backend", backend, "--effects", effects]);
    if let Some(dir) = dir {
        c.arg("--fs-root").arg(dir);
    }
    let out = c.arg(module).arg(function).args(args).output().unwrap();
    serde_json::from_slice(&out.stdout).unwrap_or_else(|e| {
        panic!(
            "{module} {function}: {e}: {} {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
    })
}
fn ok(v: Value) -> Value {
    assert_eq!(v["ok"], true, "{v}");
    v["result"].clone()
}
#[test]
fn field_transforms_and_table_shapes_are_real_public_tools() {
    for backend in ["tree", "bytecode"] {
        let m = "lib/table.mq.md";
        assert_eq!(
            ok(call(
                backend,
                m,
                "range",
                &["start=5", "end=0", "step=-2"],
                "none",
                None
            )),
            json!([5, 3, 1])
        );
        let rows =
            "list=[{\"team\":\"a\",\"n\":3},{\"team\":\"b\",\"n\":1},{\"team\":\"a\",\"n\":2}]";
        assert_eq!(
            ok(call(
                backend,
                m,
                "pluck",
                &[rows, "key=\"n\""],
                "none",
                None
            )),
            json!([3, 1, 2])
        );
        let groups = ok(call(
            backend,
            m,
            "group_by",
            &[rows, "key=\"team\""],
            "none",
            None,
        ));
        assert_eq!(groups[0]["count"], 2);
        assert_eq!(groups[1]["key"], "b");
        assert_eq!(
            ok(call(
                backend,
                m,
                "aggregate",
                &[rows, "key=\"n\""],
                "none",
                None
            )),
            6
        );
        assert_eq!(
            ok(call(
                backend,
                m,
                "where",
                &[rows, "key=\"n\"", "value=2", "op=\"ge\""],
                "none",
                None
            ))
            .as_array()
            .unwrap()
            .len(),
            2
        );
        assert_eq!(
            ok(call(
                backend,
                m,
                "from_columns",
                &["columns={\"a\":[1,2],\"b\":[3,4]}"],
                "none",
                None
            )),
            json!([{"a":1,"b":3},{"a":2,"b":4}])
        );
        assert_eq!(
            call(
                backend,
                m,
                "from_columns",
                &["columns={\"a\":[1],\"b\":[3,4]}"],
                "none",
                None
            )["ok"],
            false
        );
        let joined = ok(call(
            backend,
            m,
            "join",
            &[
                "a=[{\"id\":1,\"name\":\"left\"},{\"id\":2,\"name\":\"only\"}]",
                "b=[{\"id\":1,\"name\":\"right\"}]",
                "key=\"id\"",
                "right_key=\"id\"",
                "mode=\"left\"",
            ],
            "none",
            None,
        ));
        assert_eq!(joined[0]["right_name"], "right");
        assert_eq!(joined[1]["right_name"], Value::Null);
        assert_eq!(
            ok(call(
                backend,
                m,
                "cast",
                &[
                    "rows=[{\"n\":\"42\",\"yes\":\"true\"}]",
                    "schema={\"n\":\"int\",\"yes\":\"bool\"}"
                ],
                "none",
                None
            )),
            json!([{"n":42,"yes":true}])
        );
        assert_eq!(
            ok(call(
                backend,
                "lib/表.mq.md",
                "分组",
                &[&rows.replace("list=", "列表="), "键=\"team\""],
                "none",
                None
            )),
            groups
        );
        assert_eq!(
            ok(call(
                backend,
                "lib/path.mq.md",
                "with_extension",
                &["path=\"a/b.txt\"", "extension=\"md\""],
                "none",
                None
            )),
            "a/b.md"
        );
    }
}
#[test]
fn file_tasks_cover_binary_chunks_atomicity_and_conflicts() {
    for backend in ["tree", "bytecode"] {
        let scratch = Scratch::new();
        let dir = Some(scratch.0.as_path());
        let m = "lib/fs.mq.md";
        fs::write(scratch.0.join("binary"), [0u8, 255, 128]).unwrap();
        let bytes = ok(call(
            backend,
            m,
            "read_bytes",
            &["path=\"binary\""],
            "fs.read",
            dir,
        ));
        assert_eq!(bytes, json!({"_type":"bytes","base64":"AP+A"}));
        let chunk = ok(call(
            backend,
            m,
            "read_chunk",
            &["path=\"binary\"", "offset=1", "size=1"],
            "fs.read",
            dir,
        ));
        assert_eq!(chunk["data"]["base64"], "/w==");
        assert_eq!(chunk["next_offset"], 2);
        assert_eq!(chunk["eof"], false);
        ok(call(
            backend,
            m,
            "write_bytes",
            &[
                "path=\"copy\"",
                "data={\"_type\":\"bytes\",\"base64\":\"AP+A\"}",
            ],
            "fs.write",
            dir,
        ));
        assert_eq!(fs::read(scratch.0.join("copy")).unwrap(), [0, 255, 128]);
        assert_eq!(
            call(backend, m, "exists", &["path=\"copy\""], "none", dir)["error"]["code"],
            "artifact.effect_denied"
        );
        ok(call(
            backend,
            m,
            "atomic_write",
            &["path=\"nested/a.txt\"", "text=\"old\""],
            "fs.write",
            dir,
        ));
        assert_eq!(
            ok(call(
                backend,
                m,
                "write_changed",
                &["path=\"nested/a.txt\"", "text=\"old\""],
                "fs.write",
                dir
            )),
            false
        );
        let conflict = ok(call(
            backend,
            m,
            "write_checked",
            &[
                "path=\"nested/a.txt\"",
                "text=\"new\"",
                "expected_sha256=\"wrong\"",
            ],
            "fs.write",
            dir,
        ));
        assert_eq!(conflict["error"]["code"], "conflict");
        assert_eq!(
            fs::read_to_string(scratch.0.join("nested/a.txt")).unwrap(),
            "old"
        );
        let result = ok(call(
            backend,
            m,
            "try_read_text",
            &["path=\"missing\""],
            "fs.read",
            dir,
        ));
        assert_eq!(result["ok"], false);
        assert_eq!(result["error"]["code"], "not_found");
        let found = ok(call(
            backend,
            m,
            "find",
            &["path=\".\"", "pattern=\"**/*.txt\""],
            "fs.read",
            dir,
        ));
        assert_eq!(found, json!(["nested/a.txt"]));
        assert_eq!(
            call(
                backend,
                m,
                "remove_file",
                &["path=\"nested\""],
                "fs.write",
                dir
            )["ok"],
            false
        );
        assert!(scratch.0.join("nested/a.txt").exists());
        let planned = ok(call(
            backend,
            m,
            "copy_tree",
            &["src=\"nested\"", "dest=\"backup\"", "dry_run=True"],
            "fs.write",
            dir,
        ));
        assert_eq!(planned, json!(["a.txt"]));
        assert!(!scratch.0.join("backup").exists());
        ok(call(
            backend,
            m,
            "copy_tree",
            &["src=\"nested\"", "dest=\"backup\""],
            "fs.write",
            dir,
        ));
        assert_eq!(
            fs::read_to_string(scratch.0.join("backup/a.txt")).unwrap(),
            "old"
        );
        assert_eq!(
            ok(call(
                backend,
                m,
                "sync_tree",
                &["src=\"nested\"", "dest=\"backup\""],
                "fs.write",
                dir
            )),
            json!([])
        );
        assert_eq!(
            call(
                backend,
                m,
                "read_text",
                &["path=\"../outside\""],
                "fs.read",
                dir
            )["ok"],
            false
        );
    }
}
#[test]
fn effects_follow_host_identity_through_aliases_and_chinese() {
    let scratch = Scratch::new();
    let library = root().join("lib/文件.mq.md");
    let source=format!("---\nmarkup: occlusion\nimport:\n  disk: {}\n---\n\n## query\n\n*> disk.存在 路径=\"a.txt\"*\n",library.display());
    let file = scratch.0.join("caller.mq.md");
    fs::write(&file, source).unwrap();
    fs::write(scratch.0.join("a.txt"), "a").unwrap();
    for backend in ["tree", "bytecode"] {
        let file = file.to_str().unwrap();
        assert_eq!(
            call(backend, file, "query", &[], "none", None)["error"]["code"],
            "artifact.effect_denied"
        );
        assert_eq!(ok(call(backend, file, "query", &[], "fs.read", None)), true);
        assert_eq!(
            call(
                backend,
                "lib/文件.mq.md",
                "存在",
                &["路径=\"a.txt\""],
                "none",
                Some(&scratch.0)
            )["error"]["code"],
            "artifact.effect_denied"
        );
    }
}
#[cfg(unix)]
#[test]
fn sandbox_rejects_symlink_escapes_even_for_new_descendants() {
    use std::os::unix::fs::symlink;
    let inside = Scratch::new();
    let outside = Scratch::new();
    symlink(&outside.0, inside.0.join("escape")).unwrap();
    for backend in ["tree", "bytecode"] {
        assert_eq!(
            call(
                backend,
                "lib/fs.mq.md",
                "write_text",
                &["path=\"escape/new/a.txt\"", "text=\"no\""],
                "fs.write",
                Some(&inside.0)
            )["ok"],
            false
        );
        assert!(!outside.0.join("new").exists());
    }
}

#[test]
fn inert_documents_preserve_surrounding_source_and_reject_ambiguity() {
    let source="---\ntitle: sample\n---\n\n# Intro\n\nkeep\n\n## Data\n\n| item | n |\n|---|---|\n| a | 2 |\n\n```mq\n> fs.remove_tree path=\".\"\n# not a heading\n```\n\n# End\n\nleave me\n";
    for backend in ["tree", "bytecode"] {
        let text = format!("text=\"{source}\"");
        let sections = ok(call(
            backend,
            "lib/document.mq.md",
            "sections",
            &[&text],
            "none",
            None,
        ));
        assert_eq!(sections.as_array().unwrap().len(), 3);
        let tables = ok(call(
            backend,
            "lib/document.mq.md",
            "tables",
            &[&text],
            "none",
            None,
        ));
        assert_eq!(tables[0]["headers"], json!(["item", "n"]));
        assert_eq!(tables[0]["rows"], json!([["a", "2"]]));
        let code = ok(call(
            backend,
            "lib/document.mq.md",
            "code_blocks",
            &[&text],
            "none",
            None,
        ));
        assert!(code[0]["text"].as_str().unwrap().contains("remove_tree"));
        let result = ok(call(
            backend,
            "lib/document.mq.md",
            "replace_section",
            &[&text, "title=\"Data\"", "body=\"replacement\""],
            "none",
            None,
        ));
        assert_eq!(result["ok"], true);
        let changed = result["value"].as_str().unwrap();
        assert!(changed.starts_with("---\ntitle: sample\n---\n\n# Intro\n\nkeep"));
        assert!(changed.ends_with("# End\n\nleave me\n"));
        assert!(changed.contains("## Data\n\nreplacement\n"));
        let replaced = ok(call(
            backend,
            "lib/document.mq.md",
            "replace_table",
            &[
                &text,
                "index=0",
                "headers=[\"item\",\"n\"]",
                "rows=[[\"b\",\"3\"]]",
            ],
            "none",
            None,
        ));
        assert_eq!(replaced["ok"], true);
        assert!(replaced["value"]
            .as_str()
            .unwrap()
            .contains("# not a heading"));
        let conflict = ok(call(
            backend,
            "lib/document.mq.md",
            "replace_section",
            &[
                &text,
                "title=\"Data\"",
                "body=\"x\"",
                "expected_sha256=\"stale\"",
            ],
            "none",
            None,
        ));
        assert_eq!(conflict["error"]["code"], "conflict");
    }
}
#[test]
fn timezone_dst_and_full_toml_have_explicit_data_boundaries() {
    for backend in ["tree", "bytecode"] {
        let gap = ok(call(
            backend,
            "lib/datetime.mq.md",
            "parse_local",
            &["text=\"2024-03-10 02:30:00\"", "zone=\"America/New_York\""],
            "none",
            None,
        ));
        assert_eq!(gap["ok"], false);
        let args = [
            "text=\"2024-11-03 01:30:00\"",
            "zone=\"America/New_York\"",
            "ambiguity=\"earliest\"",
        ];
        let early = ok(call(
            backend,
            "lib/datetime.mq.md",
            "parse_local",
            &args,
            "none",
            None,
        ));
        let late = ok(call(
            backend,
            "lib/datetime.mq.md",
            "parse_local",
            &[args[0], args[1], "ambiguity=\"latest\""],
            "none",
            None,
        ));
        assert_eq!(
            late["value"]["unix"].as_i64().unwrap() - early["value"]["unix"].as_i64().unwrap(),
            3600
        );
        let before = ok(call(
            backend,
            "lib/datetime.mq.md",
            "parse_local",
            &["text=\"2024-03-09 12:00:00\"", args[1]],
            "none",
            None,
        ));
        let dt = format!("dt={}", before["value"]);
        let after = ok(call(
            backend,
            "lib/datetime.mq.md",
            "add_calendar",
            &[&dt, "days=1"],
            "none",
            None,
        ));
        assert_eq!(
            after["value"]["unix"].as_i64().unwrap() - before["value"]["unix"].as_i64().unwrap(),
            23 * 3600
        );
        let elapsed = ok(call(
            backend,
            "lib/datetime.mq.md",
            "add",
            &[&dt, "days=1"],
            "none",
            None,
        ));
        assert!(elapsed["iso"].as_str().unwrap().contains("13:00:00-04:00"));
        let toml="text=\"[app]\ninline = { ok = true }\n[[workers]]\nname = 'one'\n[[workers]]\nname = 'two'\n\"";
        let parsed = ok(call(
            backend,
            "lib/toml.mq.md",
            "parse",
            &[toml],
            "none",
            None,
        ));
        assert_eq!(parsed["workers"].as_array().unwrap().len(), 2);
        assert_eq!(parsed["app"]["inline"]["ok"], true);
        let value = format!("value={parsed}");
        let rendered = ok(call(
            backend,
            "lib/toml.mq.md",
            "stringify",
            &[&value],
            "none",
            None,
        ));
        let text = format!("text=\"{}\"", rendered.as_str().unwrap());
        assert_eq!(
            ok(call(
                backend,
                "lib/toml.mq.md",
                "parse",
                &[&text],
                "none",
                None
            )),
            parsed
        );
    }
}
#[test]
fn bounded_file_batch_inherits_parent_permissions_and_returns_ordered_results() {
    for backend in ["tree", "bytecode"] {
        let scratch = Scratch::new();
        fs::write(scratch.0.join("one.mq.md"), "# main\n\n*1*\n").unwrap();
        fs::write(scratch.0.join("two.mq.md"), "# main\n\n*2*\n").unwrap();
        fs::write(scratch.0.join("write.mq.md"),format!("---\nimport:\n  fs: {}\n---\n# main\n\n> fs.write_text path=forbidden text=no\n*3*\n",root().join("lib/fs.mq.md").display())).unwrap();
        let results=ok(call(backend,"lib/subtask.mq.md","run_many",&["tasks=[{\"path\":\"one.mq.md\"},{\"path\":\"two.mq.md\"},{\"path\":\"write.mq.md\"}]","limit=2"],"process.exec",Some(&scratch.0)));
        assert_eq!(results[0]["value"]["value"], 1);
        assert_eq!(results[1]["value"]["value"], 2);
        assert_ne!(results[2]["value"]["code"], 0);
        assert!(!scratch.0.join("forbidden").exists());
    }
}

fn public_standard_libraries() -> Vec<PathBuf> {
    let mut files: Vec<_> = fs::read_dir(root().join("lib"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.is_file() && path.to_string_lossy().ends_with(".mq.md"))
        .collect();
    files.sort();
    files
}

#[test]
fn single_file_source_examples_execute_on_both_backends() {
    let guides = public_standard_libraries();
    assert!(guides.len() >= 60, "missing standard library sources");
    for path in guides {
        let source = fs::read_to_string(&path).unwrap();
        let mut shell = false;
        let mut commands = Vec::new();
        for line in source.lines() {
            if line == "```sh" {
                shell = true
            } else if line == "```" {
                shell = false
            } else if shell && !line.trim().is_empty() {
                commands.push(line.to_string());
            }
        }
        assert!(
            !commands.is_empty(),
            "{} lacks an executable recipe",
            path.display()
        );
        let errors = marqdo::check::check_path(&path).unwrap();
        assert!(errors.is_empty(), "{}: {errors:?}", path.display());
        for backend in ["tree", "bytecode"] {
            let scratch = Scratch::new();
            fs::create_dir(scratch.0.join("input")).unwrap();
            fs::write(scratch.0.join("input/note.txt"), "Archive example").unwrap();
            for command in &commands {
                assert!(command.starts_with("marqdo call "), "{command}");
                let command = command.replacen(
                    "marqdo call ",
                    &format!(
                        "\"{}\" call --backend {backend} ",
                        env!("CARGO_BIN_EXE_marqdo")
                    ),
                    1,
                );
                let out = Command::new("bash")
                    .args(["--noprofile", "--norc", "-ec", &command])
                    .env_remove("BASH_ENV")
                    .env("MQ_WORK", &scratch.0)
                    .current_dir(root())
                    .output()
                    .unwrap();
                assert!(
                    out.status.success(),
                    "{} [{backend}] {command}: {} {}",
                    path.display(),
                    String::from_utf8_lossy(&out.stdout),
                    String::from_utf8_lossy(&out.stderr)
                );
                let body: Value = serde_json::from_slice(&out.stdout).unwrap();
                assert_eq!(body["ok"], true, "{body}");
                if body["result"]["ok"].is_boolean() {
                    assert_eq!(
                        body["result"]["ok"], true,
                        "example returned failure: {body}"
                    );
                }
            }
        }
    }
}

#[test]
fn complete_engineering_example_runs_in_a_disposable_root() {
    for backend in ["tree", "bytecode"] {
        for (file, function) in [
            ("examples/stdlib-handbook/index.mq.md", "demo"),
            ("examples/stdlib-handbook/索引.mq.md", "演示"),
        ] {
            let scratch = Scratch::new();
            let result = ok(call(
                backend,
                file,
                function,
                &[],
                "fs.write",
                Some(&scratch.0),
            ));
            let files = result["inventory"].as_array().unwrap();
            assert_eq!(files.len(), 2);
            assert_eq!(files.iter().filter(|row| row["ok"] == true).count(), 1);
            assert_eq!(files.iter().filter(|row| row["ok"] == false).count(), 1);
            assert!(result["csv"].as_str().unwrap().contains("a,30,2"));
            assert_eq!(result["update"]["ok"], true);
            let document = result["document"].as_str().unwrap();
            assert!(document.contains("Updated through public"));
            assert!(document.contains("# Appendix\n\nKeep this paragraph."));
            assert_eq!(
                fs::read_dir(&scratch.0).unwrap().count(),
                0,
                "demo leaked its temporary tree"
            );
        }
    }
}
#[test]
fn optional_zip_roundtrip_and_expansion_preflight() {
    for backend in ["tree", "bytecode"] {
        let scratch = Scratch::new();
        let dir = Some(scratch.0.as_path());
        fs::create_dir(scratch.0.join("input")).unwrap();
        fs::write(scratch.0.join("input/data.bin"), [0u8, 255]).unwrap();
        let packed = ok(call(
            backend,
            "lib/archive.mq.md",
            "pack",
            &["src=\"input\"", "dest=\"bundle.zip\""],
            "fs.write",
            dir,
        ));
        assert_eq!(packed["ok"], true);
        let limited = ok(call(
            backend,
            "lib/archive.mq.md",
            "unpack",
            &["src=\"bundle.zip\"", "dest=\"too-large\"", "max_bytes=1"],
            "fs.write",
            dir,
        ));
        assert_eq!(limited["ok"], false);
        assert!(!scratch.0.join("too-large").exists());
        let unpacked = ok(call(
            backend,
            "lib/archive.mq.md",
            "unpack",
            &["src=\"bundle.zip\"", "dest=\"output\""],
            "fs.write",
            dir,
        ));
        assert_eq!(unpacked["ok"], true);
        assert_eq!(
            fs::read(scratch.0.join("output/data.bin")).unwrap(),
            [0, 255]
        );
    }
}
#[test]
fn http_results_keep_status_headers_and_binary_downloads() {
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
        time::Duration,
    };
    for backend in ["tree", "bytecode"] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            for i in 0..4 {
                let (mut socket, _) = listener.accept().unwrap();
                socket
                    .set_read_timeout(Some(Duration::from_secs(5)))
                    .unwrap();
                let mut buffer = [0u8; 4096];
                let _ = socket.read(&mut buffer);
                let body = if i == 2 || i == 3 {
                    vec![0u8, 255, 128]
                } else {
                    b"ready".to_vec()
                };
                let status = if i == 1 { "404 Not Found" } else { "200 OK" };
                let header=format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nX-Example: yes\r\nSet-Cookie: a=1\r\nSet-Cookie: b=2\r\nConnection: close\r\n\r\n",body.len());
                socket.write_all(header.as_bytes()).unwrap();
                socket.write_all(&body).unwrap();
            }
        });
        let url = format!("url=\"http://{addr}/sample\"");
        let response = ok(call(
            backend,
            "lib/net.mq.md",
            "get",
            &[&url],
            "network.read",
            None,
        ));
        assert_eq!(response["ok"], true);
        assert_eq!(response["value"]["status"], 200);
        assert_eq!(response["value"]["body"], "ready");
        assert_eq!(
            response["value"]["headers"]
                .as_array()
                .unwrap()
                .iter()
                .find(|row| row["name"] == "set-cookie")
                .unwrap()["values"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        let response = ok(call(
            backend,
            "lib/net.mq.md",
            "get",
            &[&url],
            "network.read",
            None,
        ));
        assert_eq!(response["ok"], true);
        assert_eq!(response["value"]["status"], 404);
        let response = ok(call(
            backend,
            "lib/net.mq.md",
            "get",
            &[&url, "response=\"bytes\""],
            "network.read",
            None,
        ));
        assert_eq!(response["value"]["body"]["base64"], "AP+A");
        let scratch = Scratch::new();
        let download = ok(call(
            backend,
            "lib/net.mq.md",
            "download",
            &[&url, "path=\"download.bin\""],
            "network.read,fs.write",
            Some(&scratch.0),
        ));
        assert_eq!(download["ok"], true);
        assert_eq!(
            fs::read(scratch.0.join("download.bin")).unwrap(),
            [0, 255, 128]
        );
        server.join().unwrap();
    }
}

#[test]
fn timeout_retains_a_handle_until_the_caller_chooses_to_join() {
    for backend in ["tree", "bytecode"] {
        let scratch = Scratch::new();
        fs::write(
            scratch.0.join("child.mq.md"),
            format!(
                "---\nimport:\n  time: {}\n---\n# main\n\n> time.sleep_ms ms=100\n*17*\n",
                root().join("lib/time.mq.md").display()
            ),
        )
        .unwrap();
        let source=format!("---\nmarkup: occlusion\nimport:\n  task: {}\n---\n\n## example\n\n**id = task.spawn path=\"child.mq.md\"**\n**early = task.wait_timeout id=`id` timeout_ms=1**\n**late = task.wait_timeout id=`id` timeout_ms=2000 cancel=True**\n`out` =\n\n| early | late |\n|---|---|\n| `early` | `late` |\n\n*out*\n",root().join("lib/subtask.mq.md").display());
        let parent = scratch.0.join("parent.mq.md");
        fs::write(&parent, source).unwrap();
        let value = ok(call(
            backend,
            parent.to_str().unwrap(),
            "example",
            &[],
            "process.exec",
            None,
        ));
        assert_eq!(value["early"]["error"]["code"], "timed_out");
        assert_eq!(value["late"]["ok"], true);
        assert_eq!(value["late"]["value"]["value"], 17);
    }
}
#[test]
fn validation_set_operations_and_batch_aggregation_share_collection_semantics() {
    for backend in ["tree", "bytecode"] {
        let m = "lib/table.mq.md";
        assert_eq!(
            ok(call(
                backend,
                m,
                "union",
                &["a=[1,2,1]", "b=[2,3]"],
                "none",
                None
            )),
            json!([1, 2, 3])
        );
        let validation = ok(call(
            backend,
            m,
            "validate",
            &[
                "rows=[{\"n\":1},{\"n\":\"bad\"},{}]",
                "schema={\"n\":\"number\"}",
            ],
            "none",
            None,
        ));
        assert_eq!(validation["error"]["code"], "validation_failed");
        assert_eq!(validation["error"]["details"].as_array().unwrap().len(), 2);
        let cast = ok(call(
            backend,
            m,
            "try_cast",
            &["rows=[{\"n\":\"bad\"}]", "schema={\"n\":\"int\"}"],
            "none",
            None,
        ));
        assert_eq!(cast["ok"], false);
        let summary = ok(call(
            backend,
            m,
            "summarize",
            &[
                "list=[{\"team\":\"a\",\"n\":1},{\"team\":\"a\",\"n\":2},{\"team\":\"b\",\"n\":4}]",
                "key=\"team\"",
                "field=\"n\"",
            ],
            "none",
            None,
        ));
        assert_eq!(
            summary,
            json!([{"key":"a","count":2,"value":3},{"key":"b","count":1,"value":4}])
        );
    }
}

#[test]
fn invalid_http_options_send_nothing_and_failed_download_keeps_original() {
    use std::{
        io::{Read, Write},
        net::TcpListener,
        thread,
        time::Duration,
    };
    for backend in ["tree", "bytecode"] {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let url = format!("url=\"http://{}/\"", listener.local_addr().unwrap());
        let result = ok(call(
            backend,
            "lib/net.mq.md",
            "request",
            &[
                &url,
                "method=\"POST\"",
                "body=\"must not send\"",
                "response=\"invalid\"",
            ],
            "network.write",
            None,
        ));
        assert_eq!(result["error"]["code"], "invalid_argument");
        assert!(
            matches!(listener.accept(),Err(error) if error.kind()==std::io::ErrorKind::WouldBlock)
        );
        let scratch = Scratch::new();
        let result = ok(call(
            backend,
            "lib/net.mq.md",
            "download",
            &[&url, "path=\"../escape.bin\""],
            "network.read,fs.write",
            Some(&scratch.0),
        ));
        assert_eq!(result["error"]["code"], "permission_denied");
        assert!(
            matches!(listener.accept(),Err(error) if error.kind()==std::io::ErrorKind::WouldBlock)
        );
        listener.set_nonblocking(false).unwrap();
        let server = thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut request = [0u8; 2048];
            let _ = socket.read(&mut request);
            socket
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 8\r\nConnection: close\r\n\r\n12345678",
                )
                .unwrap();
        });
        fs::write(scratch.0.join("saved.bin"), "keep").unwrap();
        let result = ok(call(
            backend,
            "lib/net.mq.md",
            "download",
            &[&url, "path=\"saved.bin\"", "max_bytes=3"],
            "network.read,fs.write",
            Some(&scratch.0),
        ));
        assert_eq!(result["error"]["code"], "resource_limit");
        assert_eq!(fs::read(scratch.0.join("saved.bin")).unwrap(), b"keep");
        assert_eq!(fs::read_dir(&scratch.0).unwrap().count(), 1);
        server.join().unwrap();
    }
}
#[test]
fn zip_preflight_rejects_escaping_and_symlink_entries() {
    use std::io::Write;
    for backend in ["tree", "bytecode"] {
        let scratch = Scratch::new();
        let file = fs::File::create(scratch.0.join("escaping.zip")).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        zip.start_file("../outside.txt", zip::write::FileOptions::default())
            .unwrap();
        zip.write_all(b"no").unwrap();
        zip.finish().unwrap();
        let failed = ok(call(
            backend,
            "lib/archive.mq.md",
            "unpack",
            &["src=\"escaping.zip\"", "dest=\"escaped\""],
            "fs.write",
            Some(&scratch.0),
        ));
        assert_eq!(failed["ok"], false);
        assert!(!scratch.0.join("escaped").exists());
        let file = fs::File::create(scratch.0.join("symlink.zip")).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        zip.add_symlink("link", "../../target", zip::write::FileOptions::default())
            .unwrap();
        zip.finish().unwrap();
        let failed = ok(call(
            backend,
            "lib/archive.mq.md",
            "unpack",
            &["src=\"symlink.zip\"", "dest=\"links\""],
            "fs.write",
            Some(&scratch.0),
        ));
        assert_eq!(failed["ok"], false);
        assert!(!scratch.0.join("links").exists());
    }
}

#[test]
fn every_standard_library_is_its_own_readable_handbook() {
    let files = public_standard_libraries();
    assert!(files.len() >= 60);
    for path in files {
        let text = fs::read_to_string(&path).unwrap();
        assert!(
            !text
                .lines()
                .any(|line| line.trim_start().starts_with("+ `")),
            "{} retains legacy parameters",
            path.display()
        );
        for line in text
            .lines()
            .filter(|line| line.contains("host_") && line.trim_start().starts_with('*'))
        {
            assert!(
                line.contains("]("),
                "{} has an opaque unlabeled instruction: {line}",
                path.display()
            );
        }
        let module = marqdo::load::load_module(&path).unwrap();
        assert!(
            !module.functions.is_empty(),
            "empty library: {}",
            path.display()
        );
        let diagnostics = marqdo::check::check_path(&path).unwrap();
        assert!(
            diagnostics.is_empty(),
            "{}: {diagnostics:?}",
            path.display()
        );
        let reports = marqdo::narrative_ratio::measure_source_units(&text, &module.functions);
        for report in &reports {
            assert!(
                report.ratio > 0.5,
                "{} {} has {:.1}% narrative",
                path.display(),
                report.name,
                report.ratio * 100.0
            );
        }
        for function in &module.functions {
            assert!(
                function.params.iter().all(|p| p.inferred),
                "{} {} retains explicit parameters",
                path.display(),
                function.name
            );
            let contract = function
                .contract
                .as_ref()
                .unwrap_or_else(|| panic!("{} {} lacks a contract", path.display(), function.name));
            assert!(
                contract.returns.is_some(),
                "{} {} lacks a return contract",
                path.display(),
                function.name
            );
            assert_eq!(
                contract.params.len(),
                function.params.len(),
                "{} {} incomplete parameter table",
                path.display(),
                function.name
            );
            let first = text
                .lines()
                .skip(function.span.line as usize)
                .find(|line| !line.trim().is_empty())
                .unwrap();
            assert!(
                first.starts_with(if function.params.is_empty() {
                    "| 返回 |"
                } else {
                    "| 参数 |"
                }),
                "{} {} must put its contract table directly below the heading",
                path.display(),
                function.name
            );
        }
    }
}

#[test]
fn large_indices_and_chunk_sizes_have_portable_meaning() {
    for backend in ["tree", "bytecode"] {
        let table = "lib/table.mq.md";
        assert_eq!(
            ok(call(
                backend,
                table,
                "at",
                &["xs=[1,2]", "i=4294967296"],
                "none",
                None
            )),
            Value::Null
        );
        assert_eq!(
            ok(call(
                backend,
                table,
                "chunk",
                &["list=[1,2]", "size=4294967296"],
                "none",
                None
            )),
            json!([[1, 2]])
        );
        assert_eq!(
            call(
                backend,
                table,
                "put",
                &["in=[1,2]", "at=[4294967296,1]", "value=9"],
                "none",
                None
            )["ok"],
            false
        );
        for operation in ["insert", "set_at"] {
            assert_eq!(
                call(
                    backend,
                    table,
                    operation,
                    &["list=[1,2]", "index=4294967296", "item=9"],
                    "none",
                    None
                )["ok"],
                false
            );
        }
        let bytes = ok(call(
            backend,
            "lib/bytes.mq.md",
            "slice",
            &[
                "data={\"_type\":\"bytes\",\"base64\":\"AP+A\"}",
                "start=4294967296",
                "end=4294967298",
            ],
            "none",
            None,
        ));
        assert_eq!(bytes["base64"], "");
    }
}

#[test]
fn library_development_guide_examples_match_their_contracts() {
    let file = "doc/library-development.mq.md";
    let source = fs::read_to_string(root().join(file)).unwrap();
    let module = marqdo::load::load_module(&root().join(file)).unwrap();
    assert_eq!(
        module
            .functions
            .iter()
            .map(|f| f.name.as_str())
            .collect::<Vec<_>>(),
        ["计算金额", "形成订单摘要", "尝试读取正文"]
    );
    assert!(marqdo::check::check_path(&root().join(file))
        .unwrap()
        .is_empty());
    for report in marqdo::narrative_ratio::measure_source_units(&source, &module.functions) {
        assert!(
            report.ratio > 0.5,
            "{} has {:.1}% narrative",
            report.name,
            report.ratio * 100.0
        );
    }
    let recipe = source
        .split("```sh\n")
        .nth(1)
        .unwrap()
        .split("\n```")
        .next()
        .unwrap();
    let commands: Vec<_> = recipe.lines().collect();
    assert_eq!(commands.len(), 3);
    for backend in ["tree", "bytecode"] {
        let mut results = Vec::new();
        for command in &commands {
            let command = command.replacen(
                "marqdo call ",
                &format!(
                    "\"{}\" call --backend {backend} ",
                    env!("CARGO_BIN_EXE_marqdo")
                ),
                1,
            );
            let output = Command::new("bash")
                .args(["--noprofile", "--norc", "-ec", &command])
                .env_remove("BASH_ENV")
                .current_dir(root())
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{backend}: {command}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            results.push(ok(serde_json::from_slice(&output.stdout).unwrap()));
        }
        assert_eq!(results[0], 36);
        assert_eq!(results[1], json!({"商品":"笔记本","数量":3,"金额":36}));
        assert_eq!(results[2]["ok"], true);
        assert_eq!(results[2]["value"], source);
        assert_eq!(
            ok(call(backend, file, "计算金额", &["单价=12"], "none", None)),
            12
        );
        let scratch = Scratch::new();
        let missing = ok(call(
            backend,
            file,
            "尝试读取正文",
            &["路径=\"missing.md\""],
            "fs.read",
            Some(&scratch.0),
        ));
        assert_eq!(missing["ok"], false);
        assert_eq!(missing["error"]["code"], "not_found");
        assert_eq!(
            call(
                backend,
                file,
                "尝试读取正文",
                &["路径=\"missing.md\""],
                "none",
                Some(&scratch.0)
            )["ok"],
            false
        );
    }
}
