use crate::ast::*;

pub trait Visitor {
    fn visit_module(&mut self, m: &Module) {
        for s in &m.stmts {
            self.visit_stmt(s);
        }
    }
    fn visit_stmt(&mut self, s: &Stmt) {
        match &s.node {
            StmtNode::Let { name, ty, init, is_mut } => self.visit_let(name, ty, init, *is_mut, s.span.clone()),
            StmtNode::Assign { target, value } => self.visit_assign(target, value, s.span.clone()),
            StmtNode::If { cond, then_branch, else_branch } => self.visit_if(cond, then_branch, else_branch.as_ref(), s.span.clone()),
            StmtNode::While { cond, body } => self.visit_while(cond, body, s.span.clone()),
            StmtNode::For { var, iter, body } => self.visit_for(var, iter, body, s.span.clone()),
            StmtNode::Return(e) => self.visit_return(e.as_ref(), s.span.clone()),
            StmtNode::Expr(e) => self.visit_expr(e),
            StmtNode::Func(f) => self.visit_func(f, s.span.clone()),
            StmtNode::Import { path, symbols } => self.visit_import(path, symbols.as_ref(), s.span.clone()),
            StmtNode::Export(s) => self.visit_export(s),
            StmtNode::Struct { name, fields } => self.visit_struct(name, fields, s.span.clone()),
            StmtNode::Impl { target, methods } => self.visit_impl(target, methods, s.span.clone()),
            StmtNode::ExternBlock { lang, body, declarations } => self.visit_extern_block(lang, body, declarations, s.span.clone()),
        }
    }
    fn visit_let(&mut self, _name: &str, _ty: &Option<PeelType>, init: &Expr, _is_mut: bool, _span: std::ops::Range<usize>) {
        self.visit_expr(init);
    }
    fn visit_assign(&mut self, target: &Expr, value: &Expr, _span: std::ops::Range<usize>) {
        self.visit_expr(target);
        self.visit_expr(value);
    }
    fn visit_if(&mut self, cond: &Expr, then_branch: &[Stmt], else_branch: Option<&Vec<Stmt>>, _span: std::ops::Range<usize>) {
        self.visit_expr(cond);
        for s in then_branch { self.visit_stmt(s); }
        if let Some(eb) = else_branch {
            for s in eb { self.visit_stmt(s); }
        }
    }
    fn visit_while(&mut self, cond: &Expr, body: &[Stmt], _span: std::ops::Range<usize>) {
        self.visit_expr(cond);
        for s in body { self.visit_stmt(s); }
    }
    fn visit_for(&mut self, _var: &str, iter: &Expr, body: &[Stmt], _span: std::ops::Range<usize>) {
        self.visit_expr(iter);
        for s in body { self.visit_stmt(s); }
    }
    fn visit_return(&mut self, e: Option<&Expr>, _span: std::ops::Range<usize>) {
        if let Some(ev) = e { self.visit_expr(ev); }
    }
    fn visit_func(&mut self, f: &Func, _span: std::ops::Range<usize>) {
        for p in &f.params { self.visit_param(p); }
        for s in &f.body { self.visit_stmt(s); }
    }
    fn visit_param(&mut self, _p: &Param) {}
    fn visit_import(&mut self, _path: &str, _symbols: Option<&Vec<String>>, _span: std::ops::Range<usize>) {}
    fn visit_export(&mut self, s: &Stmt) { self.visit_stmt(s); }
    fn visit_struct(&mut self, _name: &str, _fields: &[(String, PeelType)], _span: std::ops::Range<usize>) {}
    fn visit_impl(&mut self, _target: &str, methods: &[Func], _span: std::ops::Range<usize>) {
        for m in methods { self.visit_func(m, 0..0); }
    }
    fn visit_extern_block(&mut self, _lang: &str, _body: &str, declarations: &[Func], _span: std::ops::Range<usize>) {
        for d in declarations { self.visit_func(d, 0..0); }
    }
    fn visit_expr(&mut self, e: &Expr) {
        match &e.node {
            ExprNode::Literal(_) => {},
            ExprNode::Ident(_) => {},
            ExprNode::Binary { left, right, .. } => {
                self.visit_expr(left);
                self.visit_expr(right);
            }
            ExprNode::Unary { right, .. } => self.visit_expr(right),
            ExprNode::Call { callee, args } => {
                self.visit_expr(callee);
                for a in args { self.visit_expr(a); }
            }
            ExprNode::Await(inner) => self.visit_expr(inner),
            ExprNode::Match { expr, arms } => {
                self.visit_expr(expr);
                for arm in arms {
                    self.visit_match_arm(arm);
                }
            }
            ExprNode::ObjectLiteral { fields } => {
                for (_, val) in fields { self.visit_expr(val); }
            }
            ExprNode::ArrayLiteral(elms) => {
                for e in elms { self.visit_expr(e); }
            }
            ExprNode::Index { target, index } => {
                self.visit_expr(target);
                self.visit_expr(index);
            }
            ExprNode::FieldAccess { target, .. } => self.visit_expr(target),
            ExprNode::Try(inner) => self.visit_expr(inner),
            ExprNode::Return(e) => { if let Some(ev) = e { self.visit_expr(ev); } },
            ExprNode::TypeCast { expr, .. } => self.visit_expr(expr),
        }
    }
    fn visit_match_arm(&mut self, arm: &MatchArm) {
        self.visit_expr(&arm.body);
    }
}
