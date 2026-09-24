use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};

use marqdo::catalog::{write_catalog, CatalogOptions};
use marqdo::diagnostics::{Diagnostic, Span};
use marqdo::ext_cli::{add_ext, list_ext, remove_ext};
use marqdo::input_feed::load_stdin_file;
use marqdo::view::{serve, serve_debug, write_static, DebugOptions, OutputOptions, ViewOptions};
use marqdo::{Backend, RunOptions};

#[derive(Parser, Debug)]
#[command(name = "marqdo", version, about = "Marqdo interpreter — run, view, debug, catalog, and ext")]
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
    /// Generate OKF-compatible catalog YAML + module pages
    Catalog {
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        #[arg(short = 'o', long = "out", default_value = ".marqdo")]
        out: PathBuf,
    },
    /// Alias for `catalog`
    Sync {
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        #[arg(short = 'o', long = "out", default_value = ".marqdo")]
        out: PathBuf,
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
            let mut opts = if dump_all {
                RunOptions {
                    stdin_lines: stdin_lines.clone(),
                    emit_result: emit_result.clone(),
                    binds: bind_pairs.clone(),
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
        Commands::Catalog { path, out } | Commands::Sync { path, out } => {
            write_catalog(CatalogOptions {
                path: path.unwrap_or_else(|| PathBuf::from(".")),
                out_dir: out,
            })?;
            Ok(0)
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
