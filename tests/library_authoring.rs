//! Exercise the authoring rules and actual package workflows, including repaired old paths.
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::OnceLock,
};
fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}
fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            sources(&p, out)
        } else if p.to_string_lossy().ends_with(".mq.md") {
            out.push(p)
        }
    }
}
fn native() {
    static READY: OnceLock<()> = OnceLock::new();
    READY.get_or_init(|| {
        let out = Command::new("cargo")
            .current_dir(root())
            .args([
                "build",
                "--offline",
                "-p",
                "marqdo_plugin_agent",
                "-p",
                "marqdo_plugin_linalg",
            ])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let out = Command::new("bash")
            .current_dir(root())
            .arg("scripts/build-web-plugin.sh")
            .env("GOFLAGS", "-buildvcs=false")
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
    });
}
fn command() -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_marqdo"));
    c.current_dir(root())
        .env("MARQDO_EXT", root().join("ext"))
        .env("MARQDO_LIB", root().join("lib"));
    for (name, file) in [
        ("WEB", "plugins/web/build/libweb.so"),
        ("AGENT", "target/debug/libagent.so"),
        ("LINALG", "target/debug/liblinalg.so"),
    ] {
        c.env(format!("MARQDO_{name}_PLUGIN"), root().join(file));
    }
    c
}
fn call(backend: &str, file: &str, function: &str, args: &[&str], effects: &str) -> Value {
    let mut cmd = command();
    let mut root_arg = None;
    if effects.contains("fs.write") {
        let work = std::env::temp_dir().join(format!("mq-authoring-data-{}", std::process::id()));
        fs::create_dir_all(&work).unwrap();
        root_arg = Some(format!("root=\"{}\"", work.display()));
        cmd.arg("call").arg("--fs-root").arg(work);
    } else {
        cmd.arg("call");
    }
    let out = cmd
        .args([
            "--backend",
            backend,
            "--json",
            "--effects",
            effects,
            file,
            function,
        ])
        .args(args)
        .args(root_arg)
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&out.stdout).unwrap_or_else(|_| {
        panic!(
            "{file} {function}: {} {}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        )
    });
    assert!(out.status.success(), "{backend} {file} {function}: {v}");
    assert_eq!(v["ok"], true, "{v}");
    v["result"].clone()
}
#[test]
fn all_library_units_follow_the_single_source_guide() {
    let mut paths = Vec::new();
    sources(&root().join("lib"), &mut paths);
    sources(&root().join("ext"), &mut paths);
    assert!(paths.len() >= 125);
    let mut gaps = Vec::new();
    for path in paths {
        let source = fs::read_to_string(&path).unwrap();
        let module = marqdo::load::load_module(&path)
            .unwrap_or_else(|e| panic!("{}: {e:#}", path.display()));
        let diagnostics = marqdo::check::check_path(&path).unwrap();
        if !diagnostics.is_empty() {
            gaps.push(format!("{}: {diagnostics:?}", path.display()))
        }
        for report in marqdo::narrative_ratio::measure_source_units(&source, &module.functions) {
            if report.ratio <= 0.5 {
                gaps.push(format!(
                    "{} {} narrative {:.1}%",
                    path.display(),
                    report.name,
                    report.ratio * 100.0
                ))
            }
        }
        fn units(
            functions: &[marqdo::ast::Function],
            source: &str,
            path: &Path,
            gaps: &mut Vec<String>,
        ) {
            for f in functions {
                let c = f
                    .contract
                    .as_ref()
                    .unwrap_or_else(|| panic!("{} {} missing contract", path.display(), f.name));
                assert!(
                    c.returns.is_some(),
                    "{} {} no return contract",
                    path.display(),
                    f.name
                );
                assert_eq!(
                    c.params.len(),
                    f.params.len(),
                    "{} {} parameter mismatch",
                    path.display(),
                    f.name
                );
                assert!(
                    f.params.iter().all(|p| p.inferred),
                    "{} {} legacy parameter",
                    path.display(),
                    f.name
                );
                let first = source
                    .lines()
                    .skip(f.span.line as usize)
                    .find(|l| !l.trim().is_empty())
                    .unwrap();
                if !first.starts_with(if f.params.is_empty() {
                    "| 返回 |"
                } else {
                    "| 参数 |"
                }) {
                    gaps.push(format!(
                        "{} {} contracts are not directly below heading",
                        path.display(),
                        f.name
                    ))
                }
                units(&f.children, source, path, gaps);
            }
        }
        units(&module.functions, &source, &path, &mut gaps);
        let mut fence = false;
        for (line, text) in source.lines().enumerate() {
            let t = text.trim();
            if t.starts_with("```") {
                fence = !fence;
                continue;
            }
            if fence {
                continue;
            }
            if t.starts_with("+ `") {
                gaps.push(format!("{}:{} legacy parameters", path.display(), line + 1))
            }
            if t.starts_with('>') {
                gaps.push(format!("{}:{} isolated call", path.display(), line + 1))
            }
            if t.starts_with('*')
                && t.ends_with('*')
                && !t.starts_with("**_ =")
                && !t.trim_matches('*').trim_end().ends_with('=')
            {
                gaps.push(format!(
                    "{}:{} isolated statement/return",
                    path.display(),
                    line + 1
                ))
            }
        }
    }
    assert!(
        gaps.is_empty(),
        "{} authoring gaps:\n{}",
        gaps.len(),
        gaps.join("\n")
    );
}
#[test]
fn new_package_workflows_return_real_results_on_both_backends() {
    native();
    for backend in ["tree", "bytecode"] {
        let cached = call(
            backend,
            "ext/data/index.mq.md",
            "memory_cache",
            &["key=\"manual-test\"", "value=\"retained\""],
            "plugin",
        );
        assert_eq!(cached["value"], "retained");
        let tx = call(
            backend,
            "ext/data/index.mq.md",
            "transaction_roundtrip",
            &[],
            "plugin,fs.write",
        );
        assert_eq!(tx["rows"].as_array().unwrap().len(), 1);
        assert_eq!(tx["rows"][0]["title"], "committed");
        fs::remove_dir_all(tx["work_dir"].as_str().unwrap()).unwrap();
        for (held, allowed) in [("notes:read", true), ("notes:write", false)] {
            let arg = format!("held=\"{held}\"");
            let result = call(
                backend,
                "ext/security/index.mq.md",
                "check_permission",
                &[&arg, "needed=\"notes:read\""],
                "plugin",
            );
            assert_eq!(result["ok"], true);
            assert_eq!(result["allowed"], allowed);
        }
        assert_eq!(
            call(
                backend,
                "ext/net/index.mq.md",
                "encode",
                &["text=\"Marqdo notes/one\""],
                "none"
            ),
            "Marqdo%20notes%2Fone"
        );
        for (file, entry) in [
            ("ext/linalg/index.mq.md", "solve_example"),
            ("ext/linalg/索引.mq.md", "求解示例"),
        ] {
            let result = call(backend, file, entry, &[], "plugin");
            let rows = &result["x"]["data"];
            assert!(
                (rows[0][0].as_f64().unwrap() - 2.0).abs() < 1e-9,
                "{result}"
            );
            assert!(
                (rows[1][0].as_f64().unwrap() - 1.0).abs() < 1e-9,
                "{result}"
            );
        }
        for (file, entry) in [
            ("ext/ai/index.mq.md", "offline_example"),
            ("ext/ai/索引.mq.md", "离线示例"),
        ] {
            assert_eq!(call(backend, file, entry, &[], "none"), "Hello world");
        }
        let usage = call(
            backend,
            "ext/ai/llm/index.mq.md",
            "usage",
            &["data={\"usage\":{\"prompt_tokens\":2,\"completion_tokens\":3,\"total_tokens\":5}}"],
            "none",
        );
        assert_eq!(usage["total_tokens"], 5);
        let policy = call(
            backend,
            "ext/ai/policies/step-route.mq.md",
            "routes",
            &[],
            "none",
        );
        assert_eq!(policy["kind"][0], "lookup");
        assert_eq!(policy["backend"][4], "auto");
    }
}
#[test]
fn chinese_step_runs_its_body_instead_of_returning_a_prose_word() {
    native();
    let path = std::env::temp_dir().join(format!(
        "mq-authoring-chinese-step-{}.mq.md",
        std::process::id()
    ));
    fs::write(
        &path,
        r#"---
markup: occlusion
import:
  智能: ext/ai/智能体.mq.md
  表: lib/表.mq.md
  json: lib/json.mq.md
---
# 回复器

*None*

## 运行

根据上下文`提示`生成离线固定回答；本例不连接模型服务。

**字符数 = len value=`提示`**
*"deterministic answer"*

# main

**模型 = 回复器()**
**工具 = 表.空表()**
**执行器 = 智能.智能体(大模型=`模型`, 工具=`工具`, 站立提示="")**
**结果 = > `执行器`.单步 任务="offline step" 写回=False**
**wire = json.stringify value=`结果`**
> print text=`wire`
"#,
    )
    .unwrap();
    for backend in ["tree", "bytecode"] {
        let output = command()
            .args(["run", path.to_str().unwrap(), "--backend", backend])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let v: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(v["status"], "ok");
        assert_eq!(v["task"], "offline step");
        assert_eq!(v["result"], "deterministic answer");
    }
    fs::remove_file(path).unwrap();
}

#[test]
fn pair_records_preserve_nulls_last_value_and_source_rows() {
    for backend in ["tree", "bytecode"] {
        for (file, entry, argument) in [
            ("lib/table.mq.md", "from_pairs", "pairs"),
            ("lib/表.mq.md", "键值转记录", "键值对"),
        ] {
            assert_eq!(
                call(backend, file, entry, &[&format!("{argument}=[]")], "none"),
                serde_json::json!({})
            );
            let rows = format!("{argument}=[{{\"key\":\"first\",\"value\":1}},{{\"key\":\"null\",\"value\":null}},{{\"key\":\"first\",\"value\":3,\"ignored\":true}}]");
            assert_eq!(
                call(backend, file, entry, &[&rows], "none"),
                serde_json::json!({"first":3,"null":null})
            );
        }
        let path =
            std::env::temp_dir().join(format!("mq-pair-records-{}.mq.md", std::process::id()));
        fs::write(
            &path,
            r#"---
import:
  table: lib/table.mq.md
  json: lib/json.mq.md
---
# main

`rows` =

| @ | key | value |
|---|---|---|
| 1 | first | 1 |
| 2 | second | None |
| 3 | first | 3 |

**original = json.stringify(value=`rows`)**
**record = table.from_pairs(pairs=`rows`)**
**after = json.stringify(value=`rows`)**
**keys = table.keys(map=`record`)**

`out` =

| original | after | keys |
|---|---|---|
| `original` | `after` | `keys` |

**wire = json.stringify(value=`out`)**
> print text=`wire`
"#,
        )
        .unwrap();
        let output = command()
            .args(["run", path.to_str().unwrap(), "--backend", backend])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(value["original"], value["after"]);
        assert_eq!(value["keys"], serde_json::json!(["first", "second"]));
        fs::remove_file(path).unwrap();
    }
}

#[test]
fn shipped_package_sources_include_their_runtime_imports_and_entries() {
    use std::collections::BTreeSet;
    let catalog = marqdo::ext_cli::CATALOG;
    let files: BTreeSet<_> = catalog
        .iter()
        .flat_map(|p| p.mq_files.iter().copied())
        .collect();
    for package in catalog {
        for file in package.mq_files {
            let path = root().join("ext").join(file);
            let module = marqdo::load::load_module(&path).unwrap();
            for import in &module.imports {
                if let Some(dependency) = import.path.strip_prefix("ext/") {
                    assert!(
                        files.contains(dependency),
                        "{} imports an unshipped source {}",
                        path.display(),
                        import.path
                    );
                }
            }
        }
    }
    for entry in [
        "ai/index.mq.md",
        "ai/索引.mq.md",
        "ai/llm/index.mq.md",
        "ai/policies/step-route.mq.md",
        "data/index.mq.md",
        "net/index.mq.md",
        "security/index.mq.md",
        "linalg/index.mq.md",
        "linalg/索引.mq.md",
    ] {
        assert!(files.contains(entry), "unshipped entry {entry}");
    }
}
