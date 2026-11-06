use crate::diagnostics::{Diagnostic, Span};
use crate::{
    bytecode::{compile_module, Vm},
    interp::Interpreter,
    load::load_module,
    value::Value,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

fn digest(text: &str) -> String {
    format!("{:x}", Sha256::digest(text.as_bytes()))
}
fn line(text: &str, marker: &str) -> u32 {
    text.lines()
        .position(|s| s.contains(marker))
        .expect("oracle marker") as u32
        + 1
}
fn root(label: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target/source-map")
        .join(format!("{label}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
fn save(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}
fn source(key: &str, path: &Path, text: &str) -> serde_json::Value {
    json!({"key":key,"path":path,"text":text,"virtual":!path.is_absolute()})
}
fn site(key: &str, marker: &str) -> serde_json::Value {
    json!({"source":key,"marker":marker})
}
fn frame(
    name: &str,
    declaration: serde_json::Value,
    call_site: Option<serde_json::Value>,
) -> serde_json::Value {
    json!({"function":name,"declaration":declaration,"call_site":call_site})
}
fn record(
    id: &str,
    backend: &str,
    diagnostic: &Diagnostic,
    sources: Vec<serde_json::Value>,
    origin: serde_json::Value,
    frames: Vec<serde_json::Value>,
) {
    if let Some(directory) = std::env::var_os("MARQDO_SOURCE_MAP_DIR") {
        let directory = PathBuf::from(directory);
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(
            directory.join(format!("{id}-{backend}.json")),
            serde_json::to_vec_pretty(
                &json!({"version":1,"id":id,"backend":backend,"sources":sources,
                "expected":{"origin":origin,"frames":frames},"diagnostic":diagnostic.to_json()}),
            )
            .unwrap(),
        )
        .unwrap();
    }
}
fn assert_site(site: &crate::diagnostics::SourceLocation, path: &Path, text: &str, expected: u32) {
    assert_eq!(
        site.source.path.as_deref().unwrap().canonicalize().unwrap(),
        path.canonicalize().unwrap()
    );
    assert_eq!(site.source.sha256.as_deref(), Some(digest(text).as_str()));
    assert_eq!(site.span, Span::new(expected, 1));
}
fn assert_origin(d: &Diagnostic, path: &Path, text: &str, expected: u32) {
    assert_eq!(
        d.path.as_deref().unwrap().canonicalize().unwrap(),
        path.canonicalize().unwrap()
    );
    assert_eq!(d.source_sha256.as_deref(), Some(digest(text).as_str()));
    assert_eq!(d.span, Span::new(expected, 1));
    assert!(d.message.contains("division by zero"), "{d}");
    assert_eq!(d.to_json()["source_sha256"], json!(digest(text)));
}

#[test]
fn external_argument_binding_uses_target_declaration_source() {
    let dir = root("external-binding");
    let entry = dir.join("entry.mq.md");
    let target = dir.join("target.mq.md");
    let leaf = dir.join("leaf.mq.md");
    let main = "# main\n*7*\n";
    let text = "---\nimport:\n  leaf: leaf.mq.md\n---\n\n\n## required\n    + `x`\n*x*\n\n## defaulted\n    + `x`=> leaf.explode\n*x*\n";
    let leaf_text = "\n\n\n## explode\n**bad = 1 / 0**\n*bad*\n";
    save(&entry, main);
    save(&target, text);
    save(&leaf, leaf_text);
    let module = load_module(&target).unwrap();
    let mut tree = Interpreter::with_capture(Some(&entry), false);
    let before = tree.diagnostic_scope_state();
    for name in ["required", "defaulted"] {
        let fun = module.functions.iter().find(|f| f.name == name).unwrap();
        let error = tree.bind_external_args(&module, fun, &[]).unwrap_err();
        let d = Diagnostic::find(&error).unwrap();
        assert_eq!(tree.diagnostic_scope_state(), before);
        let expected_frames = if name == "required" {
            assert_eq!(d.code, "call.missing_arg");
            assert_eq!(d.path.as_deref(), Some(target.as_path()));
            assert_eq!(d.source_sha256.as_deref(), Some(digest(text).as_str()));
            assert_eq!(d.span, fun.span);
            vec![frame("required", site("target", "## required"), None)]
        } else {
            assert_origin(d, &leaf, leaf_text, line(leaf_text, "**bad"));
            vec![
                frame("defaulted", site("target", "## defaulted"), None),
                frame(
                    "explode",
                    site("leaf", "## explode"),
                    Some(site("target", "+ `x`=> leaf.explode")),
                ),
            ]
        };
        record(
            &format!("external-binding-{name}"),
            "tree",
            d,
            vec![
                source("entry", &entry, main),
                source("target", &target, text),
                source("leaf", &leaf, leaf_text),
            ],
            if name == "required" {
                site("target", "## required")
            } else {
                site("leaf", "**bad")
            },
            expected_frames,
        );
        let first = d.to_json();
        let bound = tree
            .bind_external_args(&module, fun, &[("x".into(), Value::Int(7))])
            .unwrap();
        assert_eq!(bound, vec![("x".into(), Value::Int(7))]);
        assert_eq!(tree.diagnostic_scope_state(), before);
        let again = tree.bind_external_args(&module, fun, &[]).unwrap_err();
        assert_eq!(Diagnostic::find(&again).unwrap().to_json(), first);
        assert_eq!(tree.diagnostic_scope_state(), before);
    }
}

#[test]
fn imported_source_identity_frames_and_reuse_match_on_both_backends() {
    let root = root("imports");
    let mut count = 0;
    for pad in [0, 4, 17] {
        for crlf in [false, true] {
            for side in ["left", "right"] {
                let dir = root.join(format!("{pad}-{crlf}-{side}"));
                let basename = if pad == 17 {
                    "共同源.mq.md"
                } else {
                    "shared.mq.md"
                };
                let left = dir.join("left").join(basename);
                let right = dir.join("right").join(basename);
                let middle = dir.join("middle.mq.md");
                let entry = dir.join("entry.mq.md");
                let lf = |s: String| if crlf { s.replace('\n', "\r\n") } else { s };
                let leaf_a = lf(format!(
                    "源位置验证 🧪\n{}## explode\n**bad = 1 / 0**\n*bad*\n\n## good\n*7*\n",
                    "\n".repeat(pad)
                ));
                let leaf_b = if pad == 4 {
                    leaf_a.clone()
                } else {
                    lf(format!(
                        "{}## explode\n\n**bad = 1 / 0**\n*bad*\n\n## good\n*7*\n",
                        "\n".repeat(pad + 3)
                    ))
                };
                save(&left, &leaf_a);
                save(&right, &leaf_b);
                let mid=lf(format!("---\nmarkup: occlusion\nimport:\n  leaf: {side}/{basename}\n---\n## relay\n**answer = > leaf.explode**\n*answer*\n"));
                let main=lf("---\nmarkup: occlusion\nimport:\n  mid: middle.mq.md\n---\n# main\n**answer = > mid.relay**\n*answer*\n".into());
                save(&middle, &mid);
                save(&entry, &main);
                let module = load_module(&entry).unwrap();
                let (leaf_path, leaf) = if side == "left" {
                    (&left, &leaf_a)
                } else {
                    (&right, &leaf_b)
                };
                let leaf_module = &module.import_modules["mid"].import_modules["leaf"];
                let program = compile_module(Some(&entry), &module).unwrap();
                // Dependencies are private, so API calls compile that module
                // directly rather than assuming transitive re-exports.
                let good_code =
                    crate::bytecode::compile_for_call(Some(leaf_path), leaf_module).unwrap();
                let good_fid = good_code.path_index[&vec!["good".into()]];
                for backend in ["tree", "bytecode"] {
                    let host = || {
                        let mut h =
                            crate::host::HostContext::for_capture(Some(&entry), Default::default());
                        h.call_stack.push("outside-source-entry".into());
                        h.call_site_lines.push(99);
                        h
                    };
                    let mut tree = Interpreter::with_capture(Some(&entry), false).with_host(host());
                    let mut vm = Vm::with_capture(Some(&entry)).with_host(host());
                    let before_tree = tree.diagnostic_scope_state();
                    let before_vm = vm.diagnostic_scope_state();
                    let err = if backend == "tree" {
                        tree.run_module(&module)
                    } else {
                        vm.run(&program)
                    }
                    .unwrap_err();
                    let diagnostic = Diagnostic::find(&err).unwrap();
                    assert_origin(diagnostic, leaf_path, leaf, line(leaf, "**bad"));
                    assert_eq!(diagnostic.frames.len(), 3);
                    assert_eq!(
                        diagnostic
                            .frames
                            .iter()
                            .map(|f| f.function.as_str())
                            .collect::<Vec<_>>(),
                        ["main", "relay", "explode"]
                    );
                    assert!(diagnostic.frames[0].call_site.is_none());
                    assert_site(
                        diagnostic.frames[1].call_site.as_ref().unwrap(),
                        &entry,
                        &main,
                        line(&main, "**answer"),
                    );
                    assert_site(
                        diagnostic.frames[2].call_site.as_ref().unwrap(),
                        &middle,
                        &mid,
                        line(&mid, "**answer"),
                    );
                    assert_site(
                        &diagnostic.frames[2].declaration,
                        leaf_path,
                        leaf,
                        line(leaf, "## explode"),
                    );
                    assert_eq!(tree.diagnostic_scope_state(), before_tree);
                    assert_eq!(vm.diagnostic_scope_state(), before_vm);
                    record(
                        &format!("imports-{pad}-{crlf}-{side}"),
                        backend,
                        diagnostic,
                        vec![
                            source("entry", &entry, &main),
                            source("middle", &middle, &mid),
                            source("leaf", leaf_path, leaf),
                        ],
                        site("leaf", "**bad"),
                        vec![
                            frame("main", site("entry", "# main"), None),
                            frame(
                                "relay",
                                site("middle", "## relay"),
                                Some(site("entry", "**answer")),
                            ),
                            frame(
                                "explode",
                                site("leaf", "## explode"),
                                Some(site("middle", "**answer")),
                            ),
                        ],
                    );
                    let expected = diagnostic.to_json();
                    let good = if backend == "tree" {
                        tree.invoke_function(leaf_module, "good", &[])
                    } else {
                        vm.call_function(&good_code, good_fid, &[])
                    }
                    .unwrap();
                    assert_eq!(good, Value::Int(7));
                    let again = if backend == "tree" {
                        tree.run_module(&module)
                    } else {
                        vm.run(&program)
                    }
                    .unwrap_err();
                    assert_eq!(
                        Diagnostic::find(&again).unwrap().to_json(),
                        expected,
                        "stale caller leaked after reuse"
                    );
                    assert_eq!(tree.diagnostic_scope_state(), before_tree);
                    assert_eq!(vm.diagnostic_scope_state(), before_vm);
                    count += 1;
                }
            }
        }
    }
    println!(
        "SOURCE_MAP_METRIC:{}",
        json!({"kind":"imports","cases":count,"correct":true})
    );
}

#[test]
fn source_revision_is_loaded_text_not_later_disk_contents() {
    let root = root("revision");
    let lib = root.join("shared.mq.md");
    let entry = root.join("entry.mq.md");
    let first = "## explode\n**bad = 1 / 0**\n*bad*\n";
    let second = "\n\n\n\n\n## explode\n**bad = 1 / 0**\n*bad*\n";
    let main="---\nmarkup: occlusion\nimport:\n  lib: shared.mq.md\n---\n# main\n**answer = > lib.explode**\n*answer*\n";
    save(&entry, main);
    save(&lib, first);
    let old = load_module(&entry).unwrap();
    let old_code = compile_module(Some(&entry), &old).unwrap();
    save(&lib, second);
    let new = load_module(&entry).unwrap();
    let new_code = compile_module(Some(&entry), &new).unwrap();
    for (revision, module, code, text) in [
        ("old", &old, &old_code, first),
        ("new", &new, &new_code, second),
    ] {
        let mut tree = Interpreter::with_capture(Some(&entry), false);
        let mut vm = Vm::with_capture(Some(&entry));
        for (backend, error) in [
            ("tree", tree.run_module(module).unwrap_err()),
            ("bytecode", vm.run(code).unwrap_err()),
        ] {
            let d = Diagnostic::find(&error).unwrap();
            assert_origin(d, &lib, text, line(text, "**bad"));
            assert_site(
                d.frames[1].call_site.as_ref().unwrap(),
                &entry,
                main,
                line(main, "**answer"),
            );
            record(
                &format!("revision-{revision}"),
                backend,
                d,
                vec![source("entry", &entry, main), source("leaf", &lib, text)],
                site("leaf", "**bad"),
                vec![
                    frame("main", site("entry", "# main"), None),
                    frame(
                        "explode",
                        site("leaf", "## explode"),
                        Some(site("entry", "**answer")),
                    ),
                ],
            );
        }
    }
}

#[test]
#[cfg(feature = "exec-host")]
fn foreign_boundary_diagnostics_keep_library_origin_and_real_user_call_site() {
    let root = root("foreign");
    let lib = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib/foreign.mq.md");
    let library = std::fs::read_to_string(&lib).unwrap();
    let mut count = 0;
    for lang in ["python", "node"] {
        for output in [false, true] {
            let entry = root.join(format!("{lang}-{output}.mq.md"));
            let action = if output {
                "**> foreign.session_eval id=`s` source=\"n = 99; 9007199254740992\"**"
            } else {
                "**> foreign.session_set id=`s` name=\"n\" value=`s`**"
            };
            let main=format!("---\nmarkup: occlusion\nimport:\n  foreign: {}\n---\n# main\n**s = > foreign.session_open lang=\"{lang}\"**\n**> foreign.session_set id=`s` name=\"n\" value=1**\n{action}\n",lib.display());
            save(&entry, &main);
            let module = load_module(&entry).unwrap();
            let program = compile_module(Some(&entry), &module).unwrap();
            let expected_function = if output {
                "session_eval"
            } else {
                "session_set"
            };
            let expected_primitive = if output {
                "host_foreign_session_eval"
            } else {
                "host_foreign_session_set"
            };
            for backend in ["tree", "bytecode"] {
                let mut tree = Interpreter::with_capture(Some(&entry), false);
                let mut vm = Vm::with_capture(Some(&entry));
                let error = if backend == "tree" {
                    tree.run_module(&module)
                } else {
                    vm.run(&program)
                }
                .unwrap_err();
                let d = Diagnostic::find(&error).unwrap();
                assert!(d.message.contains("outside V_safe"));
                assert_eq!(
                    d.path.as_ref().unwrap().canonicalize().unwrap(),
                    lib.canonicalize().unwrap()
                );
                assert_eq!(d.source_sha256, Some(digest(&library)));
                assert_eq!(d.span.line, line(&library, expected_primitive));
                let last_frame = d.frames.last().unwrap();
                assert_eq!(last_frame.function, expected_function);
                assert_site(
                    last_frame.call_site.as_ref().unwrap(),
                    &entry,
                    &main,
                    line(&main, action),
                );
                assert!(d.format_message().contains(&format!(
                    "{}:{}:1",
                    entry.display(),
                    line(&main, action)
                )));
                let (_, values, _) = if backend == "tree" {
                    tree.foreign_source_evidence()
                } else {
                    vm.foreign_source_evidence()
                };
                assert_eq!(values, vec![Value::Int(if output { 99 } else { 1 })]);
                record(
                    &format!("foreign-{lang}-{output}"),
                    backend,
                    d,
                    vec![
                        source("entry", &entry, &main),
                        source("foreign", &lib, &library),
                    ],
                    site("foreign", expected_primitive),
                    vec![
                        frame("main", site("entry", "# main"), None),
                        frame(
                            expected_function,
                            site("foreign", &format!("## {expected_function}")),
                            Some(site("entry", action)),
                        ),
                    ],
                );
                count += 1;
            }
        }
    }
    println!(
        "SOURCE_MAP_METRIC:{}",
        json!({"kind":"foreign","cases":count,"correct":true})
    );
}

#[test]
fn memory_dynamic_and_method_calls_keep_their_own_sources() {
    let directory = root("entry-kinds");
    let memory="---\nmarkup: occlusion\nimport:\n  json: lib/json.mq.md\n---\n# main\n**bad = > json.parse text=not-json**\n";
    let module = crate::load::load_module_from_source(memory).unwrap();
    let json_text = crate::embedded_lib::read_file("json.mq.md").unwrap();
    let code = compile_module(None, &module).unwrap();
    let dynamic="# main\n**name = \"explode\"**\n> call_fn name=`name`\n\n## explode\n**bad = 1 / 0**\n*bad*\n";
    let dynamic_path = directory.join("dynamic.mq.md");
    save(&dynamic_path, dynamic);
    let dynamic_module = load_module(&dynamic_path).unwrap();
    let dynamic_code = compile_module(Some(&dynamic_path), &dynamic_module).unwrap();
    let object_path = directory.join("object.mq.md");
    let json_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib/json.mq.md");
    let object=format!("---\nmarkup: occlusion\nimport:\n  json: {}\n---\n# Counter\n**state = > json.parse text={{\"n\":1}}**\n*state*\n\n## fail\n**saved = `self`**\n**bad = 1 / 0**\n*bad*\n",json_path.display());
    save(&object_path, &object);
    let entry_path = directory.join("method.mq.md");
    let method="---\nmarkup: occlusion\nimport:\n  obj: object.mq.md\n---\n# main\n**c = > obj.Counter**\n> `c`.fail\n";
    save(&entry_path, method);
    let method_module = load_module(&entry_path).unwrap();
    let method_code = compile_module(Some(&entry_path), &method_module).unwrap();
    for backend in ["tree", "bytecode"] {
        let error = if backend == "tree" {
            Interpreter::with_capture(None, false).run_module(&module)
        } else {
            Vm::with_capture(None).run(&code)
        }
        .unwrap_err();
        let d = Diagnostic::find(&error).unwrap();
        assert_eq!(d.path.as_deref(), Some(Path::new("lib/json.mq.md")));
        assert_eq!(d.source_sha256, Some(digest(&json_text)));
        assert_eq!(d.span.line, line(&json_text, "host_json_parse"));
        assert_eq!(
            d.frames
                .last()
                .unwrap()
                .call_site
                .as_ref()
                .unwrap()
                .source
                .path
                .as_deref(),
            Some(Path::new("<memory>"))
        );
        record(
            "memory",
            backend,
            d,
            vec![
                source("entry", Path::new("<memory>"), memory),
                source("json", Path::new("lib/json.mq.md"), &json_text),
            ],
            site("json", "host_json_parse"),
            vec![
                frame("main", site("entry", "# main"), None),
                frame(
                    "parse",
                    site("json", "## parse"),
                    Some(site("entry", "**bad")),
                ),
            ],
        );
        let error = if backend == "tree" {
            Interpreter::with_capture(Some(&dynamic_path), false).run_module(&dynamic_module)
        } else {
            Vm::with_capture(Some(&dynamic_path)).run(&dynamic_code)
        }
        .unwrap_err();
        let d = Diagnostic::find(&error).unwrap();
        assert_origin(d, &dynamic_path, dynamic, line(dynamic, "**bad"));
        assert_eq!(d.frames.last().unwrap().function, "explode");
        assert_site(
            d.frames.last().unwrap().call_site.as_ref().unwrap(),
            &dynamic_path,
            dynamic,
            line(dynamic, "> call_fn"),
        );
        record(
            "dynamic",
            backend,
            d,
            vec![source("entry", &dynamic_path, dynamic)],
            site("entry", "**bad"),
            vec![
                frame("main", site("entry", "# main"), None),
                frame(
                    "explode",
                    site("entry", "## explode"),
                    Some(site("entry", "> call_fn")),
                ),
            ],
        );
        let error = if backend == "tree" {
            Interpreter::with_capture(Some(&entry_path), false).run_module(&method_module)
        } else {
            Vm::with_capture(Some(&entry_path)).run(&method_code)
        }
        .unwrap_err();
        let d = Diagnostic::find(&error).unwrap();
        assert_origin(d, &object_path, &object, line(&object, "**bad"));
        assert_eq!(d.frames.last().unwrap().function, "fail");
        assert_site(
            d.frames.last().unwrap().call_site.as_ref().unwrap(),
            &entry_path,
            method,
            line(method, "> `c`.fail"),
        );
        record(
            "method",
            backend,
            d,
            vec![
                source("entry", &entry_path, method),
                source("object", &object_path, &object),
            ],
            site("object", "**bad"),
            vec![
                frame("main", site("entry", "# main"), None),
                frame(
                    "fail",
                    site("object", "## fail"),
                    Some(site("entry", "> `c`.fail")),
                ),
            ],
        );
    }
}
