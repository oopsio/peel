mod ast;
mod lexer;
mod parser;
mod visitor;
mod engine;

use clap::Parser;
use std::fs;
use std::process;
use colored::*;
use crate::parser::Parser as PeelParser;
use crate::engine::{LinterEngine, Level};
use crate::visitor::Visitor;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Files to lint
    files: Vec<String>,

    /// Show only errors
    #[arg(short, long)]
    errors_only: bool,
}

fn main() {
    let args = Args::parse();

    if args.files.is_empty() {
        println!("{}", "No files provided to lint.".yellow());
        process::exit(0);
    }

    let mut total_issues = 0;

    for file_path in args.files {
        let content = match fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("{}: {}: {}", "Error".red().bold(), file_path, e);
                continue;
            }
        };

        let mut parser = PeelParser::new(&content, &file_path);
        match parser.parse_module() {
            Ok(module) => {
                let mut engine = LinterEngine::new(&content);
                engine.visit_module(&module);

                if !engine.errors.is_empty() {
                    println!("\n{} in {}:", "Lint Issues".bold(), file_path.cyan());
                    for err in &engine.errors {
                        if args.errors_only && !matches!(err.level, Level::Error) {
                            continue;
                        }

                        let level_str = match err.level {
                            Level::Error => "error".red().bold(),
                            Level::Warning => "warning".yellow().bold(),
                            Level::Info => "info".blue().bold(),
                        };

                        println!(
                            "  [{}] {}:{}: {} ({})",
                            level_str,
                            file_path,
                            err.line + 1,
                            err.message,
                            err.id.dimmed()
                        );
                        total_issues += 1;
                    }
                } else {
                    println!("{}: {} {}", "Clean".green().bold(), file_path, "is lint-free!".dimmed());
                }
            }
            Err(e) => {
                eprintln!("{}: {}: {}", "Parse Error".red().bold(), file_path, e);
            }
        }
    }

    if total_issues > 0 {
        println!("\n{} total issues found.", total_issues);
    } else {
        println!("\n{}", "All checks passed!".green().bold());
    }
}
