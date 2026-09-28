use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};

use marqdo::catalog::{write_catalog, CatalogOptions};
use marqdo::diagnostics::{Diagnostic, Span};
use marqdo::ext_cli::{add_ext, list_ext, remove_ext};
use marqdo::input_feed::load_stdin_file;
use marqdo::knowledge::{
    compile_policy, conflicts_json, ensure_compiled, find, find_duplicates, find_json,
    find_conflicts, find_stale, format_conflicts_text, format_duplicates_text, format_find_text,
    format_impact_text, format_preflight_yaml, format_reuse_text, format_stale_text,
    format_verify_text, impact, impact_json, learn, preflight, preflight_json, record_decision,
    resolve, reuse_json, stale_json, verify, verify_json, write_knowledge, KnowledgeOptions,
    LearnInput, ReuseBudget,
};
use marqdo::view::{serve, serve_debug, write_static, DebugOptions, OutputOptions, ViewOptions};
use marqdo::{Backend, RunOptions};

#[derive(Parser, Debug)]
#[command(name = "marqdo", version, about = "Marqdo interpreter — run, view, debug, catalog, knowledge, and ext")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum BackendCli {
    Tree,
    Bytecode,
}

impl From<BackendCli> for Backend {
    fn from(b: BackendCli) -> Self {
        match b {
            BackendCli::Tree => Backend::Tree,
            BackendCli::Bytecode => Backend::Bytecode,
        }
    }
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Execute a Marqdo source file (defaults to ./index.mq.md)
    Run {
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,

        #[arg(long, value_enum, default_value_t = BackendCli::Tree)]
        backend: BackendCli,

        #[arg(long)]
        dump_lines: bool,
        #[arg(long)]
        dump_tokens: bool,
        #[arg(long)]
        dump_ast: bool,
        #[arg(long)]
        dump_sema: bool,
        #[arg(long)]
        dump_bytecode: bool,
        #[arg(long)]
        trace_eval: bool,
        #[arg(long)]
        dump_all: bool,

        /// Feed `input` from a text file (one line per call), instead of / after the terminal.
        #[arg(long, value_name = "FILE")]
        stdin_file: Option<PathBuf>,

        /// Write `# main` return value as JSON (used by file subtasks).
        #[arg(long, value_name = "FILE")]
        emit_result: Option<PathBuf>,

        /// Artifact Metadata Binding: `KEY=VALUE` (repeatable). Feeds `arg.KEY` and overlays metadata.
        #[arg(long = "bind", value_name = "KEY=VALUE")]
        binds: Vec<String>,

        /// Filesystem sandbox + process cwd root (default: entry file's directory).
        /// Also read from env `MARQDO_FS_ROOT`. Use repo root when running `cli/…` entries.
        #[arg(long = "fs-root", value_name = "DIR")]
        fs_root: Option<PathBuf>,

        /// Emit diagnostics as JSON on stderr (machine-readable; AI/MLSP 面).
        #[arg(long)]
        json: bool,
    },
    /// Static contract cross-check (渐进式契约): drift, unknown types, misplaced tables
    Check {
        /// Program path (.mq.md)
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Emit one JSON diagnostic per line on stdout (machine-readable; AI/MLSP 面)
        #[arg(long)]
        json: bool,
    },
    /// MLSP for AI: line-delimited JSON on stdio (locate / syntax / validate / repair_targets / schema)
    Mlsp,
    /// Browse `.mq.md` structure + execution (live server or static output)
    View {
        #[command(subcommand)]
        action: Option<ViewAction>,

        /// Path when running the live server (default: `.`)
        #[arg(value_name = "PATH", global = true)]
        path: Option<PathBuf>,

        #[arg(long, default_value = "127.0.0.1", global = true)]
        host: String,

        #[arg(long, default_value_t = 7429, global = true)]
        port: u16,

        #[arg(long, global = true)]
        no_open: bool,
    },
    /// Interactive debugger (tree-walk breakpoints; separate UI from `view`)
    Debug {
        /// Path to a `.mq.md` file or directory (default: `.`)
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        #[arg(long, default_value_t = 7430)]
        port: u16,

        #[arg(long)]
        no_open: bool,
    },
    /// Generate OKF-compatible catalog YAML + module pages (+ Engineering Knowledge)
    Catalog {
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        #[arg(short = 'o', long = "out", default_value = ".marqdo")]
        out: PathBuf,

        /// Skip Engineering Knowledge Compiler after catalog
        #[arg(long)]
        no_knowledge: bool,
    },
    /// Alias for `catalog`
    Sync {
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        #[arg(short = 'o', long = "out", default_value = ".marqdo")]
        out: PathBuf,

        #[arg(long)]
        no_knowledge: bool,
    },
    /// Engineering Knowledge Compiler (L0–L4 catalog / graph / projections)
    Knowledge {
        #[command(subcommand)]
        action: Option<KnowledgeAction>,

        #[arg(value_name = "PATH", global = true)]
        path: Option<PathBuf>,

        #[arg(short = 'o', long = "out", default_value = ".marqdo", global = true)]
        out: PathBuf,

        #[arg(long, global = true)]
        json: bool,
    },
    /// Find engineering capabilities / symbols / knowledge
    Find {
        #[arg(value_name = "QUERY")]
        query: String,

        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        #[arg(short = 'o', long = "out", default_value = ".marqdo")]
        out: PathBuf,

        #[arg(long, default_value_t = 20)]
        limit: usize,

        #[arg(long)]
        json: bool,
    },
    /// Engineering First Information: REUSE / ADAPT / CREATE
    Reuse {
        #[arg(value_name = "TASK")]
        task: String,

        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        #[arg(short = 'o', long = "out", default_value = ".marqdo")]
        out: PathBuf,

        /// Also write preflight context pack + policy
        #[arg(long)]
        preflight: bool,

        #[arg(long, default_value_t = 3)]
        min_candidates: usize,

        #[arg(long, default_value_t = 2)]
        allow_create_after: usize,

        #[arg(long)]
        json: bool,
    },
    /// Knowledge impact of changed paths
    Impact {
        #[arg(value_name = "PATH")]
        paths: Vec<PathBuf>,

        #[arg(long, value_name = "ROOT", default_value = ".")]
        root: PathBuf,

        #[arg(short = 'o', long = "out", default_value = ".marqdo")]
        out: PathBuf,

        #[arg(long)]
        json: bool,
    },
    /// Detect knowledge conflicts / possible duplicates edges
    Conflicts {
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        #[arg(short = 'o', long = "out", default_value = ".marqdo")]
        out: PathBuf,

        #[arg(long)]
        json: bool,
    },
    /// List stale / deprecated / superseded knowledge
    Stale {
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        #[arg(short = 'o', long = "out", default_value = ".marqdo")]
        out: PathBuf,

        #[arg(long)]
        json: bool,
    },
    /// Detect duplicate capabilities via symbol fingerprints
    Duplicate {
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        #[arg(short = 'o', long = "out", default_value = ".marqdo")]
        out: PathBuf,

        #[arg(long, default_value_t = 0.75)]
        threshold: f64,

        /// Exit non-zero when duplicates found (CI gate)
        #[arg(long)]
        fail: bool,

        #[arg(long)]
        json: bool,
    },
    /// Verify code ↔ knowledge consistency
    Verify {
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        #[arg(short = 'o', long = "out", default_value = ".marqdo")]
        out: PathBuf,

        #[arg(long)]
        json: bool,
    },
    /// Official extension installer (`list` / `add` / `remove`)
    Ext {
        #[command(subcommand)]
        action: ExtAction,
    },
    /// Print version; `--check` compares with the latest GitHub release
    Version {
        #[arg(long)]
        check: bool,
    },
    /// Build browser WASM artifact (route C)
    Wasm {
        #[command(subcommand)]
        action: WasmAction,
    },
}

#[derive(Subcommand, Debug)]
enum KnowledgeAction {
    /// Compile L0–L4 projections (default)
    Compile,
    /// Run engineering preflight for a task
    Preflight {
        #[arg(value_name = "TASK")]
        task: String,

        #[arg(long, default_value_t = 3)]
        min_candidates: usize,

        #[arg(long, default_value_t = 2)]
        allow_create_after: usize,
    },
    /// Record a reuse decision into metrics
    Record {
        #[arg(value_name = "DECISION")]
        decision: String,

        #[arg(long)]
        duplicated: bool,
    },
    /// Learn failure/decision/constraint into knowledge/
    Learn {
        #[arg(long)]
        task: String,
        #[arg(long)]
        failed_approach: Option<String>,
        #[arg(long)]
        failure: Option<String>,
        #[arg(long)]
        correct_approach: Option<String>,
        #[arg(long)]
        decision_title: Option<String>,
        #[arg(long)]
        decision_body: Option<String>,
        #[arg(long)]
        constraint_title: Option<String>,
        #[arg(long)]
        constraint_body: Option<String>,
    },
}

#[derive(Subcommand, Debug)]
enum ExtAction {
    /// List official extensions and install status
    List,
    /// Install an official extension into the local ext root
    Add {
        #[arg(value_name = "NAME")]
        name: String,
    },
    /// Remove an installed official extension
    Remove {
        #[arg(value_name = "NAME")]
        name: String,
    },
}

#[derive(Subcommand, Debug)]
enum WasmAction {
    /// `cargo build -p marqdo-wasm --target wasm32-unknown-unknown --release` and copy `.wasm`
    Build {
        /// Destination directory (default: examples/browser-hello)
        #[arg(short = 'o', long = "out", default_value = "examples/browser-hello")]
        out: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
enum ViewAction {
    /// Write static HTML documentation site
    Output {
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        #[arg(short = 'o', long = "out", required = true)]
        out: PathBuf,

        #[arg(long)]
        no_exec: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let json_errors = matches!(
        &cli.command,
        Commands::Run { json: true, .. } | Commands::Check { json: true, .. }
    );
    match try_main(cli) {
        Ok(code) => ExitCode::from(code as u8),
        Err(err) => {
            if json_errors {
                let diag = match err.downcast::<Diagnostic>() {
                    Ok(d) => d,
                    Err(e) => Diagnostic::find(&e).cloned().unwrap_or_else(|| {
                        Diagnostic::new(None, Span::new(0, 0), format!("{e:#}"))
                    }),
                };
                eprintln!("{}", diag.to_json());
            } else {
                eprintln!("error: {err:#}");
            }
            ExitCode::from(1)
        }
    }
}

fn try_main(cli: Cli) -> Result<i32> {
    match cli.command {
        Commands::Run {
            file,
            backend,
            dump_lines,
            dump_tokens,
            dump_ast,
            dump_sema,
            dump_bytecode,
            trace_eval,
            dump_all,
            stdin_file,
            emit_result,
            binds,
            fs_root,
            json: _,
        } => {
            let path = file.unwrap_or_else(|| PathBuf::from("index.mq.md"));
            let stdin_lines = match stdin_file {
                Some(p) => load_stdin_file(&p)?,
                None => Vec::new(),
            };
            let mut bind_pairs = Vec::new();
            for b in &binds {
                bind_pairs.push(marqdo::binding::parse_bind_kv(b)?);
            }
            let env_fs_root = std::env::var_os("MARQDO_FS_ROOT").map(PathBuf::from);
            let fs_root = fs_root.or(env_fs_root);
            let mut opts = if dump_all {
                RunOptions {
                    stdin_lines: stdin_lines.clone(),
                    emit_result: emit_result.clone(),
                    binds: bind_pairs.clone(),
                    fs_root: fs_root.clone(),
                    ..RunOptions::dump_all()
                }
            } else {
                RunOptions {
                    dump_lines,
                    dump_tokens,
                    dump_ast,
                    dump_sema,
                    dump_bytecode,
                    trace_eval,
                    backend: backend.into(),
                    stdin_lines,
                    emit_result,
                    binds: bind_pairs,
                    fs_root,
                    ..RunOptions::default()
                }
            };
            if dump_all {
                opts.backend = backend.into();
            }
            marqdo::run_file(&path, &opts)?;
            Ok(0)
        }
        Commands::Check { file, json } => {
            let diags = marqdo::check::check_path(&file)?;
            let mut errors = 0i32;
            for d in &diags {
                if matches!(d.severity, marqdo::diagnostics::Severity::Error) {
                    errors += 1;
                }
                if json {
                    println!("{}", d.to_json());
                } else {
                    println!("{}", d.format_message());
                }
            }
            if !json {
                if diags.is_empty() {
                    println!("contract check OK: 无契约问题");
                } else {
                    println!("contract check: {} 条诊断（{} error）", diags.len(), errors);
                }
            }
            Ok(errors.min(1))
        }
        Commands::Mlsp => {
            use std::io::{BufRead, Write};
            let stdin = std::io::stdin();
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            for line in stdin.lock().lines() {
                let line = line?;
                if line.trim().is_empty() {
                    continue;
                }
                writeln!(out, "{}", marqdo::mlsp::handle_line(&line))?;
                out.flush()?;
            }
            Ok(0)
        }
        Commands::View {
            action,
            path,
            host,
            port,
            no_open,
        } => match action {
            Some(ViewAction::Output {
                path: out_path,
                out,
                no_exec,
            }) => {
                write_static(OutputOptions {
                    path: out_path
                        .or(path)
                        .unwrap_or_else(|| PathBuf::from(".")),
                    out_dir: out,
                    no_exec,
                })?;
                Ok(0)
            }
            None => {
                serve(ViewOptions {
                    path: path.unwrap_or_else(|| PathBuf::from(".")),
                    host,
                    port,
                    open_browser: !no_open,
                })?;
                Ok(0)
            }
        },
        Commands::Debug {
            path,
            host,
            port,
            no_open,
        } => {
            serve_debug(DebugOptions {
                path: path.unwrap_or_else(|| PathBuf::from(".")),
                host,
                port,
                open_browser: !no_open,
            })?;
            Ok(0)
        }
        Commands::Catalog {
            path,
            out,
            no_knowledge,
        }
        | Commands::Sync {
            path,
            out,
            no_knowledge,
        } => {
            let path = path.unwrap_or_else(|| PathBuf::from("."));
            write_catalog(CatalogOptions {
                path: path.clone(),
                out_dir: out.clone(),
            })?;
            if !no_knowledge {
                write_knowledge(KnowledgeOptions {
                    path,
                    out_dir: out,
                })?;
            }
            Ok(0)
        }
        Commands::Knowledge {
            action,
            path,
            out,
            json,
        } => {
            let path = path.unwrap_or_else(|| PathBuf::from("."));
            match action.unwrap_or(KnowledgeAction::Compile) {
                KnowledgeAction::Compile => {
                    write_knowledge(KnowledgeOptions {
                        path,
                        out_dir: out,
                    })?;
                    Ok(0)
                }
                KnowledgeAction::Preflight {
                    task,
                    min_candidates,
                    allow_create_after,
                } => {
                    let graph = ensure_compiled(&path, &out)?;
                    let budget = ReuseBudget {
                        min_candidates,
                        allow_create_after,
                        ..ReuseBudget::default()
                    };
                    let pf = preflight(&graph, &task, &budget, Some(&out))?;
                    let _ = compile_policy(&out, &task, &resolve(&graph, &task, &budget));
                    if json {
                        println!("{}", preflight_json(&pf));
                    } else {
                        print!("{}", format_preflight_yaml(&pf));
                    }
                    Ok(if pf.create_allowed
                        || pf.decision == "REUSE"
                        || pf.decision == "ADAPT"
                    {
                        0
                    } else {
                        2
                    })
                }
                KnowledgeAction::Record {
                    decision,
                    duplicated,
                } => {
                    let m = record_decision(&out, &decision, duplicated)?;
                    if json {
                        println!("{}", m.to_json());
                    } else {
                        println!(
                            "reuse_ratio={:.2} adapt_ratio={:.2} novel_ratio={:.2} duplication_rate={:.2}",
                            m.reuse_ratio(),
                            m.adapt_ratio(),
                            m.novel_ratio(),
                            m.duplication_rate()
                        );
                    }
                    Ok(0)
                }
                KnowledgeAction::Learn {
                    task,
                    failed_approach,
                    failure,
                    correct_approach,
                    decision_title,
                    decision_body,
                    constraint_title,
                    constraint_body,
                } => {
                    let written = learn(
                        &out,
                        &LearnInput {
                            task,
                            failed_approach,
                            failure,
                            correct_approach,
                            decision_title,
                            decision_body,
                            constraint_title,
                            constraint_body,
                        },
                    )?;
                    for p in written {
                        println!("{}", p.display());
                    }
                    Ok(0)
                }
            }
        }
        Commands::Find {
            query,
            path,
            out,
            limit,
            json,
        } => {
            let path = path.unwrap_or_else(|| PathBuf::from("."));
            let graph = ensure_compiled(&path, &out)?;
            let hits = find(&graph, &query, limit);
            if json {
                println!("{}", find_json(&query, &hits));
            } else {
                print!("{}", format_find_text(&query, &hits, &graph));
            }
            Ok(0)
        }
        Commands::Reuse {
            task,
            path,
            out,
            preflight: do_preflight,
            min_candidates,
            allow_create_after,
            json,
        } => {
            let path = path.unwrap_or_else(|| PathBuf::from("."));
            let graph = ensure_compiled(&path, &out)?;
            let budget = ReuseBudget {
                min_candidates,
                allow_create_after,
                ..ReuseBudget::default()
            };
            let result = resolve(&graph, &task, &budget);
            let _ = compile_policy(&out, &task, &result);
            if do_preflight {
                let _ = preflight(&graph, &task, &budget, Some(&out))?;
            }
            if json {
                println!("{}", reuse_json(&task, &result));
            } else {
                print!("{}", format_reuse_text(&task, &result));
            }
            Ok(0)
        }
        Commands::Impact {
            paths,
            root,
            out,
            json,
        } => {
            let graph = ensure_compiled(&root, &out)?;
            let path_strs: Vec<String> = paths
                .iter()
                .map(|p| p.to_string_lossy().replace('\\', "/"))
                .collect();
            let report = impact(&graph, &path_strs);
            if json {
                println!("{}", impact_json(&report));
            } else {
                print!("{}", format_impact_text(&report));
            }
            Ok(0)
        }
        Commands::Conflicts { path, out, json } => {
            let path = path.unwrap_or_else(|| PathBuf::from("."));
            let graph = ensure_compiled(&path, &out)?;
            let items = find_conflicts(&graph);
            if json {
                println!("{}", conflicts_json(&items));
            } else {
                print!("{}", format_conflicts_text(&items));
            }
            Ok(0)
        }
        Commands::Stale { path, out, json } => {
            let path = path.unwrap_or_else(|| PathBuf::from("."));
            let graph = ensure_compiled(&path, &out)?;
            let items = find_stale(&graph, &path);
            if json {
                println!("{}", stale_json(&items));
            } else {
                print!("{}", format_stale_text(&items));
            }
            Ok(0)
        }
        Commands::Duplicate {
            path,
            out,
            threshold,
            fail,
            json,
        } => {
            let path = path.unwrap_or_else(|| PathBuf::from("."));
            let graph = ensure_compiled(&path, &out)?;
            let pairs = find_duplicates(&graph.symbols, threshold);
            if json {
                println!(
                    "{}",
                    serde_json::json!({
                        "threshold": threshold,
                        "duplicates": pairs.iter().map(|p| serde_json::json!({
                            "left": p.left,
                            "right": p.right,
                            "similarity": p.similarity,
                            "left_resource": p.left_resource,
                            "right_resource": p.right_resource,
                        })).collect::<Vec<_>>(),
                    })
                );
            } else {
                print!("{}", format_duplicates_text(&pairs));
            }
            if fail && !pairs.is_empty() {
                Ok(1)
            } else {
                Ok(0)
            }
        }
        Commands::Verify { path, out, json } => {
            let path = path.unwrap_or_else(|| PathBuf::from("."));
            let graph = ensure_compiled(&path, &out)?;
            let issues = verify(&graph);
            if json {
                println!("{}", verify_json(&issues));
            } else {
                print!("{}", format_verify_text(&issues));
            }
            let errors = issues.iter().filter(|i| i.severity == "error").count();
            Ok(if errors > 0 { 1 } else { 0 })
        }
        Commands::Ext { action } => {
            match action {
                ExtAction::List => list_ext()?,
                ExtAction::Add { name } => add_ext(&name)?,
                ExtAction::Remove { name } => remove_ext(&name)?,
            }
            Ok(0)
        }
        Commands::Version { check } => {
            if check {
                marqdo::version_check::check_latest().map_err(|e| anyhow::anyhow!(e))?;
            } else {
                marqdo::version_check::print_version();
            }
            Ok(0)
        }
        Commands::Wasm { action } => match action {
            WasmAction::Build { out } => {
                marqdo::wasm_cli::build_wasm(&out)?;
                Ok(0)
            }
        },
    }
}
