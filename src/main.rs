use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::Result;
use clap::{Parser, Subcommand};

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
        Commands::Run { file } => {
            let path = file.unwrap_or_else(|| PathBuf::from("index.mq.md"));
            marqdo::run_file(&path)?;
            Ok(0)
        }
    }
}
