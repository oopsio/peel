use crate::ast::*;
use crate::lexer::Token;
use anyhow::{Result, anyhow};
use logos::Logos;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Precedence {
    None, Assignment, Or, And, Equality, Comparison, Term, Factor, Cast, Unary, Call,
}

pub struct Parser<'a> {
    tokens: Vec<(Token, std::ops::Range<usize>)>,
    pos: usize,
    source: &'a str,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str, _file_path: &'a str) -> Self {
        let lex = Token::lexer(source);
        let mut tokens: Vec<(Token, std::ops::Range<usize>)> = lex
            .spanned()
            .map(|(t, span)| (t.unwrap_or(Token::Ident("ERROR".to_string())), span))
            .collect();
        tokens.push((Token::EOF, source.len()..source.len()));
        Self { tokens, pos: 0, source }
    }

    fn peek(&self) -> Token {
        let mut p = self.pos;
        while p < self.tokens.len() && matches!(self.tokens[p].0, Token::Comment(_)) {
            p += 1;
        }
        if p >= self.tokens.len() { Token::EOF }
        else { self.tokens[p].0.clone() }
    }
    
    fn peek_span(&self) -> std::ops::Range<usize> {
        let mut p = self.pos;
        while p < self.tokens.len() && matches!(self.tokens[p].0, Token::Comment(_)) {
            p += 1;
        }
        if p >= self.tokens.len() { self.source.len()..self.source.len() }
        else { self.tokens[p].1.clone() }
    }

    fn advance(&mut self) -> (Token, std::ops::Range<usize>) {
        while self.pos < self.tokens.len() && matches!(self.tokens[self.pos].0, Token::Comment(_)) {
            self.pos += 1;
        }
        if self.pos >= self.tokens.len() { (Token::EOF, self.source.len()..self.source.len()) }
        else {
            let t = self.tokens[self.pos].clone();
            self.pos += 1;
            t
        }
    }

    fn check(&self, token: Token) -> bool { self.peek() == token }
    fn match_token(&mut self, token: Token) -> bool {
        if self.check(token) { self.advance(); true } else { false }
    }
    fn consume(&mut self, token: Token, message: &str) -> Result<(Token, std::ops::Range<usize>)> {
        if self.check(token) { Ok(self.advance()) }
        else { Err(anyhow!("{} at line {}", message, self.line_of(self.pos))) }
    }
    fn consume_ident(&mut self, message: &str) -> Result<(String, std::ops::Range<usize>)> {
        if let Token::Ident(name) = self.peek() {
            let (_, span) = self.advance();
            Ok((name, span))
        } else { Err(anyhow!("{} at line {}", message, self.line_of(self.pos))) }
    }

    fn line_of(&self, pos: usize) -> usize {
        let idx = pos.min(self.tokens.len() - 1);
        let span = &self.tokens[idx].1;
        self.source[..span.start].lines().count()
    }

    pub fn parse_module(&mut self) -> Result<Module> {
        let mut stmts = Vec::new();
        while !self.is_at_end() { stmts.push(self.parse_stmt()?); }
        Ok(Module { stmts })
    }

    fn is_at_end(&self) -> bool { self.peek() == Token::EOF }

    fn parse_stmt(&mut self) -> Result<Stmt> {
        let start = self.peek_span().start;
        self.match_token(Token::Semicolon);
        let node = if self.match_token(Token::Let) { self.parse_let(false)? }
        else if self.match_token(Token::Mut) { self.consume(Token::Let, "Expected 'let'")?; self.parse_let(true)? }
        else if self.match_token(Token::Fn) { StmtNode::Func(self.parse_func(false)?) }
        else if self.match_token(Token::Async) { self.consume(Token::Fn, "Expected 'fn'")?; StmtNode::Func(self.parse_func(true)?) }
        else if self.match_token(Token::If) { self.parse_if()? }
        else if self.match_token(Token::While) { self.parse_while()? }
        else if self.match_token(Token::For) { self.parse_for()? }
        else if self.match_token(Token::Return) { self.parse_return()? }
        else if self.match_token(Token::Import) { self.parse_import()? }
        else if self.match_token(Token::Export) { self.parse_export()? }
        else if self.match_token(Token::Struct) { self.parse_struct()? }
        else if self.match_token(Token::Impl) { self.parse_impl()? }
        else if self.match_token(Token::Extern) { self.parse_extern()? }
        else {
            let expr = self.parse_expr()?;
            if self.match_token(Token::Assign) {
                StmtNode::Assign { target: expr, value: self.parse_expr()? }
            } else { StmtNode::Expr(expr) }
        };
        let end = self.peek_span().end;
        self.match_token(Token::Semicolon);
        Ok(Spanned { node, span: start..end })
    }

    fn parse_let(&mut self, is_mut: bool) -> Result<StmtNode> {
        let (name, _) = self.consume_ident("Expected name")?;
        let mut ty = None;
        if self.match_token(Token::Colon) { ty = Some(self.parse_type()?); }
        self.consume(Token::Assign, "Expected '='")?;
        Ok(StmtNode::Let { name, ty, init: self.parse_expr()?, is_mut })
    }

    fn parse_func(&mut self, is_async: bool) -> Result<Func> {
        let (name, _) = self.consume_ident("Expected name")?;
        self.consume(Token::LParen, "Expected '('")?;
        let mut params = Vec::new();
        if !self.check(Token::RParen) {
            loop {
                let p_is_mut = self.match_token(Token::Mut);
                let (p_name, _) = self.consume_ident("Expected param name")?;
                self.consume(Token::Colon, "Expected ':'")?;
                params.push(Param { name: p_name, ty: self.parse_type()?, is_mut: p_is_mut });
                if !self.match_token(Token::Comma) { break; }
            }
        }
        self.consume(Token::RParen, "Expected ')'")?;
        let mut ret_ty = PeelType::Void;
        if self.match_token(Token::Arrow) { ret_ty = self.parse_type()?; }
        let body = self.parse_block()?;
        Ok(Func { name, params, ret_ty, body, is_async })
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>> {
        self.consume(Token::LBrace, "Expected '{'")?;
        let mut stmts = Vec::new();
        while !self.check(Token::RBrace) && !self.is_at_end() { stmts.push(self.parse_stmt()?); }
        self.consume(Token::RBrace, "Expected '}'")?;
        Ok(stmts)
    }

    fn parse_if(&mut self) -> Result<StmtNode> {
        let cond = self.parse_expr()?;
        let then_branch = self.parse_block()?;
        let mut else_branch = None;
        if self.match_token(Token::Else) {
            if self.match_token(Token::If) {
                let start = self.peek_span().start;
                let node = self.parse_if()?;
                let end = self.peek_span().end;
                else_branch = Some(vec![Spanned { node, span: start..end }]);
            } else { else_branch = Some(self.parse_block()?); }
        }
        Ok(StmtNode::If { cond, then_branch, else_branch })
    }

    fn parse_while(&mut self) -> Result<StmtNode> {
        let cond = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(StmtNode::While { cond, body })
    }

    fn parse_return(&mut self) -> Result<StmtNode> {
        let mut value = None;
        if !self.check(Token::Semicolon) && !self.check(Token::RBrace) { value = Some(self.parse_expr()?); }
        Ok(StmtNode::Return(value))
    }

    fn parse_for(&mut self) -> Result<StmtNode> {
        let (var, _) = self.consume_ident("Expected var name")?;
        self.consume(Token::In, "Expected 'in'")?;
        let iter = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(StmtNode::For { var, iter, body })
    }

    fn parse_import(&mut self) -> Result<StmtNode> {
        let mut symbols = None;
        if self.match_token(Token::LBrace) {
            let mut syms = Vec::new();
            while !self.check(Token::RBrace) && !self.is_at_end() {
                let (sym, _) = self.consume_ident("Expected symbol")?;
                syms.push(sym);
                if !self.match_token(Token::Comma) { break; }
            }
            self.consume(Token::RBrace, "Expected '}'")?;
            self.consume(Token::Ident("from".to_string()), "Expected 'from'")?;
            symbols = Some(syms);
        }
        let token = self.advance().0;
        let path = match token {
            Token::String(s) => s,
            Token::Ident(s) => s,
            _ => return Err(anyhow!("Expected path")),
        };
        Ok(StmtNode::Import { path, symbols })
    }

    fn parse_export(&mut self) -> Result<StmtNode> {
        Ok(StmtNode::Export(Box::new(self.parse_stmt()?)))
    }

    fn parse_struct(&mut self) -> Result<StmtNode> {
        let (name, _) = self.consume_ident("Expected name")?;
        self.consume(Token::LBrace, "Expected '{'")?;
        let mut fields = Vec::new();
        while !self.check(Token::RBrace) && !self.is_at_end() {
            let (f_name, _) = self.consume_ident("Expected field name")?;
            self.consume(Token::Colon, "Expected ':'")?;
            fields.push((f_name, self.parse_type()?));
            self.match_token(Token::Comma);
        }
        self.consume(Token::RBrace, "Expected '}'")?;
        Ok(StmtNode::Struct { name, fields })
    }

    fn parse_impl(&mut self) -> Result<StmtNode> {
        let (target, _) = self.consume_ident("Expected name")?;
        self.consume(Token::LBrace, "Expected '{'")?;
        let mut methods = Vec::new();
        while !self.check(Token::RBrace) && !self.is_at_end() {
            let is_async = self.match_token(Token::Async);
            self.consume(Token::Fn, "Expected 'fn'")?;
            methods.push(self.parse_func(is_async)?);
        }
        self.consume(Token::RBrace, "Expected '}'")?;
        Ok(StmtNode::Impl { target, methods })
    }

    fn parse_extern(&mut self) -> Result<StmtNode> {
        let lang = if let Token::String(s) = self.advance().0 { s } else { return Err(anyhow!("Expected lang")); };
        let body = if let Token::String(s) = self.advance().0 { s } else { return Err(anyhow!("Expected body")); };
        self.consume(Token::LBrace, "Expected '{'")?;
        let mut declarations = Vec::new();
        while !self.check(Token::RBrace) && !self.is_at_end() {
            self.consume(Token::Fn, "Expected 'fn'")?;
            let (name, _) = self.consume_ident("Expected name")?;
            self.consume(Token::LParen, "Expected '('")?;
            let mut params = Vec::new();
            if !self.check(Token::RParen) {
                loop {
                    let (p_name, _) = self.consume_ident("Expected name")?;
                    self.consume(Token::Colon, "Expected ':'")?;
                    params.push(Param { name: p_name, ty: self.parse_type()?, is_mut: false });
                    if !self.match_token(Token::Comma) { break; }
                }
            }
            self.consume(Token::RParen, "Expected ')'")?;
            let mut ret_ty = PeelType::Void;
            if self.match_token(Token::Arrow) { ret_ty = self.parse_type()?; }
            self.consume(Token::Semicolon, "Expected ';'")?;
            declarations.push(Func { name, params, ret_ty, body: vec![], is_async: false });
        }
        self.consume(Token::RBrace, "Expected '}'")?;
        Ok(StmtNode::ExternBlock { lang, body, declarations })
    }

    fn parse_type(&mut self) -> Result<PeelType> {
        if self.match_token(Token::Ident("int".to_string())) { Ok(PeelType::Int) }
        else if self.match_token(Token::Ident("float".to_string())) { Ok(PeelType::Float) }
        else if self.match_token(Token::Ident("string".to_string())) { Ok(PeelType::String) }
        else if self.match_token(Token::Ident("bool".to_string())) { Ok(PeelType::Bool) }
        else {
            let (name, _) = self.consume_ident("Expected type")?;
            Ok(PeelType::Object(name))
        }
    }

    fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_precedence(Precedence::Assignment)
    }

    fn parse_precedence(&mut self, prec: Precedence) -> Result<Expr> {
        let start = self.peek_span().start;
        let (t, _) = self.advance();
        let mut left = self.parse_prefix(t, start)?;
        while prec <= self.get_precedence(&self.peek()) {
            let (op, _) = self.advance();
            left = self.parse_infix(left, op)?;
        }
        Ok(left)
    }

    fn parse_prefix(&mut self, t: Token, start: usize) -> Result<Expr> {
        let node = match t {
            Token::Integer(v) => ExprNode::Literal(Literal::Int(v)),
            Token::Float(v) => ExprNode::Literal(Literal::Float(v)),
            Token::String(v) => ExprNode::Literal(Literal::String(v)),
            Token::True => ExprNode::Literal(Literal::Bool(true)),
            Token::False => ExprNode::Literal(Literal::Bool(false)),
            Token::NoneVal => ExprNode::Literal(Literal::None),
            Token::Ident(name) => ExprNode::Ident(name),
            Token::LParen => {
                let e = self.parse_expr()?;
                self.consume(Token::RParen, "Expected ')'")?;
                return Ok(e);
            }
            Token::Minus => ExprNode::Unary { op: UnaryOp::Neg, right: Box::new(self.parse_precedence(Precedence::Unary)?) },
            Token::Not => ExprNode::Unary { op: UnaryOp::Not, right: Box::new(self.parse_precedence(Precedence::Unary)?) },
            _ => return Err(anyhow!("Unexpected token {:?}", t)),
        };
        let end = self.peek_span().end;
        Ok(Spanned { node, span: start..end })
    }

    fn parse_infix(&mut self, left: Expr, t: Token) -> Result<Expr> {
        let start = left.span.start;
        let node = match t {
            Token::Plus | Token::Minus | Token::Star | Token::Slash | Token::Equal | Token::NotEqual | Token::Less | Token::Greater | Token::And | Token::Or => {
                let op = match t {
                    Token::Plus => Op::Add, Token::Minus => Op::Sub, Token::Star => Op::Mul, Token::Slash => Op::Div,
                    Token::Equal => Op::Eq, Token::NotEqual => Op::Ne, Token::Less => Op::Lt, Token::Greater => Op::Gt,
                    Token::And => Op::And, Token::Or => Op::Or, _ => unreachable!(),
                };
                let prec = self.get_precedence(&t);
                let right = self.parse_precedence(prec)?;
                ExprNode::Binary { left: Box::new(left), op, right: Box::new(right) }
            }
            Token::LParen => {
                let mut args = Vec::new();
                if !self.check(Token::RParen) {
                    loop {
                        args.push(self.parse_expr()?);
                        if !self.match_token(Token::Comma) { break; }
                    }
                }
                self.consume(Token::RParen, "Expected ')'")?;
                ExprNode::Call { callee: Box::new(left), args }
            }
            Token::Dot => {
                let (field, _) = self.consume_ident("Expected field")?;
                ExprNode::FieldAccess { target: Box::new(left), field }
            }
            _ => return Ok(left),
        };
        let end = self.peek_span().end;
        Ok(Spanned { node, span: start..end })
    }

    fn get_precedence(&self, t: &Token) -> Precedence {
        match t {
            Token::Or => Precedence::Or, Token::And => Precedence::And,
            Token::Equal | Token::NotEqual => Precedence::Equality,
            Token::Less | Token::Greater => Precedence::Comparison,
            Token::Plus | Token::Minus => Precedence::Term,
            Token::Star | Token::Slash => Precedence::Factor,
            Token::LParen | Token::Dot => Precedence::Call,
            _ => Precedence::None,
        }
    }
}
