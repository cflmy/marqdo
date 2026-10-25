use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};

use marqdo::catalog::{write_catalog, CatalogOptions};
use marqdo::diagnostics::{Diagnostic, Span};
use marqdo::ext_cli::{add_ext, list_ext, remove_ext};
use marqdo::input_feed::load_stdin_file;
use marqdo::knowledge::{
    compile_policy, conflicts_json, ensure_compiled, find, find_conflicts, find_duplicates,
    find_json, find_stale, format_conflicts_text, format_duplicates_text, format_find_text,
    format_impact_text, format_preflight_yaml, format_reuse_text, format_stale_text,
    format_verify_text, impact, impact_json, learn, preflight, preflight_json, record_decision,
    resolve, reuse_json, stale_json, verify, verify_json, write_knowledge, KnowledgeOptions,
    LearnInput, ReuseBudget,
};
use marqdo::view::{
    serve, serve_debug, write_capture, write_static, CaptureOptions, DebugOptions, OutputOptions,
    ViewOptions,
};
use marqdo::{Backend, RunOptions};

#[derive(Parser, Debug)]
#[command(
    name = "marqdo",
    version,
    about = "Marqdo interpreter — run, view, debug, catalog, knowledge, and ext"
)]
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
enum MigrateAction {
    /// `[键](集合)` → `access(集合, 键)`；`- [项](集合)` → `- [项](遍历 集合 为 项)`
    Labels {
        #[arg(value_name = "PATH", default_value = ".")]
        paths: Vec<PathBuf>,
        /// Actually rewrite files (default: dry run)
        #[arg(long)]
        write: bool,
        /// Add `markup: occlusion` to every unmarked file, including files
        /// with no legacy index. Use this when migrating a shipped library.
        #[arg(long)]
        mark: bool,
        /// One JSON object on stdout
        #[arg(long)]
        json: bool,
    },
    /// One-liner `import bind:target` → nested YAML `import:\n  bind: target`
    Imports {
        #[arg(value_name = "PATH", default_value = ".")]
        paths: Vec<PathBuf>,
        /// Actually rewrite files (default: dry run)
        #[arg(long)]
        write: bool,
        /// One JSON array on stdout
        #[arg(long)]
        json: bool,
    },
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

        /// Parse unmarked files using the legacy `[key](collection)` grammar.
        /// Explicit per-file `markup:` still wins; this flag is transitional.
        #[arg(long)]
        legacy_markup: bool,
    },
    /// Static contract cross-check (渐进式契约): drift, unknown types, misplaced tables
    Check {
        /// Program path (.mq.md)
        #[arg(value_name = "FILE")]
        file: PathBuf,

        /// Emit one JSON diagnostic per line on stdout (machine-readable; AI/MLSP 面)
        #[arg(long)]
        json: bool,

        /// Also check prose above `>` calls (implicit occlusion; fuzzy, off by default)
        #[arg(long)]
        implicit_labels: bool,

        /// Parse an unmarked file with the legacy grammar for migration review.
        #[arg(long)]
        legacy_markup: bool,
    },
    /// Occlusion label queries: pending stubs (built-in words) or keyword search
    Labels {
        /// Files or directories (recursively scans `*.mq.md`)
        #[arg(value_name = "PATH", default_value = ".")]
        paths: Vec<PathBuf>,

        /// Keywords (comma separated). Replaces the built-in pending set.
        #[arg(long, value_name = "WORDS")]
        grep: Option<String>,

        /// Union with the built-in groups instead of replacing them
        #[arg(long)]
        also_builtin: bool,

        /// Built-in group names, comma separated (default: pending = 未完成,假数据)
        #[arg(long, value_name = "GROUPS")]
        group: Option<String>,

        /// One JSON array on stdout
        #[arg(long)]
        json: bool,

        /// Print only the hit count
        #[arg(long)]
        count: bool,

        /// Debt ledger (JSON) for the ratchet: seed it, then only allow decreases
        #[arg(long, value_name = "FILE")]
        baseline: Option<PathBuf>,

        /// Fail (exit 1) when the hit count exceeds the baseline (`<TAB>` M8)
        #[arg(long)]
        fail_on_increase: bool,
    },
    /// Migrate legacy syntax to the purity forms (`<TAB>` M7)
    Migrate {
        #[command(subcommand)]
        action: MigrateAction,
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

        /// Exit non-zero when conflicts found (CI gate; default remains report-only)
        #[arg(long)]
        fail: bool,
    },
    /// List stale / deprecated / superseded knowledge
    Stale {
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        #[arg(short = 'o', long = "out", default_value = ".marqdo")]
        out: PathBuf,

        #[arg(long)]
        json: bool,

        /// Exit non-zero when stale items found (CI gate; default remains report-only)
        #[arg(long)]
        fail: bool,
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
    /// Verify code ↔ knowledge consistency (EKC structural; exit 1 only on errors)
    Verify {
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        #[arg(short = 'o', long = "out", default_value = ".marqdo")]
        out: PathBuf,

        #[arg(long)]
        json: bool,

        /// Also fail on warnings (does not change default success-when-no-errors)
        #[arg(long)]
        fail_on_warning: bool,
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
    /// Inspect stable project symbols by exact URI or display name
    Inspect {
        /// Symbol URI (`mq://…`) or declaration name
        #[arg(value_name = "SYMBOL")]
        symbol: String,
    },
    /// Resolve a new-style knowledge path in the import scope of a source file
    Resolve {
        /// `mq://…` URI or a value-path such as `facts.Person.birth_year`
        #[arg(value_name = "REFERENCE")]
        reference: String,

        /// Source file whose frontmatter imports supply the path's first segment
        #[arg(long, value_name = "FILE")]
        from: Option<PathBuf>,
    },
    /// Check or update the reviewed semantic target lock (`links.lock.json`)
    Lock {
        /// Replace the lock with the current deterministic semantic graph
        #[arg(long)]
        update: bool,
        /// Return exit code 3 when the lock is missing or differs
        #[arg(long)]
        strict: bool,
    },
    /// Show transitive reverse dependencies of a symbol, artifact, or source path
    Impact {
        #[arg(value_name = "SYMBOL_OR_PATH")]
        query: String,
    },
    /// Machine adjudication of semantic artifacts (M03): may_bind / may_publish
    Adjudicate {
        /// Artifact id, symbol id, or display name (omit to report all functions)
        #[arg(value_name = "ARTIFACT")]
        artifact: Option<String>,
        /// Policy: pure | business | static
        #[arg(long, default_value = "business")]
        policy: String,
        /// Exit 1 when any reported artifact has may_bind=false
        #[arg(long)]
        fail: bool,
    },
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

        #[arg(long, value_name = "FILE")]
        snapshot: Option<PathBuf>,
    },
    /// Capture live cell results for `view output --snapshot`
    Capture {
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        #[arg(short = 'o', long = "out", required = true)]
        out: PathBuf,
    },
    /// Read-only DocumentViewV1 (never executes). JSON to stdout or `--out`.
    Document {
        /// Project root (defaults to cwd / CLI path)
        #[arg(long)]
        project: Option<PathBuf>,

        /// Document path relative to project
        #[arg(long)]
        document: String,

        #[arg(long)]
        json: bool,

        /// Optional HTML wrapper for browser inspection (embed fragment only)
        #[arg(long, value_name = "FILE")]
        out: Option<PathBuf>,
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

/// Resolve the value-path spelling introduced for executable knowledge without
/// evaluating it. The import bind comes from the caller's source file, so
/// `math.pi` never degrades into a project-wide name guess.
fn resolve_knowledge_reference<'a>(
    index: &'a marqdo::sema::ProjectIndex,
    reference: &str,
    from: Option<&std::path::Path>,
) -> Result<&'a marqdo::sema::SymbolDecl> {
    if reference.starts_with("mq://") {
        return index
            .symbols()
            .iter()
            .find(|symbol| symbol.id.as_str() == reference)
            .ok_or_else(|| anyhow::anyhow!("unknown symbol URI `{reference}`"));
    }

    let expression = marqdo::parse::expr::parse_expr_prefer_var(reference)
        .map_err(|error| anyhow::anyhow!("invalid knowledge reference `{reference}`: {error}"))?;
    let marqdo::ast::Expr::Call(call) = expression else {
        anyhow::bail!("knowledge reference `{reference}` must be a dotted value path or mq:// URI");
    };
    if !call.args.is_empty() {
        anyhow::bail!(
            "knowledge resolve only binds a zero-argument path; `{reference}` has arguments"
        );
    }
    let path = call.path.ok_or_else(|| {
        anyhow::anyhow!("knowledge reference `{reference}` must start with an imported module bind")
    })?;
    if path.len() < 2 {
        anyhow::bail!("knowledge reference `{reference}` needs at least `module.symbol`");
    }
    let from = from.ok_or_else(|| {
        anyhow::anyhow!("`--from FILE` is required to resolve imported path `{reference}`")
    })?;
    let module = marqdo::load::load_module(from)?;
    let imported = module
        .imports
        .iter()
        .find(|item| item.bind == path[0])
        .ok_or_else(|| {
            anyhow::anyhow!("unknown import bind `{}` in {}", path[0], from.display())
        })?;
    let base = from.parent().unwrap_or_else(|| std::path::Path::new("."));
    let imported_path = marqdo::load::resolve_import(base, &imported.path)?;
    let imported_path = imported_path.canonicalize().unwrap_or(imported_path);
    let module_id = index
        .modules()
        .iter()
        .find_map(|(module_id, source)| {
            (source.canonicalize().unwrap_or_else(|_| source.clone()) == imported_path)
                .then_some(module_id)
        })
        .ok_or_else(|| {
            anyhow::anyhow!(
                "import `{}` resolves to {}, which is outside the indexed project",
                path[0],
                imported_path.display()
            )
        })?;
    let declaration_path = &path[1..];
    index
        .symbols()
        .iter()
        .find(|symbol| symbol.module == *module_id && symbol.declaration_path == declaration_path)
        .ok_or_else(|| anyhow::anyhow!("unknown knowledge path `{reference}`"))
}

/// Recursively collect `*.mq.md` under `path` (file → itself).
fn collect_mq_files(path: &std::path::Path) -> Result<Vec<std::path::PathBuf>> {
    fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        let Ok(rd) = std::fs::read_dir(dir) else {
            return;
        };
        for e in rd.flatten() {
            let p = e.path();
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if p.is_dir() {
                if matches!(
                    name,
                    "target" | "dist" | "node_modules" | ".git" | ".marqdo"
                ) {
                    continue;
                }
                walk(&p, out);
            } else if name.ends_with(".mq.md") {
                out.push(p);
            }
        }
    }
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }
    let mut out = Vec::new();
    walk(path, &mut out);
    out.sort();
    Ok(out)
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
            legacy_markup,
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
            if legacy_markup {
                marqdo::parse::expr::with_markup(marqdo::parse::expr::Markup::Classic, || {
                    marqdo::run_file(&path, &opts)
                })?;
            } else {
                marqdo::run_file(&path, &opts)?;
            }
            Ok(0)
        }
        Commands::Check {
            file,
            json,
            implicit_labels,
            legacy_markup,
        } => {
            let diags = if legacy_markup {
                marqdo::parse::expr::with_markup(marqdo::parse::expr::Markup::Classic, || {
                    marqdo::check::check_path_opts(&file, implicit_labels)
                })?
            } else {
                marqdo::check::check_path_opts(&file, implicit_labels)?
            };
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
        Commands::Labels {
            paths,
            grep,
            also_builtin,
            group,
            json,
            count,
            baseline,
            fail_on_increase,
        } => {
            let mut words: Vec<String> = match &grep {
                Some(spec) => marqdo::label::parse_words(spec),
                None => {
                    let groups: Vec<&str> = match &group {
                        Some(g) => g
                            .split(',')
                            .map(|s| s.trim())
                            .filter(|s| !s.is_empty())
                            .collect(),
                        None => marqdo::label::PENDING_GROUPS.to_vec(),
                    };
                    marqdo::label::group_words(&groups)
                }
            };
            if also_builtin {
                let groups: Vec<&str> = match &group {
                    Some(g) => g
                        .split(',')
                        .map(|s| s.trim())
                        .filter(|s| !s.is_empty())
                        .collect(),
                    None => marqdo::label::PENDING_GROUPS.to_vec(),
                };
                for w in marqdo::label::group_words(&groups) {
                    if !words.contains(&w) {
                        words.push(w);
                    }
                }
            }

            // Accept the documented shapes `labels pending` / `labels search`
            // as no-op words (the flags carry the real intent).
            let paths: Vec<&std::path::PathBuf> = paths
                .iter()
                .filter(|p| {
                    let s = p.to_string_lossy();
                    !(s == "pending" || s == "search")
                })
                .collect();
            let mut files: Vec<std::path::PathBuf> = Vec::new();
            for p in &paths {
                files.extend(collect_mq_files(p)?);
            }
            files.sort();
            files.dedup();
            let mut rows: Vec<(String, u32, String, String, Vec<String>, bool)> = Vec::new();
            for f in &files {
                let module = match marqdo::load::load_module(f) {
                    Ok(m) => m,
                    Err(_) => continue, // unparsable files are `check`'s problem, not ours
                };
                for hit in marqdo::label::search(&module, &words) {
                    rows.push((
                        f.display().to_string(),
                        hit.site.line,
                        hit.site.unit.clone(),
                        hit.site.label.clone(),
                        hit.words.clone(),
                        hit.site.has_payload,
                    ));
                }
            }

            if json {
                let arr: Vec<serde_json::Value> = rows
                    .iter()
                    .map(|(file, line, unit, label, words, has_payload)| {
                        serde_json::json!({
                            "file": file, "line": line, "unit": unit,
                            "label": label, "words": words, "has_payload": has_payload,
                        })
                    })
                    .collect();
                println!("{}", serde_json::Value::Array(arr));
            } else if count {
                println!("{}", rows.len());
            } else {
                for (file, line, unit, label, words, _) in &rows {
                    println!("{file}:{line}  [{unit}] {label}  ← {}", words.join(","));
                }
                println!(
                    "label search: {} 命中（{} 文件，词={:?}）",
                    rows.len(),
                    files.len(),
                    words
                );
            }

            // `<TAB>` M8: ratchet — debt may shrink, never grow.
            let current = rows.len();
            let mut violation = false;
            if let Some(ledger) = &baseline {
                let prior = std::fs::read_to_string(ledger)
                    .ok()
                    .and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
                    .and_then(|v| v.get("count").and_then(|c| c.as_u64()))
                    .map(|n| n as usize);
                match prior {
                    None => {
                        let body = serde_json::json!({"count": current, "words": words});
                        if let Some(dir) = ledger.parent() {
                            if !dir.as_os_str().is_empty() {
                                let _ = std::fs::create_dir_all(dir);
                            }
                        }
                        std::fs::write(ledger, format!("{body}\n"))?;
                        if !json && !count {
                            println!("baseline: 已建立 {current}（{}）", ledger.display());
                        }
                    }
                    Some(prior) if current > prior => {
                        violation = true;
                        if !count {
                            println!(
                                "baseline: {current} > {prior} —— 未完成项增加了 {}（{}）",
                                current - prior,
                                ledger.display()
                            );
                        }
                    }
                    Some(prior) => {
                        if current < prior || std::fs::read_to_string(ledger).is_err() {
                            let body = serde_json::json!({"count": current, "words": words});
                            std::fs::write(ledger, format!("{body}\n"))?;
                            if !json && !count {
                                println!(
                                    "baseline: {prior} → {current}（棘轮收紧，{}）",
                                    ledger.display()
                                );
                            }
                        }
                    }
                }
            }
            if violation && fail_on_increase {
                return Ok(1);
            }
            Ok(0)
        }
        Commands::Migrate { action } => match action {
            MigrateAction::Labels {
                paths,
                write,
                mark,
                json,
            } => {
                let mut reports = Vec::new();
                for p in &paths {
                    reports.extend(marqdo::migrate::migrate_path_with_options(p, write, mark)?);
                }
                let (mut idx, mut loops, mut marked, mut applied) =
                    (0usize, 0usize, 0usize, 0usize);
                for r in &reports {
                    idx += r.indexes;
                    loops += r.loops;
                    marked += usize::from(r.mode_marked);
                    applied += usize::from(r.applied);
                }
                if json {
                    let arr: Vec<serde_json::Value> = reports
                        .iter()
                        .filter(|r| {
                            r.indexes > 0 || r.loops > 0 || r.mode_marked || r.note.is_some()
                        })
                        .map(|r| {
                            serde_json::json!({
                                "file": r.file.display().to_string(),
                                "indexes": r.indexes,
                                "loops": r.loops,
                                "mode_marked": r.mode_marked,
                                "applied": r.applied,
                                "note": r.note,
                            })
                        })
                        .collect();
                    println!("{}", serde_json::Value::Array(arr));
                } else {
                    for r in &reports {
                        if r.indexes == 0 && r.loops == 0 && !r.mode_marked {
                            continue;
                        }
                        let state = if r.applied {
                            "written"
                        } else if r.note.is_some() {
                            "skipped"
                        } else {
                            "would change"
                        };
                        println!(
                            "{}  indexes={} loops={} mode={} {}{}",
                            r.file.display(),
                            r.indexes,
                            r.loops,
                            if r.mode_marked {
                                "occlusion"
                            } else {
                                "unchanged"
                            },
                            state,
                            r.note
                                .as_ref()
                                .map(|n| format!(" ({n})"))
                                .unwrap_or_default()
                        );
                    }
                    println!(
                        "migrate labels: {idx} index, {loops} loop, {marked} explicit mode, {applied} file written{}",
                        if write {
                            ""
                        } else {
                            " (dry run; pass --write)"
                        }
                    );
                }
                Ok(0)
            }
            MigrateAction::Imports { paths, write, json } => {
                let mut reports = Vec::new();
                for p in &paths {
                    reports.extend(marqdo::migrate::migrate_imports_path(p, write)?);
                }
                let (mut bindings, mut applied, mut problems) = (0usize, 0usize, 0usize);
                for r in &reports {
                    bindings += r.bindings;
                    applied += usize::from(r.applied);
                    if r.note
                        .as_ref()
                        .is_some_and(|n| n != "no legacy import one-liners")
                        && r.bindings == 0
                    {
                        problems += 1;
                    }
                }
                if json {
                    let arr: Vec<serde_json::Value> = reports
                        .iter()
                        .filter(|r| {
                            r.bindings > 0
                                || r.note
                                    .as_ref()
                                    .is_some_and(|n| n != "no legacy import one-liners")
                        })
                        .map(|r| {
                            serde_json::json!({
                                "file": r.file.display().to_string(),
                                "bindings": r.bindings,
                                "applied": r.applied,
                                "note": r.note,
                            })
                        })
                        .collect();
                    println!("{}", serde_json::Value::Array(arr));
                } else {
                    for r in &reports {
                        if r.bindings == 0
                            && r.note
                                .as_ref()
                                .is_some_and(|n| n == "no legacy import one-liners")
                        {
                            continue;
                        }
                        if r.bindings == 0 {
                            println!(
                                "{}  skipped ({})",
                                r.file.display(),
                                r.note.as_deref().unwrap_or("?")
                            );
                            continue;
                        }
                        let state = if r.applied {
                            "written"
                        } else if r.note.is_some() {
                            "skipped"
                        } else {
                            "would change"
                        };
                        println!(
                            "{}  bindings={} {}{}",
                            r.file.display(),
                            r.bindings,
                            state,
                            r.note
                                .as_ref()
                                .map(|n| format!(" ({n})"))
                                .unwrap_or_default()
                        );
                    }
                    println!(
                        "migrate imports: {bindings} binding(s), {applied} file written, {problems} problem(s){}",
                        if write { "" } else { " (dry run; pass --write)" }
                    );
                }
                Ok(0)
            }
        },
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
                snapshot,
            }) => {
                write_static(OutputOptions {
                    path: out_path.or(path).unwrap_or_else(|| PathBuf::from(".")),
                    out_dir: out,
                    no_exec,
                    snapshot,
                })?;
                Ok(0)
            }
            Some(ViewAction::Capture {
                path: out_path,
                out,
            }) => {
                write_capture(CaptureOptions {
                    path: out_path.or(path).unwrap_or_else(|| PathBuf::from(".")),
                    out,
                })?;
                Ok(0)
            }
            Some(ViewAction::Document {
                project,
                document,
                json,
                out,
            }) => {
                let project = project
                    .or(path)
                    .unwrap_or_else(|| PathBuf::from("."));
                let req = marqdo::workbench::DocumentViewRequest {
                    project,
                    document,
                    revision: None,
                    generation: None,
                    node_id: None,
                    include: vec![
                        "fragment".into(),
                        "outline".into(),
                        "nodes".into(),
                        "refs".into(),
                        "diagnostics".into(),
                    ],
                };
                let view = marqdo::workbench::build_document_view(&req).map_err(|e| {
                    anyhow::anyhow!("{}", e)
                })?;
                if let Some(out_path) = out {
                    let fragment = view
                        .pointer("/render/fragment")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let title = view
                        .pointer("/document/title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("DocumentView");
                    let html = format!(
                        "<!doctype html><html><head><meta charset=\"utf-8\"/><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"/><title>{title}</title><style>:root{{--mq-bg:#f7f5f0;--mq-text:#1a1a1a;--mq-muted:#5c5c5c;--mq-border:#d0cbc2;--mq-accent:#0b5fff;--mq-font-body:\"Iowan Old Style\",\"Palatino Linotype\",Palatino,serif;--mq-font-code:ui-monospace,Consolas,monospace}}body{{margin:0;background:var(--mq-bg);color:var(--mq-text);font:16px/1.75 var(--mq-font-body)}}.wrap{{max-width:46rem;margin:0 auto;padding:1.5rem}} .mq-document a{{color:var(--mq-accent)}}</style></head><body><div class=\"wrap\">{fragment}</div></body></html>"
                    );
                    std::fs::write(&out_path, html)?;
                    if json {
                        println!("{}", serde_json::to_string_pretty(&view)?);
                    } else {
                        println!("wrote {}", out_path.display());
                    }
                } else {
                    println!("{}", serde_json::to_string_pretty(&view)?);
                }
                let _ = json;
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
                write_knowledge(KnowledgeOptions { path, out_dir: out })?;
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
                    write_knowledge(KnowledgeOptions { path, out_dir: out })?;
                    Ok(0)
                }
                KnowledgeAction::Inspect { symbol } => {
                    // A supplied directory is an explicit compilation scope;
                    // a supplied file inherits the nearest project boundary.
                    let project = if path.is_dir() {
                        marqdo::project::ProjectRoot::new(&path)?
                    } else {
                        marqdo::project::ProjectRoot::discover(&path)?
                    };
                    let index = marqdo::sema::ProjectIndex::build(project)?;
                    let candidates: Vec<_> = if symbol.starts_with("mq://") {
                        index
                            .symbols()
                            .iter()
                            .filter(|item| item.id.as_str() == symbol)
                            .collect()
                    } else {
                        index.symbols_named(&symbol)
                    };
                    if candidates.is_empty() {
                        anyhow::bail!(
                            "no semantic symbol matching `{symbol}` in project {}",
                            index.root().path().display()
                        );
                    }
                    if json {
                        println!(
                            "{}",
                            serde_json::json!({
                                "project": index.project_id(),
                                "query": symbol,
                                "ambiguous": candidates.len() > 1,
                                "symbols": candidates.iter().map(|item| serde_json::json!({
                                    "id": item.id.as_str(),
                                    "module": item.module.as_str(),
                                    "kind": item.kind.as_str(),
                                    "callable": item.callable.map(|kind| kind.as_str()),
                                    "name": item.name,
                                    "declaration_path": item.declaration_path,
                                    "source": {
                                        "file": item.source,
                                        "line": item.line,
                                    },
                                })).collect::<Vec<_>>(),
                            })
                        );
                    } else {
                        for item in &candidates {
                            println!(
                                "{}\n  {}{} {} · module {}\n  {}:{}",
                                item.id,
                                item.kind.as_str(),
                                item.callable
                                    .map(|kind| format!(" ({})", kind.as_str()))
                                    .unwrap_or_default(),
                                item.declaration_path.join("."),
                                item.module,
                                item.source.display(),
                                item.line,
                            );
                        }
                        if candidates.len() > 1 {
                            println!(
                                "ambiguous: {} symbols named `{symbol}`; use one full mq:// URI",
                                candidates.len()
                            );
                        }
                    }
                    Ok(0)
                }
                KnowledgeAction::Resolve { reference, from } => {
                    let from = from
                        .as_deref()
                        .or_else(|| path.is_file().then_some(path.as_path()));
                    let index = marqdo::sema::ProjectIndex::build(if path.is_dir() {
                        marqdo::project::ProjectRoot::new(&path)?
                    } else {
                        marqdo::project::ProjectRoot::discover(&path)?
                    })?;
                    let symbol = resolve_knowledge_reference(&index, &reference, from)?;
                    let instance_only =
                        symbol.callable == Some(marqdo::sema::CallableKind::InstanceMethod);
                    if json {
                        println!(
                            "{}",
                            serde_json::json!({
                                "reference": reference,
                                "resolution": if instance_only { "instance_only" } else { "symbol" },
                                "symbol": {
                                    "id": symbol.id.as_str(),
                                    "module": symbol.module.as_str(),
                                    "kind": symbol.kind.as_str(),
                                    "callable": symbol.callable.map(|kind| kind.as_str()),
                                    "declaration_path": symbol.declaration_path,
                                    "source": { "file": symbol.source, "line": symbol.line },
                                },
                            })
                        );
                    } else {
                        println!("{}", symbol.id);
                        println!("  {}", symbol.declaration_path.join("."));
                        println!("  {}:{}", symbol.source.display(), symbol.line);
                        if instance_only {
                            println!("  instance_only: call through an instance receiver");
                        }
                    }
                    Ok(if instance_only { 1 } else { 0 })
                }
                KnowledgeAction::Lock { update, strict } => {
                    let semantic_root = if path.is_file() {
                        path.parent()
                            .unwrap_or_else(|| std::path::Path::new("."))
                            .to_path_buf()
                    } else {
                        path.clone()
                    };
                    let compilation =
                        marqdo::sema::compile_project(&marqdo::sema::CompileOptions {
                            project_root: semantic_root,
                            entry: path.is_file().then(|| path.clone()),
                            lock_mode: if strict {
                                marqdo::sema::LockMode::Strict
                            } else {
                                marqdo::sema::LockMode::Check
                            },
                        })?;
                    if update {
                        marqdo::sema::update_links_lock(&out, &compilation.semantics)?;
                        let body = serde_json::json!({"schema_version": 1, "status": "updated", "path": out.join("links.lock.json")});
                        if json {
                            println!("{body}");
                        } else {
                            println!(
                                "knowledge lock updated: {}",
                                out.join("links.lock.json").display()
                            );
                        }
                        Ok(0)
                    } else {
                        let report = marqdo::sema::check_links_lock(&out, &compilation.semantics)?;
                        if json {
                            println!("{}", report.to_json());
                        } else {
                            println!("knowledge lock: {}", report.status);
                            for id in report
                                .missing
                                .iter()
                                .chain(report.changed.iter())
                                .chain(report.removed.iter())
                            {
                                println!("  {id}");
                            }
                        }
                        Ok(if strict && !report.is_clean() { 3 } else { 0 })
                    }
                }
                KnowledgeAction::Impact { query } => {
                    let semantic_root = if path.is_file() {
                        path.parent()
                            .unwrap_or_else(|| std::path::Path::new("."))
                            .to_path_buf()
                    } else {
                        path.clone()
                    };
                    let compilation =
                        marqdo::sema::compile_project(&marqdo::sema::CompileOptions {
                            project_root: semantic_root,
                            entry: path.is_file().then(|| path.clone()),
                            lock_mode: marqdo::sema::LockMode::Off,
                        })?;
                    let report = marqdo::sema::semantic_impact(&compilation.semantics, &query);
                    if json {
                        println!("{report}");
                    } else {
                        for item in report["affected"].as_array().into_iter().flatten() {
                            println!(
                                "{}  {}",
                                item["artifact_id"].as_str().unwrap_or(""),
                                item["display_name"].as_str().unwrap_or("")
                            );
                        }
                    }
                    Ok(if report["resolved"].as_bool() == Some(true) {
                        0
                    } else {
                        1
                    })
                }
                KnowledgeAction::Adjudicate {
                    artifact,
                    policy,
                    fail,
                } => {
                    let semantic_root = if path.is_file() {
                        path.parent()
                            .unwrap_or_else(|| std::path::Path::new("."))
                            .to_path_buf()
                    } else {
                        path.clone()
                    };
                    let report = marqdo::mlsp::adjudicate_project(
                        &semantic_root,
                        artifact.as_deref(),
                        &policy,
                    )
                    .map_err(|e| anyhow::anyhow!("{}", e))?;
                    if json {
                        println!("{report}");
                    } else {
                        println!(
                            "knowledge adjudicate policy={} may_bind_all={} count={}",
                            policy,
                            report["may_bind_all"],
                            report["count"]
                        );
                        for item in report["reports"].as_array().into_iter().flatten() {
                            println!(
                                "  {} evidence={} may_bind={} may_publish={}",
                                item["artifact_id"].as_str().unwrap_or(""),
                                item["verification"]["evidence"].as_str().unwrap_or(""),
                                item["may_bind"],
                                item["may_publish"]
                            );
                        }
                    }
                    Ok(
                        if fail && report["may_bind_all"].as_bool() != Some(true) {
                            1
                        } else {
                            0
                        },
                    )
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
                    Ok(
                        if pf.create_allowed || pf.decision == "REUSE" || pf.decision == "ADAPT" {
                            0
                        } else {
                            2
                        },
                    )
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
        Commands::Conflicts {
            path,
            out,
            json,
            fail,
        } => {
            let path = path.unwrap_or_else(|| PathBuf::from("."));
            let graph = ensure_compiled(&path, &out)?;
            let items = find_conflicts(&graph);
            if json {
                println!("{}", conflicts_json(&items));
            } else {
                print!("{}", format_conflicts_text(&items));
            }
            Ok(if fail && !items.is_empty() { 1 } else { 0 })
        }
        Commands::Stale {
            path,
            out,
            json,
            fail,
        } => {
            let path = path.unwrap_or_else(|| PathBuf::from("."));
            let graph = ensure_compiled(&path, &out)?;
            let items = find_stale(&graph, &path);
            if json {
                println!("{}", stale_json(&items));
            } else {
                print!("{}", format_stale_text(&items));
            }
            Ok(if fail && !items.is_empty() { 1 } else { 0 })
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
        Commands::Verify {
            path,
            out,
            json,
            fail_on_warning,
        } => {
            let path = path.unwrap_or_else(|| PathBuf::from("."));
            let graph = ensure_compiled(&path, &out)?;
            let issues = verify(&graph);
            if json {
                println!("{}", verify_json(&issues));
            } else {
                print!("{}", format_verify_text(&issues));
            }
            let errors = issues.iter().filter(|i| i.severity == "error").count();
            let warnings = issues.iter().filter(|i| i.severity == "warning").count();
            Ok(if errors > 0 || (fail_on_warning && warnings > 0) {
                1
            } else {
                0
            })
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
