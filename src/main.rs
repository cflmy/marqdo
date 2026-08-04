use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};

use marqdo::catalog::{write_catalog, CatalogOptions};
use marqdo::input_feed::load_stdin_file;
use marqdo::view::{serve, write_static, OutputOptions, ViewOptions};
use marqdo::{Backend, RunOptions};

#[derive(Parser, Debug)]
#[command(name = "marqdo", version, about = "Marqdo interpreter — run, view, and catalog .mq.md")]
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
    },
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
    match try_main() {
        Ok(code) => ExitCode::from(code as u8),
        Err(err) => {
            eprintln!("error: {err:#}");
            ExitCode::from(1)
        }
    }
}

fn try_main() -> Result<i32> {
    let cli = Cli::parse();
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
        } => {
            let path = file.unwrap_or_else(|| PathBuf::from("index.mq.md"));
            let stdin_lines = match stdin_file {
                Some(p) => load_stdin_file(&p)?,
                None => Vec::new(),
            };
            let mut opts = if dump_all {
                RunOptions {
                    stdin_lines: stdin_lines.clone(),
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
                }
            };
            if dump_all {
                opts.backend = backend.into();
            }
            marqdo::run_file(&path, &opts)?;
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
        Commands::Catalog { path, out } | Commands::Sync { path, out } => {
            write_catalog(CatalogOptions {
                path: path.unwrap_or_else(|| PathBuf::from(".")),
                out_dir: out,
            })?;
            Ok(0)
        }
    }
}
