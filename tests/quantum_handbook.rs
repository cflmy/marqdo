//! The source documents are the examples under test, not a copied command catalog.
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
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
        let path = std::env::temp_dir().join(format!(
            "mq-quantum-handbook-{label}-{}",
            std::process::id()
        ));
        fs::create_dir_all(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files(&path, out);
        } else if path.to_string_lossy().ends_with(".mq.md") {
            out.push(path);
        }
    }
}
fn fences(source: &str, language: &str) -> Vec<String> {
    let open = format!("```{language}");
    let mut current = None;
    let mut result = Vec::new();
    for line in source.lines() {
        if line == open {
            current = Some(String::new());
        } else if line == "```" {
            if let Some(block) = current.take() {
                result.push(block);
            }
        } else if let Some(block) = current.as_mut() {
            block.push_str(line);
            block.push('\n');
        }
    }
    assert!(current.is_none(), "unclosed {language} fence");
    result
}
fn near(v: &Value, expected: f64) {
    let actual = v
        .as_f64()
        .unwrap_or_else(|| panic!("expected number, got {v}"));
    assert!((actual - expected).abs() < 1e-9, "{actual} != {expected}");
}
fn bell(v: &Value) {
    near(&v["00"], 0.5);
    near(&v["11"], 0.5);
    near(
        &Value::from(
            v.as_object()
                .unwrap()
                .values()
                .map(|v| v.as_f64().unwrap())
                .sum::<f64>(),
        ),
        1.0,
    );
}
fn report(v: &Value) {
    bell(&v["probabilities"]);
    near(&v["purity"], 1.0);
    near(&v["reduced_purity"], 0.5);
    near(&v["zz"], 1.0);
    near(&v["schmidt"]["entropy"], 2.0_f64.ln());
    for coefficient in v["schmidt"]["coeffs"].as_array().unwrap() {
        near(coefficient, 0.5_f64.sqrt());
    }
}
fn command(scratch: &Path) -> Command {
    let mut command = Command::new(bin());
    command
        .current_dir(root())
        .env("MARQDO_CALL_SESSION_DIR", scratch)
        .env(
            "MARQDO_QUANTUM_PLUGIN",
            root()
                .join("target/debug")
                .join(if cfg!(target_os = "macos") {
                    "libquantum.dylib"
                } else if cfg!(windows) {
                    "quantum.dll"
                } else {
                    "libquantum.so"
                }),
        )
        .env("MARQDO_EXT", root().join("ext"))
        .env("MARQDO_LIB", root().join("lib"));
    command
}
fn call(scratch: &Path, backend: &str, file: &str, function: &str, args: &[&str]) -> Value {
    let output = command(scratch)
        .args([
            "call",
            file,
            function,
            "--effects",
            "plugin",
            "--backend",
            backend,
            "--session",
            "handbook",
            "--json",
        ])
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{file} {function}: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let body: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(body["ok"], true, "{body}");
    body["result"].clone()
}

#[test]
fn handbook_shell_and_author_examples_on_both_backends() {
    let scratch = Scratch::new("examples");
    let mut paths = Vec::new();
    files(&root().join("ext/quantum"), &mut paths);
    paths.sort();
    assert_eq!(paths.len(), 16);
    let mut shell_calls = 0;
    let mut author_examples = 0;
    for (index, path) in paths.iter().enumerate() {
        let source = fs::read_to_string(path).unwrap();
        let module = path
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        let examples = fences(&source, "sh");
        assert!(
            !examples.is_empty(),
            "{} needs a standalone experiment",
            path.display()
        );
        let mut reference = Vec::new();
        for backend in ["tree", "bytecode"] {
            let session_dir = scratch.0.join(format!("{index}-{backend}"));
            fs::create_dir_all(&session_dir).unwrap();
            let mut observed = Vec::new();
            for (block_index, block) in examples.iter().enumerate() {
                let mut results = Vec::new();
                for line in block.lines().filter(|line| !line.trim().is_empty()) {
                    assert!(
                        line.starts_with("marqdo call "),
                        "unsupported handbook command: {line}"
                    );
                    // Use the actual shell: invoking Command with argv would miss broken quoting.
                    let invocation = line.replacen(
                        "marqdo call ",
                        &format!("\"{}\" call --backend {backend} ", bin()),
                        1,
                    );
                    let cmd = command(&session_dir);
                    // Keep the same isolated environment as non-shell calls.
                    let mut shell = Command::new("bash");
                    shell
                        .env_remove("BASH_ENV")
                        .current_dir(root())
                        .envs(cmd.get_envs().map(|(k, v)| (k, v.unwrap())))
                        .args(["-euc", &invocation]);
                    let output = shell.output().unwrap();
                    assert!(
                        output.status.success(),
                        "{} [{backend}] {line}\n{}\n{}",
                        path.display(),
                        String::from_utf8_lossy(&output.stdout),
                        String::from_utf8_lossy(&output.stderr)
                    );
                    let body: Value = serde_json::from_slice(&output.stdout).unwrap();
                    assert_eq!(body["ok"], true, "{}: {body}", path.display());
                    results.push(body["result"].clone());
                    shell_calls += 1;
                }
                let last = results.last().unwrap();
                match module {
                    "quantum" | "量子" => {
                        if block_index == 0 {
                            bell(&results[0]);
                            report(&results[1]);
                        } else {
                            near(last, 0.5);
                        }
                    }
                    "circuit" | "电路" => bell(last),
                    "gate" | "门" => {
                        assert_eq!(results[1], true);
                        near(&last["1"], 1.0);
                    }
                    "simulate" | "模拟" => {
                        near(&results[3]["1"], 1.0);
                        assert_eq!(last["counts"]["0"], 32);
                    }
                    "state" | "态" => {
                        assert_eq!(results[2]["qubits"], 2);
                        near(&results[3], 1.0);
                        near(&last["entropy"], 0.0);
                    }
                    "density" | "密度" => {
                        near(&results[1], 0.5);
                        near(&results[2], 0.0);
                        for v in last["eigenvalues"].as_array().unwrap() {
                            near(v, 0.5);
                        }
                    }
                    "viz" | "可视化" => {
                        for (value, kind) in
                            results[2..].iter().zip(["probs", "qsphere", "multibloch"])
                        {
                            assert_eq!(value["kind"], kind);
                            assert!(value["svg"].as_str().unwrap().starts_with("<svg"));
                        }
                    }
                    "_capability" | "能力" => assert!(last.is_null()),
                    other => panic!("unverified module {other}"),
                }
                observed.extend(results);
            }
            if backend == "tree" {
                reference = observed;
            } else {
                assert_eq!(observed, reference, "backend drift in {}", path.display());
            }
            for (block_index, block) in fences(&source, "marqdo").iter().enumerate() {
                let file = scratch
                    .0
                    .join(format!("author-{index}-{block_index}.mq.md"));
                fs::write(&file, block).unwrap();
                let output = command(&session_dir)
                    .args(["run", file.to_str().unwrap(), "--backend", backend])
                    .output()
                    .unwrap();
                assert!(
                    output.status.success(),
                    "{} author [{backend}]: {}",
                    path.display(),
                    String::from_utf8_lossy(&output.stderr)
                );
                let printed = String::from_utf8(output.stdout).unwrap();
                let weights: Vec<f64> = printed.lines().map(|line| line.parse().unwrap()).collect();
                assert_eq!(weights.len(), 2, "{}: {printed}", path.display());
                for weight in weights {
                    near(&Value::from(weight), 0.5);
                }
                author_examples += 1;
            }
        }
    }
    eprintln!("Executed {shell_calls} shell calls directly from the handbooks");
    assert_eq!(author_examples, 4);
}

#[test]
fn every_callable_has_a_real_contract_and_no_prose_placeholder() {
    fn check(functions: &[marqdo::ast::Function], path: &Path) {
        for function in functions {
            let contract = function.contract.as_ref().unwrap_or_else(|| {
                panic!("{}: {} missing contract", path.display(), function.name)
            });
            assert!(
                contract.returns.is_some(),
                "{}: {} missing return contract",
                path.display(),
                function.name
            );
            assert_eq!(
                contract.params.len(),
                function.params.len(),
                "{}: {} parameter drift",
                path.display(),
                function.name
            );
            check(&function.children, path);
        }
    }
    let mut paths = Vec::new();
    files(&root().join("ext/quantum"), &mut paths);
    for path in paths {
        let module = marqdo::load::load_module(&path).unwrap();
        let diagnostics = marqdo::check::check_path(&path).unwrap();
        assert!(
            diagnostics.is_empty(),
            "{}: {diagnostics:?}",
            path.display()
        );
        assert!(
            marqdo::narrative_ratio::check_file(&path)
                .unwrap()
                .is_empty(),
            "{} narration ratio",
            path.display()
        );
        check(&module.functions, &path);
        if matches!(
            path.parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap(),
            "quantum" | "量子"
        ) {
            assert_eq!(
                module.functions.len(),
                3,
                "root headings must be executable tools"
            );
        }
    }
}

#[test]
fn free_tools_cover_gate_composition_and_matrix_observation() {
    let scratch = Scratch::new("tools");
    for backend in ["tree", "bytecode"] {
        for zh in [false, true] {
            let circuit = if zh {
                "ext/quantum/量子/电路/索引.mq.md"
            } else {
                "ext/quantum/circuit/index.mq.md"
            };
            let gate = if zh {
                "ext/quantum/量子/门/索引.mq.md"
            } else {
                "ext/quantum/gate/index.mq.md"
            };
            let density = if zh {
                "ext/quantum/量子/密度/索引.mq.md"
            } else {
                "ext/quantum/density/index.mq.md"
            };
            call(
                &scratch.0,
                backend,
                circuit,
                if zh { "新建" } else { "new" },
                &["--toget", "qc"],
            );
            call(
                &scratch.0,
                backend,
                gate,
                if zh { "具名" } else { "named" },
                &[
                    if zh { "名=\"Rx\"" } else { "name=\"Rx\"" },
                    if zh {
                        "参数=3.141592653589793"
                    } else {
                        "theta=3.141592653589793"
                    },
                    "--toget",
                    "u",
                ],
            );
            let m = call(
                &scratch.0,
                backend,
                gate,
                if zh { "矩阵" } else { "matrix" },
                &[if zh { "门=$u" } else { "gate=$u" }],
            );
            near(&m[0][0]["re"], 0.0);
            near(&m[0][1]["im"], -1.0);
            call(
                &scratch.0,
                backend,
                circuit,
                if zh { "施加" } else { "apply" },
                &[
                    if zh { "电路=$qc" } else { "circuit=$qc" },
                    if zh { "门=$u" } else { "gate=$u" },
                    "--toget",
                    "qc",
                ],
            );
            call(
                &scratch.0,
                backend,
                circuit,
                if zh { "新建" } else { "new" },
                &["--toget", "fragment"],
            );
            call(
                &scratch.0,
                backend,
                circuit,
                if zh { "绕X" } else { "rx" },
                &[
                    if zh {
                        "电路=$fragment"
                    } else {
                        "circuit=$fragment"
                    },
                    if zh {
                        "参数=3.141592653589793"
                    } else {
                        "theta=3.141592653589793"
                    },
                    "--toget",
                    "fragment",
                ],
            );
            call(
                &scratch.0,
                backend,
                circuit,
                if zh { "追加" } else { "append" },
                &[
                    if zh { "电路=$qc" } else { "circuit=$qc" },
                    if zh {
                        "操作=$fragment"
                    } else {
                        "op=$fragment"
                    },
                    "--toget",
                    "qc",
                ],
            );
            call(
                &scratch.0,
                backend,
                density,
                if zh { "从态" } else { "from_state" },
                &[if zh { "态=$qc" } else { "state=$qc" }, "--toget", "rho"],
            );
            let rho = call(
                &scratch.0,
                backend,
                density,
                if zh { "矩阵" } else { "matrix" },
                &[if zh { "密度=$rho" } else { "density=$rho" }],
            );
            near(&rho[0][0]["re"], 1.0);
            near(&rho[1][1]["re"], 0.0);
        }
    }
}

#[test]
fn drawing_returns_svg_in_call_and_saves_files_in_run() {
    let scratch = Scratch::new("plot-host");
    for backend in ["tree", "bytecode"] {
        call(
            &scratch.0,
            backend,
            "ext/quantum/index.mq.md",
            "bell_circuit",
            &["--toget", "qc"],
        );
        let forbidden = scratch.0.join(format!("denied-{backend}.svg"));
        let path_arg = format!("path=\"{}\"", forbidden.display());
        let output = command(&scratch.0)
            .args([
                "call",
                "ext/quantum/viz/index.mq.md",
                "draw_probs",
                "circuit=$qc",
                &path_arg,
                "--effects",
                "plugin",
                "--backend",
                backend,
                "--session",
                "handbook",
                "--json",
            ])
            .output()
            .unwrap();
        assert!(!output.status.success());
        let body: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(body["ok"], false);
        assert!(
            body["error"]["message"]
                .as_str()
                .unwrap()
                .contains("write_text denied"),
            "{body}"
        );
        assert!(!forbidden.exists());

        for zh in [false, true] {
            let file = scratch.0.join(format!("save-{backend}-{zh}.mq.md"));
            let image = format!("saved-{backend}-{zh}.svg");
            let import = if zh {
                "电路: ext/quantum/量子/电路/索引.mq.md"
            } else {
                "circuit: ext/quantum/circuit/index.mq.md"
            };
            let constructor = if zh {
                "电路.电路 比特数=1"
            } else {
                "circuit.circuit qubits=1"
            };
            let draw = if zh { "绘图 路径" } else { "draw path" };
            fs::write(&file, format!("---\nmarkup: occlusion\nimport:\n  {import}\n---\n\n# main\n\n**qc = {constructor}**\n**img = > `qc`.{draw}=\"{image}\"**\n")).unwrap();
            let output = command(&scratch.0)
                .args(["run", file.to_str().unwrap(), "--backend", backend])
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let svg = fs::read_to_string(scratch.0.join(image)).unwrap();
            assert!(svg.starts_with("<svg") && svg.contains("</svg>"));
        }
    }
}
