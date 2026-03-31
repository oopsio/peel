use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f]+|//.*")] // Skip whitespace and comments
pub enum Token {
    // Keywords
    #[token("let")]
    Let,
    #[token("mut")]
    Mut,
    #[token("fn")]
    Fn,
    #[token("async")]
    Async,
    #[token("await")]
    Await,
    #[token("match")]
    Match,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("return")]
    Return,
    #[token("import")]
    Import,
    #[token("export")]
    Export,
    #[token("extern")]
    Extern,
    #[token("struct")]
    Struct,
    #[token("class")]
    Class,
    #[token("impl")]
    Impl,
    #[token("self")]
    SelfToken,
    #[token("for")]
    For,
    #[token("while")]
    While,
    #[token("in")]
    In,
    #[token("yield")]
    Yield,
    #[token("get")]
    Get,
    #[token("set")]
    Set,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("Ok")]
    Ok,
    #[token("Err")]
    Err,
    #[token("Some")]
    Some,
    #[token("None")]
    None,

    // Literals
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        let inner = &s[1..s.len()-1];
        let mut res = String::with_capacity(inner.len());
        let mut chars = inner.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '\\' {
                match chars.next() {
                    Some('n') => res.push('\n'),
                    Some('r') => res.push('\r'),
                    Some('t') => res.push('\t'),
                    Some('\\') => res.push('\\'),
                    Some('\"') => res.push('\"'),
                    Some(other) => {
                        res.push('\\');
                        res.push(other);
                    }
                    None => res.push('\\'),
                }
            } else {
                res.push(c);
            }
        }
        res
    })]
    String(String),

    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().unwrap())]
    Integer(i64),

    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().parse::<f64>().unwrap())]
    Float(f64),

    // Operators & Symbols
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("=")]
    Assign,
    #[token("==")]
    Equal,
    #[token("!=")]
    NotEqual,
    #[token("<")]
    Less,
    #[token(">")]
    Greater,
    #[token("<=")]
    LessEqual,
    #[token(">=")]
    GreaterEqual,
    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("!")]
    Not,
    #[token("?")]
    Question,
    #[token("?.")]
    QuestionDot,
    #[token("??")]
    DoubleQuestion,
    #[token("...")]
    Spread,
    #[token("->")]
    Arrow,
    #[token("=>")]
    FatArrow,
    #[token(".")]
    Dot,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token("::")]
    DoubleColon,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,

    #[token(";")]
    Semicolon,

    EOF,
}
