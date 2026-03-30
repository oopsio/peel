use crate::lexer::Token;
use crate::ast::{Literal, Expr, Stmt, Func, Param, Module, Op, UnaryOp, MatchArm, Pattern};
use crate::ast::types::PeelType;
use anyhow::Result;
use logos::Logos;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
enum Precedence {
    None,
    Assignment, // =
    Or,         // ||
    And,        // &&
    Equality,   // == !=
    Comparison, // < > <= >=
    Term,       // + -
    Factor,     // * /
    Cast,       // :
    Unary,      // ! -
    Call,       // . () []
}
pub struct Parser<'a> {
    tokens: Vec<(Token, std::ops::Range<usize>)>,
    pos: usize,
    source: &'a str,
    file_path: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str, file_path: &'a str) -> Self {
        let lex = Token::lexer(source);
        let mut tokens: Vec<(Token, std::ops::Range<usize>)> = lex.spanned()
            .map(|(t, span)| (t.unwrap_or(Token::Ident("ERROR".to_string())), span))
            .collect();
        tokens.push((Token::EOF, source.len()..source.len()));
        Self { tokens, pos: 0, source, file_path }
    }

    fn error(&self, message: &str) -> anyhow::Error {
        use colored::Colorize;
        let span = &self.tokens[self.pos.min(self.tokens.len().saturating_sub(1))].1;
        
        let mut line_num = 1;
        let mut line_start = 0;
        let bytes = self.source.as_bytes();
        
        for i in 0..span.start {
            if i < bytes.len() && bytes[i] == b'\n' {
                line_num += 1;
                line_start = i + 1;
            }
        }
        
        let mut line_end = self.source.len();
        for i in line_start..self.source.len() {
            if bytes[i] == b'\n' {
                line_end = i;
                break;
            }
        }
        
        let line_content = if line_start <= line_end {
            &self.source[line_start..line_end]
        } else {
            ""
        };
        
        let col = if span.start >= line_start { span.start - line_start } else { 0 };
        let end_col = if span.end >= line_start { span.end - line_start } else { 0 };
        let length = std::cmp::max(1, end_col.saturating_sub(col));
        let span_len = std::cmp::min(length, line_content.len().saturating_sub(col).max(1));
        let carets = "^".repeat(span_len);
        
        let msg = format!(
            "{}: {}\n  {} {}:{}:{}\n   {}\n{:>3} {} {}\n    {} {}{} {}",
            "error".red().bold(),
            message.bold(),
            "-->".blue().bold(),
            self.file_path,
            line_num,
            col + 1,
            "|".blue().bold(),
            line_num.to_string().blue().bold(),
            "|".blue().bold(),
            line_content,
            "|".blue().bold(),
            " ".repeat(col),
            carets.red().bold(),
            message.red().bold()
        );
        anyhow::anyhow!(msg)
    }

    pub fn parse_module(&mut self) -> Result<Module> {
        let mut stmts = Vec::new();
        while !self.is_at_end() {
            stmts.push(self.parse_stmt()?);
        }
        Ok(Module { name: "main".to_string(), stmts })
    }

    fn parse_stmt(&mut self) -> Result<Stmt> {
        if self.match_token(Token::Semicolon) {
            return Ok(Stmt::Expr(Expr::Literal(Literal::None))); // Or a dedicated NoOp
        }
        let stmt = if self.match_token(Token::Let) {
            self.parse_let_stmt(false)?
        } else if self.match_token(Token::Mut) {
            self.consume(Token::Let, "Expected 'let' after 'mut'")?;
            self.parse_let_stmt(true)?
        } else if self.match_token(Token::Fn) {
            self.parse_func_stmt(false)?
        } else if self.match_token(Token::Async) {
            self.consume(Token::Fn, "Expected 'fn' after 'async'")?;
            self.parse_func_stmt(true)?
        } else if self.match_token(Token::If) {
            self.parse_if_stmt()?
        } else if self.match_token(Token::Return) {
            self.parse_return_stmt()?
        } else if self.match_token(Token::Import) {
            self.parse_import_stmt()?
        } else if self.match_token(Token::Export) {
            self.parse_export_stmt()?
        } else if self.match_token(Token::Struct) {
            self.parse_struct_stmt()?
        } else if self.match_token(Token::Extern) {
            self.parse_extern_block()?
        } else if self.match_token(Token::Impl) {
            self.parse_impl_stmt()?
        } else {
            let expr = self.parse_expr()?;
            // Support simple assignments
            if self.match_token(Token::Assign) {
                let value = self.parse_expr()?;
                Stmt::Assign { target: expr, value }
            } else {
                Stmt::Expr(expr)
            }
        };

        self.match_token(Token::Semicolon); // Consume optional semicolon
        Ok(stmt)
    }

    fn parse_let_stmt(&mut self, is_mut: bool) -> Result<Stmt> {
        let name = self.consume_ident("Expected identifier after 'let'")?;

        let mut ty = None;
        if self.match_token(Token::Colon) {
            ty = Some(self.parse_type()?);
        }

        self.consume(Token::Assign, "Expected '=' in variable declaration")?;
        let init = self.parse_expr()?;
        Ok(Stmt::Let { name, ty, init, is_mut })
    }

    fn parse_func_stmt(&mut self, is_async: bool) -> Result<Stmt> {
        let name = self.consume_ident("Expected function name")?;

        self.consume(Token::LParen, "Expected '(' after function name")?;
        let mut params = Vec::new();
        if !self.check(Token::RParen) {
            loop {
                let mut is_mut = false;
                if self.match_token(Token::Mut) { is_mut = true; }

                let p_name = if self.match_token(Token::SelfToken) {
                    "self".to_string()
                } else {
                    self.consume_ident("Expected parameter name")?
                };

                let p_ty = if self.match_token(Token::Colon) {
                    self.parse_type()?
                } else if p_name == "self" {
                    PeelType::Unknown // Type Checker will fill this with the struct type
                } else {
                    return Err(self.error("Expected ':' after parameter name"));
                };

                params.push(Param { name: p_name, ty: p_ty, is_mut });

                if !self.match_token(Token::Comma) { break; }
            }
        }
        self.consume(Token::RParen, "Expected ')' after parameters")?;

        let mut ret_ty = PeelType::Void;
        if self.match_token(Token::Arrow) {
            ret_ty = self.parse_type()?;
        }

        self.consume(Token::LBrace, "Expected '{' before function body")?;
        let body = self.parse_block()?;

        Ok(Stmt::Func(Box::new(Func {
            name,
            params,
            ret_ty,
            body,
            is_async,
        })))
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = Vec::new();
        while !self.check(Token::RBrace) && !self.is_at_end() {
            statements.push(self.parse_stmt()?);
        }
        self.consume(Token::RBrace, "Expected '}' after block")?;
        Ok(statements)
    }

    fn parse_struct_stmt(&mut self) -> Result<Stmt> {
        let name = self.consume_ident("Expected struct name")?;
        self.consume(Token::LBrace, "Expected '{' after struct name")?;
        let mut fields = Vec::new();
        while !self.check(Token::RBrace) && !self.is_at_end() {
            let f_name = self.consume_ident("Expected field name")?;
            self.consume(Token::Colon, "Expected ':' after field name")?;
            let f_ty = self.parse_type()?;
            fields.push((f_name, f_ty));
            self.match_token(Token::Comma); // Optional comma
        }
        self.consume(Token::RBrace, "Expected '}' after struct fields")?;
        Ok(Stmt::Struct { name, fields })
    }

    fn parse_impl_stmt(&mut self) -> Result<Stmt> {
        let target = self.consume_ident("Expected struct name after 'impl'")?;
        self.consume(Token::LBrace, "Expected '{' after 'impl'")?;
        let mut methods = Vec::new();
        while !self.check(Token::RBrace) && !self.is_at_end() {
            let is_async = self.match_token(Token::Async);
            self.consume(Token::Fn, "Expected 'fn' in 'impl' block")?;
            if let Stmt::Func(f) = self.parse_func_stmt(is_async)? {
                methods.push(*f);
            } else {
                return Err(self.error("Expected function in 'impl' block"));
            }
        }
        self.consume(Token::RBrace, "Expected '}' after 'impl' block")?;
        Ok(Stmt::Impl { target, methods })
    }

    fn parse_extern_block(&mut self) -> Result<Stmt> {
        let token = self.advance();
        let lang = match token {
            Token::String(s) => s,
            _ => return Err(self.error("Expected string literal for language (e.g. \"C\" or \"nasm\")")),
        };

        let token = self.advance();
        let body = match token {
            Token::String(s) => s,
            _ => return Err(self.error("Expected string literal for inline code")),
        };

        self.consume(Token::LBrace, "Expected '{' after extern inline code")?;
        let mut declarations = Vec::new();
        while !self.check(Token::RBrace) && !self.is_at_end() {
            self.consume(Token::Fn, "Expected 'fn' in extern block")?;
            
            let name = self.consume_ident("Expected function name")?;
            self.consume(Token::LParen, "Expected '(' after function name")?;
            let mut params = Vec::new();
            if !self.check(Token::RParen) {
                loop {
                    let mut is_mut = false;
                    if self.match_token(Token::Mut) { is_mut = true; }
                    let p_name = self.consume_ident("Expected parameter name")?;
                    self.consume(Token::Colon, "Expected ':' after parameter name")?;
                    let p_ty = self.parse_type()?;
                    params.push(Param { name: p_name, ty: p_ty, is_mut });
                    if !self.match_token(Token::Comma) { break; }
                }
            }
            self.consume(Token::RParen, "Expected ')' after parameters")?;
            let mut ret_ty = PeelType::Void;
            if self.match_token(Token::Arrow) {
                ret_ty = self.parse_type()?;
            }
            self.consume(Token::Semicolon, "Expected ';' after extern function declaration")?;
            
            declarations.push(Func {
                name,
                params,
                ret_ty,
                body: vec![],
                is_async: false,
            });
        }
        self.consume(Token::RBrace, "Expected '}' after extern block")?;
        Ok(Stmt::ExternBlock { lang, body, declarations })
    }

    fn parse_type(&mut self) -> Result<PeelType> {
        if self.match_token(Token::Ident("int".to_string())) {
            Ok(PeelType::Int)
        } else if self.match_token(Token::Ident("float".to_string())) {
            Ok(PeelType::Float)
        } else if self.match_token(Token::Ident("string".to_string())) {
            Ok(PeelType::String)
        } else if self.match_token(Token::Ident("bool".to_string())) {
            Ok(PeelType::Bool)
        } else if self.match_token(Token::Ident("Option".to_string())) {
            self.consume(Token::Less, "Expected '<' after Option")?;
            let inner = self.parse_type()?;
            self.consume(Token::Greater, "Expected '>' after type")?;
            Ok(PeelType::Option(Box::new(inner)))
        } else if self.match_token(Token::Ident("Result".to_string())) {
            self.consume(Token::Less, "Expected '<' after Result")?;
            let ok = self.parse_type()?;
            self.consume(Token::Comma, "Expected ',' after type")?;
            let err = self.parse_type()?;
            self.consume(Token::Greater, "Expected '>' after type")?;
            Ok(PeelType::Result(Box::new(ok), Box::new(err)))
        } else {
            let name = self.consume_ident("Expected type name")?;
            Ok(PeelType::Object(name))
        }
    }

    fn parse_return_stmt(&mut self) -> Result<Stmt> {
        let mut value = None;
        if !self.check(Token::RBrace) && !self.check(Token::Semicolon) && !self.is_at_end() {
            value = Some(self.parse_expr()?);
        }
        Ok(Stmt::Return(value))
    }

    fn parse_import_stmt(&mut self) -> Result<Stmt> {
        let mut symbols = None;

        if self.match_token(Token::LBrace) {
            let mut syms = Vec::new();
            while !self.check(Token::RBrace) && !self.is_at_end() {
                syms.push(self.consume_ident("Expected symbol name")?);
                if !self.match_token(Token::Comma) { break; }
            }
            self.consume(Token::RBrace, "Expected '}' after import symbols")?;
            self.consume(Token::Ident("from".to_string()), "Expected 'from' after import symbols")?;
            symbols = Some(syms);
        }

        let token = self.advance();
        let path = match token {
            Token::String(s) => s,
            Token::Ident(s) => s, // Support legacy unquoted identifiers for now
            _ => return Err(self.error(&format!("Expected string literal for module path, got {:?}", token))),
        };

        Ok(Stmt::Import { path, symbols })
    }

    fn parse_export_stmt(&mut self) -> Result<Stmt> {
        let stmt = if self.match_token(Token::Let) {
            self.parse_let_stmt(false)?
        } else if self.match_token(Token::Mut) {
            self.consume(Token::Let, "Expected 'let' after 'mut'")?;
            self.parse_let_stmt(true)?
        } else if self.match_token(Token::Fn) {
            self.parse_func_stmt(false)?
        } else if self.match_token(Token::Async) {
            self.consume(Token::Fn, "Expected 'fn' after 'async'")?;
            self.parse_func_stmt(true)?
        } else if self.match_token(Token::Struct) {
            self.parse_struct_stmt()?
        } else {
            return Err(self.error("Expected declaration after 'export' (let, fn, struct)"));
        };

        Ok(Stmt::Export(Box::new(stmt)))
    }

    fn parse_if_stmt(&mut self) -> Result<Stmt> {
        let cond = self.parse_expr()?;
        self.consume(Token::LBrace, "Expected '{' after if condition")?;
        let then_branch = self.parse_block()?;
        let mut else_branch = None;
        if self.match_token(Token::Else) {
            if self.match_token(Token::If) {
                else_branch = Some(vec![self.parse_if_stmt()?]);
            } else {
                self.consume(Token::LBrace, "Expected '{' after else")?;
                else_branch = Some(self.parse_block()?);
            }
        }
        Ok(Stmt::If { cond, then_branch, else_branch })
    }

    fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_precedence(Precedence::Assignment)
    }

    fn parse_precedence(&mut self, precedence: Precedence) -> Result<Expr> {
        let token = self.advance();
        let mut left = self.parse_prefix(token)?;

        while precedence <= self.get_precedence(&self.peek()) {
            let token = self.advance();
            left = self.parse_infix(left, token)?;
        }

        Ok(left)
    }

    fn parse_prefix(&mut self, token: Token) -> Result<Expr> {
        match token {
            Token::Integer(val) => Ok(Expr::Literal(Literal::Int(val))),
            Token::Float(val) => Ok(Expr::Literal(Literal::Float(val))),
            Token::String(val) => Ok(Expr::Literal(Literal::String(val))),
            Token::True => Ok(Expr::Literal(Literal::Bool(true))),
            Token::False => Ok(Expr::Literal(Literal::Bool(false))),
            Token::None => Ok(Expr::Literal(Literal::None)),
            Token::Ident(name) => {
                if self.check(Token::LBrace) && self.is_struct_literal_lookahead() {
                    self.advance();
                    let mut fields = Vec::new();
                    if !self.check(Token::RBrace) {
                        loop {
                            let f_name = self.consume_ident("Expected field name")?;
                            self.consume(Token::Colon, "Expected ':' after field name")?;
                            let value = self.parse_expr()?;
                            fields.push((f_name, value));
                            if !self.match_token(Token::Comma) { break; }
                        }
                    }
                    self.consume(Token::RBrace, "Expected '}' after struct literal")?;
                    Ok(Expr::StructLiteral { name, fields })
                } else {
                    Ok(Expr::Ident(name))
                }
            }
            Token::LParen => {
                let expr = self.parse_expr()?;
                self.consume(Token::RParen, "Expected ')' after grouping expression")?;
                Ok(expr)
            }
            Token::Minus => {
                let right = self.parse_precedence(Precedence::Unary)?;
                Ok(Expr::Unary { op: UnaryOp::Neg, right: Box::new(right) })
            }
            Token::Not => {
                let right = self.parse_precedence(Precedence::Unary)?;
                Ok(Expr::Unary { op: UnaryOp::Not, right: Box::new(right) })
            }
            Token::Await => {
                let right = self.parse_precedence(Precedence::Unary)?;
                Ok(Expr::Await(Box::new(right)))
            }
            Token::LBrace => self.parse_object_literal(),
            Token::LBracket => self.parse_array_literal(),
            Token::Match => self.parse_match_expr(),
            Token::Return => {
                let value = if self.check(Token::RBrace) || self.check(Token::Semicolon) || self.check(Token::Comma) || self.is_at_end() {
                    None
                } else {
                    Some(Box::new(self.parse_expr()?))
                };
                Ok(Expr::Return(value))
            }
            Token::Ok => self.parse_enum_literal("Ok".to_string()),
            Token::Err => self.parse_enum_literal("Err".to_string()),
            Token::Some => self.parse_enum_literal("Some".to_string()),
            Token::SelfToken => Ok(Expr::Ident("self".to_string())),
            _ => Err(self.error(&format!("Unexpected token in expression: {:?}", token))),
        }
    }

    fn parse_enum_literal(&mut self, name: String) -> Result<Expr> {
        let mut inner = None;
        if self.match_token(Token::LParen) {
            inner = Some(Box::new(self.parse_expr()?));
            self.consume(Token::RParen, "Expected ')' after enum literal arguments")?;
        }
        Ok(Expr::EnumLiteral { name, inner })
    }

    fn parse_infix(&mut self, left: Expr, token: Token) -> Result<Expr> {
        match token {
            Token::Plus | Token::Minus | Token::Star | Token::Slash |
            Token::Equal | Token::NotEqual | Token::Less | Token::Greater |
            Token::LessEqual | Token::GreaterEqual | Token::And | Token::Or => {
                let op = match token {
                    Token::Plus => Op::Add, Token::Minus => Op::Sub,
                    Token::Star => Op::Mul, Token::Slash => Op::Div,
                    Token::Equal => Op::Eq, Token::NotEqual => Op::Ne,
                    Token::Less => Op::Lt, Token::Greater => Op::Gt,
                    Token::LessEqual => Op::Le, Token::GreaterEqual => Op::Ge,
                    Token::And => Op::And, Token::Or => Op::Or,
                    _ => unreachable!(),
                };
                let precedence = self.get_precedence(&token);
                let right = self.parse_precedence(precedence)?;
                Ok(Expr::Binary { left: Box::new(left), op, right: Box::new(right) })
            }
            Token::LParen => self.parse_call_expr(left),
            Token::Dot => {
                let field = self.consume_ident("Expected field name after '.'")?;
                Ok(Expr::FieldAccess { target: Box::new(left), field })
            }
            Token::LBracket => {
                let index = self.parse_expr()?;
                self.consume(Token::RBracket, "Expected ']' after index")?;
                Ok(Expr::Index { target: Box::new(left), index: Box::new(index) })
            }
            Token::Question => Ok(Expr::Try(Box::new(left))),
            Token::Colon => {
                let ty = self.parse_type()?;
                Ok(Expr::TypeCast { expr: Box::new(left), ty })
            }
            _ => Ok(left),
        }
    }

    fn get_precedence(&self, token: &Token) -> Precedence {
        match token {
            Token::Or => Precedence::Or,
            Token::And => Precedence::And,
            Token::Equal | Token::NotEqual => Precedence::Equality,
            Token::Less | Token::Greater | Token::LessEqual | Token::GreaterEqual => Precedence::Comparison,
            Token::Plus | Token::Minus => Precedence::Term,
            Token::Star | Token::Slash => Precedence::Factor,
            Token::Colon => Precedence::Cast,
            Token::LParen | Token::Dot | Token::LBracket | Token::Question => Precedence::Call,
            _ => Precedence::None,
        }
    }

    fn parse_call_expr(&mut self, callee: Expr) -> Result<Expr> {
        let mut args = Vec::new();
        if !self.check(Token::RParen) {
            loop {
                args.push(self.parse_expr()?);
                if !self.match_token(Token::Comma) { break; }
            }
        }
        self.consume(Token::RParen, "Expected ')' after arguments")?;
        Ok(Expr::Call { callee: Box::new(callee), args })
    }

    fn parse_object_literal(&mut self) -> Result<Expr> {
        let mut fields = Vec::new();
        if !self.check(Token::RBrace) {
            loop {
                let name = self.consume_ident("Expected field name")?;
                self.consume(Token::Colon, "Expected ':' after field name")?;
                let value = self.parse_expr()?;
                fields.push((name, value));
                if !self.match_token(Token::Comma) { break; }
            }
        }
        self.consume(Token::RBrace, "Expected '}' after object literal")?;
        Ok(Expr::ObjectLiteral { fields })
    }

    fn parse_array_literal(&mut self) -> Result<Expr> {
        let mut elements = Vec::new();
        if !self.check(Token::RBracket) {
            loop {
                elements.push(self.parse_expr()?);
                if !self.match_token(Token::Comma) { break; }
            }
        }
        self.consume(Token::RBracket, "Expected ']' after array literal")?;
        Ok(Expr::ArrayLiteral(elements))
    }

    fn parse_match_expr(&mut self) -> Result<Expr> {
        let expr = self.parse_expr()?;
        self.consume(Token::LBrace, "Expected '{' after match expression")?;
        let mut arms = Vec::new();
        while !self.check(Token::RBrace) && !self.is_at_end() {
            let pattern = self.parse_pattern()?;
            self.consume(Token::FatArrow, "Expected '=>' after pattern")?;
            let body = self.parse_expr()?;
            arms.push(MatchArm { pattern, body });
            self.match_token(Token::Comma); // Optional comma
        }
        self.consume(Token::RBrace, "Expected '}' after match expression")?;
        Ok(Expr::Match { expr: Box::new(expr), arms })
    }

    fn parse_pattern(&mut self) -> Result<Pattern> {
        if self.match_token(Token::Star) {
            Ok(Pattern::Wildcard)
        } else if self.match_token(Token::Ok) {
            self.parse_enum_pattern("Ok".to_string())
        } else if self.match_token(Token::Err) {
            self.parse_enum_pattern("Err".to_string())
        } else if self.match_token(Token::Some) {
            self.parse_enum_pattern("Some".to_string())
        } else if self.match_token(Token::None) {
            Ok(Pattern::Literal(Literal::None))
        } else {
            let token = self.advance();
            match token {
                Token::Integer(v) => Ok(Pattern::Literal(Literal::Int(v))),
                Token::String(v) => Ok(Pattern::Literal(Literal::String(v))),
                Token::True => Ok(Pattern::Literal(Literal::Bool(true))),
                Token::False => Ok(Pattern::Literal(Literal::Bool(false))),
                Token::Ident(name) => Ok(Pattern::Ident(name)),
                _ => Err(self.error(&format!("Unexpected token in pattern: {:?}", token))),
            }
        }
    }

    fn parse_enum_pattern(&mut self, name: String) -> Result<Pattern> {
        let mut inner = None;
        if self.match_token(Token::LParen) {
            inner = Some(Box::new(self.parse_pattern()?));
            self.consume(Token::RParen, "Expected ')' after enum pattern")?;
        }
        Ok(Pattern::Enum { name, inner })
    }
    fn match_ident(&mut self) -> Option<String> {
        if self.is_at_end() { return None; }
        if let Token::Ident(name) = self.peek() {
            self.advance();
            Some(name)
        } else {
            None
        }
    }

    fn consume_ident(&mut self, message: &str) -> Result<String> {
        if let Some(name) = self.match_ident() {
            Ok(name)
        } else {
            Err(self.error(message))
        }
    }

    fn match_token(&mut self, token: Token) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, token: Token) -> bool {
        if self.is_at_end() { return false; }
        self.peek() == token
    }

    fn peek(&self) -> Token {
        if self.is_at_end() { return Token::EOF; }
        self.tokens[self.pos].0.clone()
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() { self.pos += 1; }
        self.previous()
    }

    fn previous(&self) -> Token {
        self.tokens[self.pos - 1].0.clone()
    }

    fn is_at_end(&self) -> bool {
        self.tokens[self.pos].0 == Token::EOF
    }

    fn consume(&mut self, token: Token, message: &str) -> Result<Token> {
        if self.check(token) {
            Ok(self.advance())
        } else {
            Err(self.error(message))
        }
    }

    fn is_struct_literal_lookahead(&self) -> bool {
        // Check if next two tokens are Ident and Colon
        // self.pos is at Ident. Peek is LBrace at self.pos
        if self.pos + 2 >= self.tokens.len() { return false; }
        match (&self.tokens[self.pos + 1].0, &self.tokens[self.pos + 2].0) {
            (Token::Ident(_), Token::Colon) => true,
            _ => false,
        }
    }
}
