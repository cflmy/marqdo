use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};

use marqdo::RunOptions;

#[derive(Parser, Debug)]
#[command(name = "marqdo", version, about = "Marqdo interpreter — run .mq.md programs")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Execute a Marqdo source file (defaults to ./index.mq.md)
    Run {
        /// Path to a `.mq.md` file
        #[arg(value_name = "FILE")]
        file: Option<PathBuf>,

        /// Dump line classification
        #[arg(long)]
        dump_lines: bool,

        /// Dump tokens (placeholder until lexer fills in)
        #[arg(long)]
        dump_tokens: bool,

        /// Dump AST (placeholder until parser exists)
        #[arg(long)]
        dump_ast: bool,

        /// Dump semantic info (placeholder)
        #[arg(long)]
        dump_sema: bool,

        /// Trace evaluation (placeholder)
        #[arg(long)]
        trace_eval: bool,

        /// Enable all dumps
        #[arg(long)]
        dump_all: bool,
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
            dump_lines,
            dump_tokens,
            dump_ast,
            dump_sema,
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
                    trace_eval,
                }
            };
            // dump_all already set; individual flags merge if not dump_all
            if !dump_all {
                opts.dump_lines = dump_lines;
                opts.dump_tokens = dump_tokens;
                opts.dump_ast = dump_ast;
                opts.dump_sema = dump_sema;
                opts.trace_eval = trace_eval;
            }
            marqdo::run_file(&path, &opts)?;
            Ok(0)
        }
    }
}
