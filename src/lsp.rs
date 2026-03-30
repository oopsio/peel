use regex::Regex;
use std::collections::HashMap;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use crate::ast::types::PeelType;
use crate::checker::Checker;
use crate::parser::Parser;

pub struct Backend {
    client: Client,
    document_map: Mutex<HashMap<String, String>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![".".to_string()]),
                    work_done_progress_options: Default::default(),
                    all_commit_characters: None,
                    completion_item: None,
                }),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Peel language server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text;

        {
            let mut map = self.document_map.lock().await;
            map.insert(uri.to_string(), text.clone());
        }

        self.publish_diagnostics(uri, text).await;
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = if let Some(change) = params.content_changes.pop() {
            change.text
        } else {
            return;
        };

        {
            let mut map = self.document_map.lock().await;
            map.insert(uri.to_string(), text.clone());
        }

        self.publish_diagnostics(uri, text).await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let pos = params.text_document_position_params.position;

        let (word, doc_text) = {
            let map = self.document_map.lock().await;
            if let Some(text) = map.get(uri.as_str()) {
                if let Some(line) = text.lines().nth(pos.line as usize) {
                    (
                        Self::extract_word(line, pos.character as usize),
                        text.clone(),
                    )
                } else {
                    (String::new(), String::new())
                }
            } else {
                (String::new(), String::new())
            }
        };

        if word.is_empty() {
            return Ok(None);
        }

        let mut desc = Self::get_hover_description(&word);
        if desc.is_empty() {
            let re = Regex::new(&format!(
                r"(?m)^[\t ]*fn\s+{}\s*\([^)]*\)(?:\s*->\s*[^{{]+)?",
                word
            ))
            .unwrap();
            if let Some(caps) = re.captures(&doc_text) {
                let decl = caps.get(0).unwrap().as_str().trim();
                desc = format!("```peel\n{}\n```", decl);
            }
        }

        if desc.is_empty() {
            return Ok(None);
        }

        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: desc,
            }),
            range: None,
        }))
    }

    async fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let keywords = vec![
            "let", "mut", "fn", "async", "await", "match", "if", "else", "return", "import",
            "export", "extern", "struct", "impl", "true", "false", "Ok", "Err", "Some", "None",
        ];

        let mut items = Vec::new();
        for k in keywords {
            items.push(CompletionItem {
                label: k.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                ..Default::default()
            });
        }

        // Add basic types
        let types = vec!["int", "float", "string", "bool", "Option", "Result"];
        for t in types {
            items.push(CompletionItem {
                label: t.to_string(),
                kind: Some(CompletionItemKind::CLASS),
                ..Default::default()
            });
        }

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn code_action(&self, _params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        Ok(Some(vec![]))
    }
}

impl Backend {
    async fn publish_diagnostics(&self, uri: Url, text: String) {
        let mut diagnostics = Vec::new();

        // Run parser
        let path = uri.path();
        let mut parser = Parser::new(&text, path);
        match parser.parse_module() {
            Ok(module) => {
                // Run checker
                let mut checker = Checker::new();
                checker.define("fmt", PeelType::Unknown, false);
                checker.define("time", PeelType::Unknown, false);
                checker.define("http", PeelType::Unknown, false);
                checker.define("fs", PeelType::Unknown, false);
                checker.define("console", PeelType::Unknown, false);
                checker.define("Math", PeelType::Unknown, false);
                checker.define("JSON", PeelType::Unknown, false);

                if let Err(e) = checker.check_module(&module) {
                    diagnostics
                        .push(self.create_diagnostic(&e.to_string(), DiagnosticSeverity::ERROR));
                }
            }
            Err(e) => {
                diagnostics.push(self.create_diagnostic(&e.to_string(), DiagnosticSeverity::ERROR));
            }
        }

        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }

    fn create_diagnostic(&self, err_msg: &str, severity: DiagnosticSeverity) -> Diagnostic {
        // Attempt to extract line and column from Peel's error output
        // Example: --> g:/peel/src/main.pel 10:5
        let re = Regex::new(r"(\d+):(\d+)").unwrap();

        let (mut line, mut col) = (0u32, 0u32);
        if let Some(caps) = re.captures(err_msg) {
            line = caps
                .get(1)
                .map_or(1, |m| m.as_str().parse::<u32>().unwrap_or(1))
                .saturating_sub(1);
            col = caps
                .get(2)
                .map_or(1, |m| m.as_str().parse::<u32>().unwrap_or(1))
                .saturating_sub(1);
        }

        Diagnostic {
            range: Range {
                start: Position {
                    line,
                    character: col,
                },
                end: Position {
                    line,
                    character: col + 1,
                }, // Simple 1-char wide diagnostic
            },
            severity: Some(severity),
            code: None,
            code_description: None,
            source: Some("peel-lsp".to_string()),
            message: self.clean_message(err_msg),
            related_information: None,
            tags: None,
            data: None,
        }
    }

    fn clean_message(&self, message: &str) -> String {
        // Remove ANSI color codes
        let re = Regex::new(r"\x1b\[[0-9;]*m").unwrap();
        let cleaned = re.replace_all(message, "").to_string();

        // We can just split by newline and take the first line + the actual error parts if needed,
        // but since Peel errors are pretty descriptive, returning raw cleaned is good.
        cleaned
    }

    fn extract_word(line: &str, char_pos: usize) -> String {
        let mut start = char_pos;
        let mut end = char_pos;
        let chars: Vec<char> = line.chars().collect();
        if char_pos >= chars.len() {
            return String::new();
        }

        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }
        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }
        chars[start..end].iter().collect()
    }

    fn get_hover_description(word: &str) -> String {
        let (kind, desc) = match word {
            "let" => (
                "keyword",
                "Variable declaration.\n\nUse `let` to bind a value to a name.",
            ),
            "mut" => (
                "keyword",
                "Mutable binding.\n\nUse `mut` to allow a variable to be mutated.",
            ),
            "fn" => ("keyword", "Function declaration."),
            "async" => (
                "keyword",
                "Asynchronous block or function.\n\nMarks a function or block that executes asynchronously and returns a Future.",
            ),
            "await" => (
                "keyword",
                "Await execution of a Future.\n\nSuspends execution until the asynchronous operation completes.",
            ),
            "match" => ("keyword", "Pattern matching."),
            "if" => ("keyword", "Conditional branching."),
            "else" => ("keyword", "Alternative conditional branch."),
            "return" => ("keyword", "Return a value from a function."),
            "struct" => (
                "keyword",
                "Structure definition.\n\nCreates a custom data type with fields.",
            ),
            "impl" => (
                "keyword",
                "Implementation block.\n\nDefines methods for a given struct.",
            ),
            "Option" => (
                "enum",
                "Optional value.\n\nCan be either `Some(value)` or `None`.",
            ),
            "Result" => (
                "enum",
                "Result value.\n\nCan be either `Ok(value)` or `Err(error)`.",
            ),
            "int" => ("type", "64-bit signed integer type."),
            "float" => ("type", "64-bit floating point type."),
            "string" => ("type", "UTF-8 string type."),
            "bool" => ("type", "Boolean type (`true` or `false`)."),
            "fmt" => (
                "module",
                "The `fmt` module provides utilities for formatting and printing strings.",
            ),
            "println" => (
                "function",
                "Prints given arguments to the standard output, followed by a newline.",
            ),
            _ => return String::new(),
        };

        format!("```peel\n({kind}) {word}\n```\n___\n{desc}")
    }
}

pub async fn start_lsp() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend {
        client,
        document_map: Mutex::new(HashMap::new()),
    })
    .finish();

    Server::new(stdin, stdout, socket).serve(service).await;
}
