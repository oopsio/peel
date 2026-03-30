
use peel::parser::Parser;
use peel::checker::Checker;
use peel::runtime::Interpreter;
use peel::stdlib::register_stdlib;
use peel::lsp::start_lsp;
use peel::ast::types::PeelType;
use anyhow::Result;
use clap::{Parser as ClapParser, Subcommand};
use std::fs;

#[derive(ClapParser)]
#[command(name = "peel")]
#[command(about = "The Peel Programming Language", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a .pel file
    Run {
        /// Path to the file
        path: String,
    },
    /// Parse and show the AST
    Parse {
        /// Path to the file
        path: String,
    },
    /// Start the Language Server Protocol (LSP) server
    Lsp,
}

#[tokio::main]
async fn main() {
    if let Err(e) = run_cli().await {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

async fn run_cli() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { path } => {
            let source = fs::read_to_string(&path)?;
            let mut parser = Parser::new(&source, &path);
            let module = parser.parse_module()?;

            // 1. Type Check
            let mut checker = Checker::new();
            checker.define("fmt", PeelType::Unknown, false);
            checker.define("time", PeelType::Unknown, false);
            checker.define("http", PeelType::Unknown, false);
            checker.define("fs", PeelType::Unknown, false);
            checker.define("console", PeelType::Unknown, false);
            checker.define("Math", PeelType::Unknown, false);
            checker.define("JSON", PeelType::Unknown, false);
            checker.define("sqlite", PeelType::Unknown, false);
            checker.check_module(&module)?;

            // 2. Initialize Runtime & StdLib
            let mut interpreter = Interpreter::new();
            interpreter.current_path = std::path::Path::new(&path)
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .to_path_buf();
            register_stdlib(interpreter.env.clone(), interpreter.methods.clone());

            // 3. Execute
            for stmt in &module.stmts {
                interpreter.eval_stmt(stmt).await?;
            }
        }
        Commands::Parse { path } => {
            let source = fs::read_to_string(&path)?;
            let mut parser = Parser::new(&source, &path);
            let module = parser.parse_module().map_err(|e| anyhow::anyhow!(e))?;
            println!("{:#?}", module);
        }
        Commands::Lsp => {
            start_lsp().await;
        }
    }

    Ok(())
}
