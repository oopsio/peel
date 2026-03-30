mod lexer;
mod ast;
mod parser;
mod checker;
mod runtime;
mod stdlib;

use clap::{Parser as ClapParser, Subcommand};
use crate::parser::Parser;
use std::fs;
use anyhow::Result;

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
            let mut checker = crate::checker::Checker::new();
            checker.define("fmt", crate::ast::types::PeelType::Unknown, false);
            checker.define("time", crate::ast::types::PeelType::Unknown, false);
            checker.define("http", crate::ast::types::PeelType::Unknown, false);
            checker.define("fs", crate::ast::types::PeelType::Unknown, false);
            checker.define("console", crate::ast::types::PeelType::Unknown, false);
            checker.define("Math", crate::ast::types::PeelType::Unknown, false);
            checker.define("JSON", crate::ast::types::PeelType::Unknown, false);
            checker.check_module(&module)?;

            // 2. Initialize Runtime & StdLib
            let mut interpreter = crate::runtime::Interpreter::new();
            interpreter.current_path = std::path::Path::new(&path).parent().unwrap_or(std::path::Path::new(".")).to_path_buf();
            crate::stdlib::register_stdlib(interpreter.env.clone(), interpreter.methods.clone());

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
    }

    Ok(())
}
