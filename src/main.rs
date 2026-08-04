use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};

use marqdo::view::{serve, ViewOptions};
use marqdo::{Backend, RunOptions};

#[derive(Parser, Debug)]
#[command(name = "marqdo", version, about = "Marqdo interpreter — run and view .mq.md programs")]
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
    },
    /// Browse `.mq.md` structure + execution in a local webpage
    View {
        #[arg(value_name = "PATH")]
        path: Option<PathBuf>,

        #[arg(long, default_value = "127.0.0.1")]
        host: String,

        #[arg(long, default_value_t = 7429)]
        port: u16,

        #[arg(long)]
        no_open: bool,
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
        } => {
            let path = file.unwrap_or_else(|| PathBuf::from("index.mq.md"));
            let mut opts = if dump_all {
                RunOptions::dump_all()
            } else {
                RunOptions {
                    dump_lines,
                    dump_tokens,
                    dump_ast,
                    dump_sema,
                    dump_bytecode,
                    trace_eval,
                    backend: backend.into(),
                }
            };
            if dump_all {
                opts.backend = backend.into();
            }
            marqdo::run_file(&path, &opts)?;
            Ok(0)
        }
        Commands::View {
            path,
            host,
            port,
            no_open,
        } => {
            serve(ViewOptions {
                path: path.unwrap_or_else(|| PathBuf::from(".")),
                host,
                port,
                open_browser: !no_open,
            })?;
            Ok(0)
        }
    }
}
