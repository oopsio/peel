use crate::ast::*;

pub struct Formatter {
    indent: usize,
    pub output: String,
}
impl Formatter {
    pub fn new() -> Self {
        Self {
            indent: 0,
            output: String::new(),
        }
    }
    fn push_str(&mut self, s: &str) {
        self.output.push_str(s);
    }
    fn push_indent(&mut self) {
        self.output.push_str(&"    ".repeat(self.indent));
    }
    pub fn format_module(&mut self, m: &Module) {
        for (i, s) in m.stmts.iter().enumerate() {
            if i > 0
                && matches!(
                    s,
                    Stmt::Func(_)
                        | Stmt::Struct { .. }
                        | Stmt::Impl { .. }
                        | Stmt::ExternBlock { .. }
                )
            {
                self.push_str("\n");
            }
            self.format_stmt(s);
            self.push_str("\n");
        }
    }
    fn format_stmt(&mut self, s: &Stmt) {
        match s {
            Stmt::Let {
                name,
                ty,
                init,
                is_mut,
            } => {
                self.push_indent();
                if *is_mut {
                    self.push_str("mut ");
                }
                self.push_str(&format!("let {}", name));
                if let Some(t) = ty {
                    self.push_str(": ");
                    self.format_type(t);
                }
                self.push_str(" = ");
                self.format_expr(init);
                self.push_str(";");
            }
            Stmt::Assign { target, value } => {
                self.push_indent();
                self.format_expr(target);
                self.push_str(" = ");
                self.format_expr(value);
                self.push_str(";");
            }
            Stmt::If {
                cond,
                then_branch,
                else_branch,
            } => {
                self.push_indent();
                self.push_str("if ");
                self.format_expr(cond);
                self.push_str(" ");
                self.format_block(then_branch);
                if let Some(eb) = else_branch {
                    self.push_str(" else ");
                    if eb.len() == 1 && matches!(eb[0], Stmt::If { .. }) {
                        self.push_stmt_no_indent(&eb[0]);
                    } else {
                        self.format_block(eb);
                    }
                }
            }
            Stmt::While { cond, body } => {
                self.push_indent();
                self.push_str("while ");
                self.format_expr(cond);
                self.push_str(" ");
                self.format_block(body);
            }
            Stmt::For { var, iter, body } => {
                self.push_indent();
                self.push_str(&format!("for {} in ", var));
                self.format_expr(iter);
                self.push_str(" ");
                self.format_block(body);
            }
            Stmt::Return(e) => {
                self.push_indent();
                self.push_str("return");
                if let Some(ev) = e {
                    self.push_str(" ");
                    self.format_expr(ev);
                }
                self.push_str(";");
            }
            Stmt::Expr(e) => {
                self.push_indent();
                self.format_expr(e);
                self.push_str(";");
            }
            Stmt::Func(f) => {
                self.push_indent();
                self.format_func(f);
            }
            Stmt::Import { path, symbols } => {
                self.push_indent();
                self.push_str("import ");
                if let Some(syms) = symbols {
                    self.push_str("{ ");
                    for (i, sym) in syms.iter().enumerate() {
                        if i > 0 {
                            self.push_str(", ");
                        }
                        self.push_str(sym);
                    }
                    self.push_str(" } from ");
                }
                self.push_str(&format!("\"{}\";", path));
            }
            Stmt::Export(inner) => {
                self.push_indent();
                self.push_str("export ");
                self.push_stmt_no_indent(inner);
            }
            Stmt::Struct { name, fields } => {
                self.push_indent();
                self.push_str(&format!("struct {} {{\n", name));
                self.indent += 1;
                for (fnm, fty) in fields {
                    self.push_indent();
                    self.push_str(fnm);
                    self.push_str(": ");
                    self.format_type(fty);
                    self.push_str(",\n");
                }
                self.indent -= 1;
                self.push_indent();
                self.push_str("}");
            }
            Stmt::Impl { target, methods } => {
                self.push_indent();
                self.push_str(&format!("impl {} {{\n", target));
                self.indent += 1;
                for (i, m) in methods.iter().enumerate() {
                    if i > 0 {
                        self.push_str("\n");
                    }
                    self.push_indent();
                    self.format_func(m);
                    self.push_str("\n");
                }
                self.indent -= 1;
                self.push_indent();
                self.push_str("}");
            }
            Stmt::ExternBlock {
                lang,
                body,
                declarations,
            } => {
                self.push_indent();
                self.push_str(&format!("extern \"{}\" \"{}\" {{\n", lang, body));
                self.indent += 1;
                for d in declarations {
                    self.push_indent();
                    self.push_str(&format!("fn {}(", d.name));
                    for (i, p) in d.params.iter().enumerate() {
                        if i > 0 {
                            self.push_str(", ");
                        }
                        self.push_str(&p.name);
                        self.push_str(": ");
                        self.format_type(&p.ty);
                    }
                    self.push_str(")");
                    if !matches!(d.ret_ty, PeelType::Void) {
                        self.push_str(" -> ");
                        self.format_type(&d.ret_ty);
                    }
                    self.push_str(";\n");
                }
                self.indent -= 1;
                self.push_indent();
                self.push_str("}");
            }
        }
    }
    fn push_stmt_no_indent(&mut self, s: &Stmt) {
        let old = self.indent;
        self.indent = 0;
        let start = self.output.len();
        self.format_stmt(s);
        let part = self.output[start..].trim_start().to_string();
        self.output.truncate(start);
        self.output.push_str(&part);
        self.indent = old;
    }
    fn format_block(&mut self, b: &[Stmt]) {
        self.push_str("{\n");
        self.indent += 1;
        for s in b {
            self.format_stmt(s);
            self.push_str("\n");
        }
        self.indent -= 1;
        self.push_indent();
        self.push_str("}");
    }
    fn format_func(&mut self, f: &Func) {
        if f.is_async {
            self.push_str("async ");
        }
        self.push_str(&format!("fn {}(", f.name));
        for (i, p) in f.params.iter().enumerate() {
            if i > 0 {
                self.push_str(", ");
            }
            if p.is_mut {
                self.push_str("mut ");
            }
            self.push_str(&p.name);
            self.push_str(": ");
            self.format_type(&p.ty);
        }
        self.push_str(")");
        if !matches!(f.ret_ty, PeelType::Void) {
            self.push_str(" -> ");
            self.format_type(&f.ret_ty);
        }
        self.push_str(" ");
        self.format_block(&f.body);
    }
    fn format_type(&mut self, t: &PeelType) {
        match t {
            PeelType::Int => self.push_str("int"),
            PeelType::Float => self.push_str("float"),
            PeelType::String => self.push_str("string"),
            PeelType::Bool => self.push_str("bool"),
            PeelType::Void => self.push_str("void"),
            PeelType::Object(s) => self.push_str(s),
            PeelType::Option(inner) => {
                self.push_str("Option<");
                self.format_type(inner);
                self.push_str(">");
            }
            PeelType::Result(ok, err) => {
                self.push_str("Result<");
                self.format_type(ok);
                self.push_str(", ");
                self.format_type(err);
                self.push_str(">");
            }
            PeelType::List(inner) => {
                self.push_str("Array<");
                self.format_type(inner);
                self.push_str(">");
            }
        }
    }
    fn format_expr(&mut self, e: &Expr) {
        match e {
            Expr::Literal(l) => match l {
                Literal::Int(v) => self.push_str(&v.to_string()),
                Literal::Float(v) => {
                    let s = v.to_string();
                    self.push_str(&s);
                    if !s.contains('.') {
                        self.push_str(".0");
                    }
                }
                Literal::String(v) => self.push_str(&format!("\"{}\"", v)),
                Literal::Bool(v) => self.push_str(if *v { "true" } else { "false" }),
                Literal::None => self.push_str("None"),
            },
            Expr::Ident(s) => self.push_str(s),
            Expr::Binary { left, op, right } => {
                self.format_expr(left);
                self.push_str(" ");
                self.push_str(match op {
                    Op::Add => "+",
                    Op::Sub => "-",
                    Op::Mul => "*",
                    Op::Div => "/",
                    Op::Eq => "==",
                    Op::Ne => "!=",
                    Op::Lt => "<",
                    Op::Gt => ">",
                    Op::Le => "<=",
                    Op::Ge => ">=",
                    Op::And => "&&",
                    Op::Or => "||",
                });
                self.push_str(" ");
                self.format_expr(right);
            }
            Expr::Unary { op, right } => {
                self.push_str(match op {
                    UnaryOp::Neg => "-",
                    UnaryOp::Not => "!",
                });
                self.format_expr(right);
            }
            Expr::Call { callee, args } => {
                self.format_expr(callee);
                self.push_str("(");
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        self.push_str(", ");
                    }
                    self.format_expr(a);
                }
                self.push_str(")");
            }
            Expr::Await(inner) => {
                self.push_str("await ");
                self.format_expr(inner);
            }
            Expr::Match { expr, arms } => {
                self.push_str("match ");
                self.format_expr(expr);
                self.push_str(" {\n");
                self.indent += 1;
                for arm in arms {
                    self.push_indent();
                    self.format_pattern(&arm.pattern);
                    self.push_str(" => ");
                    self.format_expr(&arm.body);
                    self.push_str(",\n");
                }
                self.indent -= 1;
                self.push_indent();
                self.push_str("}");
            }
            Expr::ObjectLiteral { fields } => {
                self.push_str("{ ");
                for (i, (nm, val)) in fields.iter().enumerate() {
                    if i > 0 {
                        self.push_str(", ");
                    }
                    self.push_str(nm);
                    self.push_str(": ");
                    self.format_expr(val);
                }
                self.push_str(" }");
            }
            Expr::ArrayLiteral(elms) => {
                self.push_str("[");
                for (i, e) in elms.iter().enumerate() {
                    if i > 0 {
                        self.push_str(", ");
                    }
                    self.format_expr(e);
                }
                self.push_str("]");
            }
            Expr::Index { target, index } => {
                self.format_expr(target);
                self.push_str("[");
                self.format_expr(index);
                self.push_str("]");
            }
            Expr::FieldAccess { target, field } => {
                self.format_expr(target);
                self.push_str(".");
                self.push_str(field);
            }
            Expr::Try(inner) => {
                self.format_expr(inner);
                self.push_str("?");
            }
            Expr::Return(ev) => {
                self.push_str("return");
                if let Some(e) = ev {
                    self.push_str(" ");
                    self.format_expr(e);
                }
            }
            Expr::TypeCast { expr, ty } => {
                self.format_expr(expr);
                self.push_str(": ");
                self.format_type(ty);
            }
        }
    }
    fn format_pattern(&mut self, p: &Pattern) {
        match p {
            Pattern::Literal(l) => match l {
                Literal::Int(v) => self.push_str(&v.to_string()),
                Literal::Float(v) => self.push_str(&v.to_string()),
                Literal::String(v) => self.push_str(&format!("\"{}\"", v)),
                Literal::Bool(v) => self.push_str(if *v { "true" } else { "false" }),
                Literal::None => self.push_str("None"),
            },
            Pattern::Ident(s) => self.push_str(s),
            Pattern::Wildcard => self.push_str("*"),
            Pattern::Enum { name, inner } => {
                self.push_str(name);
                if let Some(i) = inner {
                    self.push_str("(");
                    self.format_pattern(i);
                    self.push_str(")");
                }
            }
        }
    }
}
