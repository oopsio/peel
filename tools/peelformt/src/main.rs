mod ast;
mod formatter;
mod lexer;
mod parser;

use crate::formatter::Formatter;
use crate::parser::Parser;
use anyhow::{Context, Result, anyhow};
use clap::Parser as ClapParser;
use std::path::PathBuf;

#[derive(ClapParser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(help = "The peel file to format")]
    file: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let source = std::fs::read_to_string(&args.file)
        .with_context(|| format!("Failed to read {:?}", args.file))?;
    let mut parser = Parser::new(&source, args.file.to_str().unwrap_or("unknown"));
    let module = parser
        .parse_module()
        .map_err(|e| anyhow!("Parse error: {}", e))?;
    let mut formatter = Formatter::new();
    formatter.format_module(&module);

    // Always write back to the file
    std::fs::write(&args.file, formatter.output)?;
    println!("Formatted {:?}", args.file);
    Ok(())
}
