pub mod types;
use crate::ast::types::PeelType;

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal),
    Ident(String),
    Binary {
        left: Box<Expr>,
        op: Op,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        right: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Await(Box<Expr>),
    Match {
        expr: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    ObjectLiteral {
        fields: Vec<(String, Expr)>,
    },
    StructLiteral {
        name: String,
        fields: Vec<(String, Expr)>,
    },
    ArrayLiteral(Vec<Expr>),
    Index {
        target: Box<Expr>,
        index: Box<Expr>,
    },
    FieldAccess {
        target: Box<Expr>,
        field: String,
    },
    Try(Box<Expr>), // The `?` operator
    Return(Option<Box<Expr>>),
    EnumLiteral {
        name: String,
        inner: Option<Box<Expr>>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
    Add, Sub, Mul, Div,
    Eq, Ne, Lt, Gt, Le, Ge,
    And, Or,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOp {
    Neg, Not,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Literal(Literal),
    Ident(String),
    Enum {
        name: String,
        inner: Option<Box<Pattern>>,
    },
    Wildcard,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let {
        name: String,
        ty: Option<PeelType>,
        init: Expr,
        is_mut: bool,
    },
    Assign {
        target: Expr,
        value: Expr,
    },
    If {
        cond: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    For {
        var: String,
        iter: Expr,
        body: Vec<Stmt>,
    },
    Return(Option<Expr>),
    Expr(Expr),
    Func(Box<Func>),
    Import(String),
    Struct {
        name: String,
        fields: Vec<(String, PeelType)>,
    },
    Impl {
        target: String,
        methods: Vec<Func>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Func {
    pub name: String,
    pub params: Vec<Param>,
    pub ret_ty: PeelType,
    pub body: Vec<Stmt>,
    pub is_async: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: PeelType,
    pub is_mut: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub name: String,
    pub stmts: Vec<Stmt>,
}
