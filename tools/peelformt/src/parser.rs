use crate::ast::*;
use crate::lexer::Token;
use anyhow::{Result, anyhow};
use logos::Logos;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Precedence {
    None,
    Assignment,
    Or,
    And,
    Equality,
    Comparison,
    Term,
    Factor,
    Cast,
    Unary,
    Call,
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
        Self {
            tokens,
            pos: 0,
            source,
        }
    }

    fn peek(&self) -> Token {
        if self.pos >= self.tokens.len() {
            Token::EOF
        } else {
            let t = self.tokens[self.pos].0.clone();
            if matches!(t, Token::Comment(_)) {
                let mut p = self.pos;
                while p < self.tokens.len() && matches!(self.tokens[p].0, Token::Comment(_)) {
                    p += 1;
                }
                if p >= self.tokens.len() {
                    Token::EOF
                } else {
                    self.tokens[p].0.clone()
                }
            } else {
                t
            }
        }
    }

    fn advance(&mut self) -> Token {
        while self.pos < self.tokens.len() && matches!(self.tokens[self.pos].0, Token::Comment(_)) {
            self.pos += 1;
        }
        if self.pos >= self.tokens.len() {
            return Token::EOF;
        }
        let t = self.tokens[self.pos].0.clone();
        self.pos += 1;
        t
    }

    fn check(&self, token: Token) -> bool {
        self.peek() == token
    }

    fn match_token(&mut self, token: Token) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn consume(&mut self, token: Token, message: &str) -> Result<Token> {
        if self.check(token) {
            Ok(self.advance())
        } else {
            Err(anyhow!("{} at line {}", message, self.line_of(self.pos)))
        }
    }

    fn consume_ident(&mut self, message: &str) -> Result<String> {
        if let Token::Ident(name) = self.peek() {
            self.advance();
            Ok(name)
        } else {
            Err(anyhow!("{} at line {}", message, self.line_of(self.pos)))
        }
    }

    fn line_of(&self, pos: usize) -> usize {
        let span = &self.tokens[pos.min(self.tokens.len() - 1)].1;
        self.source[..span.start].lines().count()
    }

    pub fn parse_module(&mut self) -> Result<Module> {
        let mut stmts = Vec::new();
        while !self.is_at_end() {
            stmts.push(self.parse_stmt()?);
        }
        Ok(Module { stmts })
    }

    fn is_at_end(&self) -> bool {
        self.peek() == Token::EOF
    }

    fn parse_stmt(&mut self) -> Result<Stmt> {
        self.match_token(Token::Semicolon);
        let stmt = if self.match_token(Token::Let) {
            self.parse_let(false)?
        } else if self.match_token(Token::Mut) {
            self.consume(Token::Let, "Expected 'let'")?;
            self.parse_let(true)?
        } else if self.match_token(Token::Fn) {
            self.parse_func(false)?
        } else if self.match_token(Token::Async) {
            self.consume(Token::Fn, "Expected 'fn'")?;
            self.parse_func(true)?
        } else if self.match_token(Token::If) {
            self.parse_if()?
        } else if self.match_token(Token::While) {
            self.parse_while()?
        } else if self.match_token(Token::For) {
            self.parse_for()?
        } else if self.match_token(Token::Return) {
            self.parse_return()?
        } else if self.match_token(Token::Import) {
            self.parse_import()?
        } else if self.match_token(Token::Export) {
            self.parse_export()?
        } else if self.match_token(Token::Struct) {
            self.parse_struct()?
        } else if self.match_token(Token::Impl) {
            self.parse_impl()?
        } else if self.match_token(Token::Extern) {
            self.parse_extern()?
        } else {
            let expr = self.parse_expr()?;
            if self.match_token(Token::Assign) {
                Stmt::Assign {
                    target: expr,
                    value: self.parse_expr()?,
                }
            } else {
                Stmt::Expr(expr)
            }
        };
        self.match_token(Token::Semicolon);
        Ok(stmt)
    }

    fn parse_let(&mut self, is_mut: bool) -> Result<Stmt> {
        let name = self.consume_ident("Expected name")?;
        let mut ty = None;
        if self.match_token(Token::Colon) {
            ty = Some(self.parse_type()?);
        }
        self.consume(Token::Assign, "Expected '='")?;
        Ok(Stmt::Let {
            name,
            ty,
            init: self.parse_expr()?,
            is_mut,
        })
    }

    fn parse_func(&mut self, is_async: bool) -> Result<Stmt> {
        let name = self.consume_ident("Expected name")?;
        self.consume(Token::LParen, "Expected '('")?;
        let mut params = Vec::new();
        if !self.check(Token::RParen) {
            loop {
                let p_is_mut = self.match_token(Token::Mut);
                let p_name = self.consume_ident("Expected param name")?;
                self.consume(Token::Colon, "Expected ':'")?;
                params.push(Param {
                    name: p_name,
                    ty: self.parse_type()?,
                    is_mut: p_is_mut,
                });
                if !self.match_token(Token::Comma) {
                    break;
                }
            }
        }
        self.consume(Token::RParen, "Expected ')'")?;
        let mut ret_ty = PeelType::Void;
        if self.match_token(Token::Arrow) {
            ret_ty = self.parse_type()?;
        }
        let body = self.parse_block()?;
        Ok(Stmt::Func(Func {
            name,
            params,
            ret_ty,
            body,
            is_async,
        }))
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>> {
        self.consume(Token::LBrace, "Expected '{'")?;
        let mut stmts = Vec::new();
        while !self.check(Token::RBrace) && !self.is_at_end() {
            stmts.push(self.parse_stmt()?);
        }
        self.consume(Token::RBrace, "Expected '}'")?;
        Ok(stmts)
    }

    fn parse_if(&mut self) -> Result<Stmt> {
        let cond = self.parse_expr()?;
        let then_branch = self.parse_block()?;
        let mut else_branch = None;
        if self.match_token(Token::Else) {
            if self.match_token(Token::If) {
                else_branch = Some(vec![self.parse_if()?]);
            } else {
                else_branch = Some(self.parse_block()?);
            }
        }
        Ok(Stmt::If {
            cond,
            then_branch,
            else_branch,
        })
    }

    fn parse_while(&mut self) -> Result<Stmt> {
        let cond = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt::While { cond, body })
    }

    fn parse_for(&mut self) -> Result<Stmt> {
        let var = self.consume_ident("Expected name")?;
        self.consume(Token::In, "Expected 'in'")?;
        let iter = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt::For { var, iter, body })
    }

    fn parse_return(&mut self) -> Result<Stmt> {
        let mut value = None;
        if !self.check(Token::Semicolon) && !self.check(Token::RBrace) && !self.is_at_end() {
            value = Some(self.parse_expr()?);
        }
        Ok(Stmt::Return(value))
    }

    fn parse_import(&mut self) -> Result<Stmt> {
        let mut symbols = None;
        if self.match_token(Token::LBrace) {
            let mut syms = Vec::new();
            while !self.check(Token::RBrace) && !self.is_at_end() {
                syms.push(self.consume_ident("Expected symbol")?);
                if !self.match_token(Token::Comma) {
                    break;
                }
            }
            self.consume(Token::RBrace, "Expected '}'")?;
            self.consume(Token::Ident("from".to_string()), "Expected 'from'")?;
            symbols = Some(syms);
        }
        let token = self.advance();
        let path = match token {
            Token::String(s) => s,
            Token::Ident(s) => s,
            _ => return Err(anyhow!("Expected path")),
        };
        Ok(Stmt::Import { path, symbols })
    }

    fn parse_export(&mut self) -> Result<Stmt> {
        Ok(Stmt::Export(Box::new(self.parse_stmt()?)))
    }

    fn parse_struct(&mut self) -> Result<Stmt> {
        let name = self.consume_ident("Expected name")?;
        self.consume(Token::LBrace, "Expected '{'")?;
        let mut fields = Vec::new();
        while !self.check(Token::RBrace) && !self.is_at_end() {
            let f_name = self.consume_ident("Expected field name")?;
            self.consume(Token::Colon, "Expected ':'")?;
            fields.push((f_name, self.parse_type()?));
            self.match_token(Token::Comma);
        }
        self.consume(Token::RBrace, "Expected '}'")?;
        Ok(Stmt::Struct { name, fields })
    }

    fn parse_impl(&mut self) -> Result<Stmt> {
        let target = self.consume_ident("Expected name")?;
        self.consume(Token::LBrace, "Expected '{'")?;
        let mut methods = Vec::new();
        while !self.check(Token::RBrace) && !self.is_at_end() {
            let is_async = self.match_token(Token::Async);
            self.consume(Token::Fn, "Expected 'fn'")?;
            if let Stmt::Func(f) = self.parse_func(is_async)? {
                methods.push(f);
            }
        }
        self.consume(Token::RBrace, "Expected '}'")?;
        Ok(Stmt::Impl { target, methods })
    }

    fn parse_extern(&mut self) -> Result<Stmt> {
        let lang = if let Token::String(s) = self.advance() {
            s
        } else {
            return Err(anyhow!("Expected lang"));
        };
        let body = if let Token::String(s) = self.advance() {
            s
        } else {
            return Err(anyhow!("Expected body"));
        };
        self.consume(Token::LBrace, "Expected '{'")?;
        let mut declarations = Vec::new();
        while !self.check(Token::RBrace) && !self.is_at_end() {
            self.consume(Token::Fn, "Expected 'fn'")?;
            let name = self.consume_ident("Expected name")?;
            self.consume(Token::LParen, "Expected '('")?;
            let mut params = Vec::new();
            if !self.check(Token::RParen) {
                loop {
                    let p_name = self.consume_ident("Expected name")?;
                    self.consume(Token::Colon, "Expected ':'")?;
                    params.push(Param {
                        name: p_name,
                        ty: self.parse_type()?,
                        is_mut: false,
                    });
                    if !self.match_token(Token::Comma) {
                        break;
                    }
                }
            }
            self.consume(Token::RParen, "Expected ')'")?;
            let mut ret_ty = PeelType::Void;
            if self.match_token(Token::Arrow) {
                ret_ty = self.parse_type()?;
            }
            self.consume(Token::Semicolon, "Expected ';'")?;
            declarations.push(Func {
                name,
                params,
                ret_ty,
                body: vec![],
                is_async: false,
            });
        }
        self.consume(Token::RBrace, "Expected '}'")?;
        Ok(Stmt::ExternBlock {
            lang,
            body,
            declarations,
        })
    }

    fn parse_type(&mut self) -> Result<PeelType> {
        if self.match_token(Token::Ident("int".to_string())) {
            Ok(PeelType::Int)
        } else if self.match_token(Token::Ident("float".to_string())) {
            Ok(PeelType::Float)
        } else if self.match_token(Token::Ident("number".to_string())) {
            Ok(PeelType::Float)
        } else if self.match_token(Token::Ident("string".to_string())) {
            Ok(PeelType::String)
        } else if self.match_token(Token::Ident("bool".to_string())) {
            Ok(PeelType::Bool)
        } else if self.match_token(Token::Ident("Option".to_string())) {
            self.consume(Token::Less, "Expected '<'")?;
            let t = self.parse_type()?;
            self.consume(Token::Greater, "Expected '>'")?;
            Ok(PeelType::Option(Box::new(t)))
        } else if self.match_token(Token::Ident("Result".to_string())) {
            self.consume(Token::Less, "Expected '<'")?;
            let ok = self.parse_type()?;
            self.consume(Token::Comma, "Expected ','")?;
            let err = self.parse_type()?;
            self.consume(Token::Greater, "Expected '>'")?;
            Ok(PeelType::Result(Box::new(ok), Box::new(err)))
        } else if self.match_token(Token::Ident("Array".to_string())) {
            if self.match_token(Token::Less) {
                let t = self.parse_type()?;
                self.consume(Token::Greater, "Expected '>'")?;
                Ok(PeelType::List(Box::new(t)))
            } else {
                Ok(PeelType::Object("Array".to_string()))
            }
        } else {
            let name = self.consume_ident("Expected type")?;
            Ok(PeelType::Object(name))
        }
    }

    fn parse_expr(&mut self) -> Result<Expr> {
        self.parse_precedence(Precedence::Assignment)
    }

    fn parse_precedence(&mut self, prec: Precedence) -> Result<Expr> {
        let t = self.advance();
        let mut left = self.parse_prefix(t)?;
        while prec <= self.get_precedence(&self.peek()) {
            let op = self.advance();
            left = self.parse_infix(left, op)?;
        }
        Ok(left)
    }

    fn parse_prefix(&mut self, t: Token) -> Result<Expr> {
        match t {
            Token::Integer(v) => Ok(Expr::Literal(Literal::Int(v))),
            Token::Float(v) => Ok(Expr::Literal(Literal::Float(v))),
            Token::String(v) => Ok(Expr::Literal(Literal::String(v))),
            Token::True => Ok(Expr::Literal(Literal::Bool(true))),
            Token::False => Ok(Expr::Literal(Literal::Bool(false))),
            Token::NoneVal => Ok(Expr::Literal(Literal::None)),
            Token::Ident(name) => Ok(Expr::Ident(name)),
            Token::LParen => {
                let e = self.parse_expr()?;
                self.consume(Token::RParen, "Expected ')'")?;
                Ok(e)
            }
            Token::Minus => Ok(Expr::Unary {
                op: UnaryOp::Neg,
                right: Box::new(self.parse_precedence(Precedence::Unary)?),
            }),
            Token::Not => Ok(Expr::Unary {
                op: UnaryOp::Not,
                right: Box::new(self.parse_precedence(Precedence::Unary)?),
            }),
            Token::Await => Ok(Expr::Await(Box::new(
                self.parse_precedence(Precedence::Unary)?,
            ))),
            Token::LBrace => self.parse_object_lit(),
            Token::LBracket => self.parse_array_lit(),
            Token::Match => self.parse_match(),
            _ => Err(anyhow!("Unexpected token {:?}", t)),
        }
    }

    fn parse_infix(&mut self, left: Expr, t: Token) -> Result<Expr> {
        match t {
            Token::Plus
            | Token::Minus
            | Token::Star
            | Token::Slash
            | Token::Equal
            | Token::NotEqual
            | Token::Less
            | Token::Greater
            | Token::LessEqual
            | Token::GreaterEqual
            | Token::And
            | Token::Or => {
                let op = match t {
                    Token::Plus => Op::Add,
                    Token::Minus => Op::Sub,
                    Token::Star => Op::Mul,
                    Token::Slash => Op::Div,
                    Token::Equal => Op::Eq,
                    Token::NotEqual => Op::Ne,
                    Token::Less => Op::Lt,
                    Token::Greater => Op::Gt,
                    Token::LessEqual => Op::Le,
                    Token::GreaterEqual => Op::Ge,
                    Token::And => Op::And,
                    Token::Or => Op::Or,
                    _ => unreachable!(),
                };
                let prec = self.get_precedence(&t);
                let right = self.parse_precedence(prec)?;
                Ok(Expr::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                })
            }
            Token::LParen => {
                let mut args = Vec::new();
                if !self.check(Token::RParen) {
                    loop {
                        args.push(self.parse_expr()?);
                        if !self.match_token(Token::Comma) {
                            break;
                        }
                    }
                }
                self.consume(Token::RParen, "Expected ')'")?;
                Ok(Expr::Call {
                    callee: Box::new(left),
                    args,
                })
            }
            Token::Dot => Ok(Expr::FieldAccess {
                target: Box::new(left),
                field: self.consume_ident("Expected field")?,
            }),
            Token::LBracket => {
                let idx = self.parse_expr()?;
                self.consume(Token::RBracket, "Expected ']'")?;
                Ok(Expr::Index {
                    target: Box::new(left),
                    index: Box::new(idx),
                })
            }
            Token::Question => Ok(Expr::Try(Box::new(left))),
            Token::Colon => Ok(Expr::TypeCast {
                expr: Box::new(left),
                ty: self.parse_type()?,
            }),
            _ => Ok(left),
        }
    }

    fn get_precedence(&self, t: &Token) -> Precedence {
        match t {
            Token::Or => Precedence::Or,
            Token::And => Precedence::And,
            Token::Equal | Token::NotEqual => Precedence::Equality,
            Token::Less | Token::Greater | Token::LessEqual | Token::GreaterEqual => {
                Precedence::Comparison
            }
            Token::Plus | Token::Minus => Precedence::Term,
            Token::Star | Token::Slash => Precedence::Factor,
            Token::Colon => Precedence::Cast,
            Token::LParen | Token::Dot | Token::LBracket | Token::Question => Precedence::Call,
            _ => Precedence::None,
        }
    }

    fn parse_object_lit(&mut self) -> Result<Expr> {
        let mut fields = Vec::new();
        while !self.check(Token::RBrace) && !self.is_at_end() {
            let name = self.consume_ident("Expected field name")?;
            self.consume(Token::Colon, "Expected ':'")?;
            fields.push((name, self.parse_expr()?));
            if !self.match_token(Token::Comma) {
                break;
            }
        }
        self.consume(Token::RBrace, "Expected '}'")?;
        Ok(Expr::ObjectLiteral { fields })
    }

    fn parse_array_lit(&mut self) -> Result<Expr> {
        let mut elements = Vec::new();
        while !self.check(Token::RBracket) && !self.is_at_end() {
            elements.push(self.parse_expr()?);
            if !self.match_token(Token::Comma) {
                break;
            }
        }
        self.consume(Token::RBracket, "Expected ']'")?;
        Ok(Expr::ArrayLiteral(elements))
    }

    fn parse_match(&mut self) -> Result<Expr> {
        let expr = self.parse_expr()?;
        self.consume(Token::LBrace, "Expected '{'")?;
        let mut arms = Vec::new();
        while !self.check(Token::RBrace) && !self.is_at_end() {
            let pattern = self.parse_pattern()?;
            self.consume(Token::FatArrow, "Expected '=>'")?;
            arms.push(MatchArm {
                pattern,
                body: self.parse_expr()?,
            });
            self.match_token(Token::Comma);
        }
        self.consume(Token::RBrace, "Expected '}'")?;
        Ok(Expr::Match {
            expr: Box::new(expr),
            arms,
        })
    }

    fn parse_pattern(&mut self) -> Result<Pattern> {
        if self.match_token(Token::Star) {
            Ok(Pattern::Wildcard)
        } else {
            let t = self.advance();
            match t {
                Token::Integer(v) => Ok(Pattern::Literal(Literal::Int(v))),
                Token::String(v) => Ok(Pattern::Literal(Literal::String(v))),
                Token::True => Ok(Pattern::Literal(Literal::Bool(true))),
                Token::False => Ok(Pattern::Literal(Literal::Bool(false))),
                Token::NoneVal => Ok(Pattern::Literal(Literal::None)),
                Token::Ident(name) => {
                    if self.match_token(Token::LParen) {
                        let inner = self.parse_pattern()?;
                        self.consume(Token::RParen, "Expected ')'")?;
                        Ok(Pattern::Enum { name, inner: Some(Box::new(inner)) })
                    } else {
                        Ok(Pattern::Ident(name))
                    }
                },
                _ => Err(anyhow!("Unexpected pattern {:?}", t)),
            }
        }
    }
}
