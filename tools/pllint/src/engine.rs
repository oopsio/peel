use crate::ast::*;
use crate::visitor::Visitor;
use colored::*;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Level { Error, Warning, Info }

pub struct LintError {
    pub id: String,
    pub message: String,
    pub level: Level,
    pub line: usize,
}

pub struct LinterEngine<'a> {
    pub errors: Vec<LintError>,
    pub source: &'a str,
    pub banned_names: HashSet<&'static str>,
}

impl<'a> LinterEngine<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut banned = HashSet::new();
        let generic_names = [
            "data", "temp", "tmp", "val", "var", "obj", "item", "thing", "stuff", "foo", "bar", "baz", "qux",
            "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z",
            "test1", "test2", "test3", "test_var", "my_var", "dummy_var", "placeholder_var", "some_data", "data_ptr", "ptr1", "ptr2",
            "idx1", "idx2", "val1", "val2", "tmp1", "tmp2", "temp1", "temp2", "foo1", "foo2", "bar1", "bar2", "baz1", "baz2",
            "arr", "list", "vec", "map", "set", "dict", "coll", "items", "elements", "vals", "vars", "objs", "nodes", "elms",
            "ptr", "addr", "ref", "p", "q", "x", "y", "z", "w", "h", "t", "u", "v", "i", "j", "k", "l", "m", "n",
            "count", "total", "sum", "avg", "min", "max", "start", "end", "begin", "finish", "first", "last", "prev", "next",
            "curr", "old", "new", "buf", "buffer", "raw", "data_raw", "input", "output", "in", "out", "param", "arg", "res", "ret",
            "status", "state", "mode", "kind", "type", "cfg", "config", "opts", "options", "settings", "props", "attrs", "meta",
            "id", "uid", "uuid", "guid", "slug", "name", "title", "desc", "label", "tag", "cat", "group", "parent"
        ];
        for name in &generic_names { banned.insert(*name); }
        
        Self {
            errors: Vec::new(),
            source,
            banned_names: banned,
        }
    }

    fn line_of(&self, offset: usize) -> usize {
        let actual_offset = offset.min(self.source.len());
        self.source[..actual_offset].lines().count()
    }

    pub fn report(&mut self, id: &str, msg: &str, level: Level, offset: usize) {
        self.errors.push(LintError {
            id: id.to_string(),
            message: msg.to_string(),
            level,
            line: self.line_of(offset).saturating_sub(1),
        });
    }

    fn check_name(&mut self, name: &str, kind: &str, offset: usize) {
        if self.banned_names.contains(name) {
            self.report(&format!("naming:banned:{}", name), &format!("The name '{}' is too generic for a {}", name, kind), Level::Warning, offset);
        }
        if name.len() > 30 {
            self.report("naming:too_long", &format!("{} name '{}' is too long", kind, name), Level::Warning, offset);
        }
        if name.len() < 3 && !["i", "j", "k", "x", "y"].contains(&name) {
            self.report("naming:too_short", &format!("{} name '{}' is too short", kind, name), Level::Info, offset);
        }

        // Casing checks
        if kind == "struct" {
            if !name.chars().next().map_or(false, |c| c.is_uppercase()) {
                self.report("naming:pascal_case", &format!("Struct name '{}' should be PascalCase", name), Level::Warning, offset);
            }
        } else if kind == "variable" || kind == "function" || kind == "parameter" || kind == "field" {
            if name.chars().next().map_or(false, |c| c.is_uppercase()) {
                self.report("naming:snake_case", &format!("{} name '{}' should be snake_case", kind, name), Level::Warning, offset);
            }
        }
    }
}

impl<'a> Visitor for LinterEngine<'a> {
    fn visit_let(&mut self, name: &str, _ty: &Option<PeelType>, init: &Expr, _is_mut: bool, span: std::ops::Range<usize>) {
        self.check_name(name, "variable", span.start);
        self.visit_expr(init);
    }
    
    fn visit_func(&mut self, f: &Func, span: std::ops::Range<usize>) {
        self.check_name(&f.name, "function", span.start);
        if f.params.len() > 5 {
            self.report("complexity:too_many_params", "Too many parameters", Level::Warning, span.start);
        }
        for p in &f.params {
            self.check_name(&p.name, "parameter", span.start);
        }
        for s in &f.body {
            self.visit_stmt(s);
        }
    }

    fn visit_struct(&mut self, name: &str, fields: &[(String, PeelType)], span: std::ops::Range<usize>) {
        self.check_name(name, "struct", span.start);
        for (f_name, _) in fields {
            self.check_name(f_name, "field", span.start);
        }
    }

    fn visit_while(&mut self, cond: &Expr, body: &[Stmt], span: std::ops::Range<usize>) {
        if body.is_empty() {
            self.report("style:empty_while", "Empty 'while' block", Level::Warning, span.start);
        }
        self.visit_expr(cond);
        for s in body { self.visit_stmt(s); }
    }

    fn visit_if(&mut self, cond: &Expr, then_branch: &[Stmt], else_branch: Option<&Vec<Stmt>>, span: std::ops::Range<usize>) {
        if then_branch.is_empty() {
            self.report("style:empty_if", "Empty 'if' block", Level::Warning, span.start);
        }
        self.visit_expr(cond);
        for s in then_branch { self.visit_stmt(s); }
        if let Some(eb) = else_branch {
            for s in eb { self.visit_stmt(s); }
        }
    }

    fn visit_expr(&mut self, e: &Expr) {
        match &e.node {
            ExprNode::Binary { left, op, right } => {
                if matches!(op, Op::Div) {
                    if let ExprNode::Literal(Literal::Int(0)) = &right.node {
                        self.report("bug:division_by_zero", "Division by zero literal", Level::Error, e.span.start);
                    }
                }
                self.visit_expr(left);
                self.visit_expr(right);
            }
            ExprNode::Unary { op, right } => {
                if matches!(op, UnaryOp::Not) {
                    if let ExprNode::Unary { op: op2, .. } = &right.node {
                        if matches!(op2, UnaryOp::Not) {
                            self.report("style:double_negation", "Redundant double negation", Level::Warning, e.span.start);
                        }
                    }
                }
                self.visit_expr(right);
            }
            ExprNode::Call { callee, args } => {
                self.visit_expr(callee);
                for a in args { self.visit_expr(a); }
            }
            ExprNode::ObjectLiteral { fields } => {
                for (name, val) in fields {
                    self.check_name(name, "field", e.span.start);
                    self.visit_expr(val);
                }
            }
            ExprNode::ArrayLiteral(elements) => {
                for el in elements { self.visit_expr(el); }
            }
            ExprNode::Index { target, index } => {
                self.visit_expr(target);
                self.visit_expr(index);
            }
            ExprNode::FieldAccess { target, field } => {
                self.visit_expr(target);
                self.check_name(field, "field", e.span.start);
            }
            _ => {}
        }
    }
}
