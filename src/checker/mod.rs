use crate::ast::{Module, Stmt, Expr, Literal, Op, UnaryOp};
use crate::ast::types::PeelType;
use std::collections::HashMap;
use anyhow::{Result, anyhow};

pub struct Checker {
    scopes: Vec<Scope>,
    pub structs: HashMap<String, Vec<(String, PeelType)>>,
    pub methods: HashMap<String, HashMap<String, PeelType>>,
}

struct Scope {
    variables: HashMap<String, VariableInfo>,
}

struct VariableInfo {
    ty: PeelType,
    is_mut: bool,
}

impl Checker {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope { variables: HashMap::new() }],
            structs: HashMap::new(),
            methods: HashMap::new(),
        }
    }

    pub fn check_module(&mut self, module: &Module) -> Result<()> {
        for stmt in &module.stmts {
            self.check_stmt(stmt)?;
        }
        Ok(())
    }

    fn check_stmt(&mut self, stmt: &Stmt) -> Result<()> {
        match stmt {
            Stmt::Let { name, ty, init, is_mut } => {
                let init_ty = self.check_expr(init)?;
                let final_ty = if let Some(explicit_ty) = ty {
                    if !init_ty.matches(explicit_ty) {
                        return Err(anyhow!("Type mismatch for '{}': expected {:?}, got {:?}", name, explicit_ty, init_ty));
                    }
                    explicit_ty.clone()
                } else {
                    init_ty
                };
                self.define(name, final_ty, *is_mut);
            }
            Stmt::Assign { target, value } => {
                let val_ty = self.check_expr(value)?;
                match target {
                    Expr::Ident(name) => {
                        let info = self.resolve(name)?;
                        if !info.is_mut {
                            return Err(anyhow!("Cannot assign to immutable variable '{}'", name));
                        }
                        if !val_ty.matches(&info.ty) {
                            return Err(anyhow!("Type mismatch for '{}': expected {:?}, got {:?}", name, info.ty, val_ty));
                        }
                    }
                    Expr::FieldAccess { target, field: _ } => {
                        let _target_ty = self.check_expr(target)?;
                        // For now, we trust field assignments if the target type is an object.
                        // Future: Check if the field exists and its type matches.
                    }
                    _ => return Err(anyhow!("Invalid assignment target")),
                }
            }
            Stmt::Func(func) => {
                // Register function signature first
                let param_types: Vec<PeelType> = func.params.iter().map(|p| p.ty.clone()).collect();
                self.define(&func.name, PeelType::Func {
                    params: param_types,
                    ret: Box::new(func.ret_ty.clone()),
                    is_async: func.is_async,
                }, false);

                self.begin_scope();
                for param in &func.params {
                    self.define(&param.name, param.ty.clone(), false);
                }
                for stmt in &func.body {
                    self.check_stmt(stmt)?;
                }
                self.end_scope();
            }
            Stmt::If { cond, then_branch, else_branch } => {
                let cond_ty = self.check_expr(cond)?;
                if cond_ty != PeelType::Bool {
                    return Err(anyhow!("If condition must be bool, got {:?}", cond_ty));
                }
                self.begin_scope();
                for s in then_branch { self.check_stmt(s)?; }
                self.end_scope();
                if let Some(branch) = else_branch {
                    self.begin_scope();
                    for s in branch { self.check_stmt(s)?; }
                    self.end_scope();
                }
            }
            Stmt::Return(expr) => {
                if let Some(e) = expr {
                    self.check_expr(e)?;
                }
            }
            Stmt::Expr(expr) => {
                self.check_expr(expr)?;
            }
            Stmt::Import(_) => {} // Handled by runtime/resolver
            Stmt::Struct { name, fields } => {
                self.structs.insert(name.clone(), fields.clone());
            }
            Stmt::Impl { target, methods } => {
                if !self.structs.contains_key(target) {
                    return Err(anyhow!("Cannot impl for unknown struct '{}'", target));
                }
                let mut method_map = HashMap::new();
                for m in methods {
                    let param_types: Vec<PeelType> = m.params.iter().map(|p| p.ty.clone()).collect();
                    method_map.insert(m.name.clone(), PeelType::Func {
                        params: param_types,
                        ret: Box::new(m.ret_ty.clone()),
                        is_async: m.is_async,
                    });
                }
                self.methods.entry(target.clone()).or_insert_with(HashMap::new).extend(method_map);
            }
            _ => {}
        }
        Ok(())
    }

    fn check_expr(&mut self, expr: &Expr) -> Result<PeelType> {
        match expr {
            Expr::Literal(lit) => match lit {
                Literal::Int(_) => Ok(PeelType::Int),
                Literal::Float(_) => Ok(PeelType::Float),
                Literal::String(_) => Ok(PeelType::String),
                Literal::Bool(_) => Ok(PeelType::Bool),
                Literal::None => Ok(PeelType::Unknown), // Polymorphic None
            },
            Expr::Ident(name) => Ok(self.resolve(name)?.ty.clone()),
            Expr::Binary { left, op, right } => {
                let l = self.check_expr(left)?;
                let r = self.check_expr(right)?;
                if !l.matches(&r) {
                    return Err(anyhow!("Binary op {:?} type mismatch: {:?} and {:?}", op, l, r));
                }
                match op {
                    Op::Eq | Op::Ne | Op::Lt | Op::Gt | Op::Le | Op::Ge => Ok(PeelType::Bool),
                    _ => Ok(l),
                }
            }
            Expr::Unary { op, right } => {
                let r = self.check_expr(right)?;
                match op {
                    UnaryOp::Neg => {
                        if r != PeelType::Int && r != PeelType::Float {
                            return Err(anyhow!("Cannot negate {:?}", r));
                        }
                        Ok(r)
                    }
                    UnaryOp::Not => {
                        if r != PeelType::Bool {
                            return Err(anyhow!("Cannot NOT {:?}", r));
                        }
                        Ok(PeelType::Bool)
                    }
                }
            }
            Expr::Call { callee, args } => {
                let callee_ty = self.check_expr(callee)?;
                if let PeelType::Func { params, ret, .. } = callee_ty {
                    if params.len() != args.len() {
                        return Err(anyhow!("Function expected {} args, got {}", params.len(), args.len()));
                    }
                    for (i, arg) in args.iter().enumerate() {
                        let arg_ty = self.check_expr(arg)?;
                        if !arg_ty.matches(&params[i]) {
                            return Err(anyhow!("Arg {} type mismatch: expected {:?}, got {:?}", i, params[i], arg_ty));
                        }
                    }
                    Ok(*ret)
                } else if callee_ty == PeelType::Unknown {
                    Ok(PeelType::Unknown) // Late bound or builtin
                } else {
                    Err(anyhow!("Cannot call non-function type {:?}", callee_ty))
                }
            }
            Expr::Await(inner) => {
                let ty = self.check_expr(inner)?;
                // In a real system, we'd check if inner returns a Future
                Ok(ty)
            }
            Expr::Try(inner) => {
                let ty = self.check_expr(inner)?;
                match ty {
                    PeelType::Result(ok, _) => Ok(*ok),
                    PeelType::Option(inner) => Ok(*inner),
                    _ => Err(anyhow!("'?' operator used on non-Result/Option type {:?}", ty)),
                }
            }
            Expr::StructLiteral { name, fields } => {
                let struct_fields = self.structs.get(name).ok_or(anyhow!("Unknown struct '{}'", name))?.clone();
                for (f_name, f_expr) in fields {
                    let f_ty = self.check_expr(f_expr)?;
                    let expected_ty = struct_fields.iter().find(|(n, _)| n == f_name)
                        .ok_or(anyhow!("Field '{}' not found in struct '{}'", f_name, name))?.1.clone();
                    if !f_ty.matches(&expected_ty) {
                        return Err(anyhow!("Type mismatch for field '{}': expected {:?}, got {:?}", f_name, expected_ty, f_ty));
                    }
                }
                Ok(PeelType::Object(name.clone()))
            }
            Expr::FieldAccess { target, field } => {
                let target_ty = self.check_expr(target)?;
                if let PeelType::Object(struct_name) = target_ty {
                    if let Some(fields) = self.structs.get(&struct_name) {
                        if let Some((_, ty)) = fields.iter().find(|(n, _)| n == field) {
                            return Ok(ty.clone());
                        }
                    }
                    Ok(PeelType::Unknown)
                } else {
                    Ok(PeelType::Unknown)
                }
            }
            Expr::Match { expr, arms } => {
                let _val_ty = self.check_expr(expr)?;
                for _arm in arms {
                    // Pattern checking
                }
                Ok(PeelType::Unknown)
            }
            Expr::ObjectLiteral { fields } => {
                for (_, e) in fields {
                    self.check_expr(e)?;
                }
                Ok(PeelType::Unknown)
            }
            Expr::Return(expr) => {
                if let Some(e) = expr {
                    self.check_expr(e)?;
                }
                Ok(PeelType::Unknown) // diverge type later
            }
            Expr::EnumLiteral { name, inner } => {
                if let Some(e) = inner {
                    self.check_expr(e)?;
                }
                match name.as_str() {
                    "Ok" | "Err" => Ok(PeelType::Result(Box::new(PeelType::Unknown), Box::new(PeelType::Unknown))),
                    "Some" => Ok(PeelType::Option(Box::new(PeelType::Unknown))),
                    _ => Ok(PeelType::Object(name.clone())),
                }
            }
            Expr::ArrayLiteral(elements) => {
                let mut element_ty = PeelType::Unknown;
                for e in elements {
                    let ty = self.check_expr(e)?;
                    if element_ty == PeelType::Unknown {
                        element_ty = ty;
                    } else if !ty.matches(&element_ty) {
                        return Err(anyhow!("Array element type mismatch: expected {:?}, got {:?}", element_ty, ty));
                    }
                }
                Ok(PeelType::List(Box::new(element_ty)))
            }
            Expr::Index { target, index } => {
                let t_ty = self.check_expr(target)?;
                let i_ty = self.check_expr(index)?;
                if i_ty != PeelType::Int && i_ty != PeelType::Unknown {
                    return Err(anyhow!("Array index must be an integer, got {:?}", i_ty));
                }
                match t_ty {
                    PeelType::List(inner) => Ok(*inner),
                    PeelType::Unknown => Ok(PeelType::Unknown),
                    _ => Err(anyhow!("Cannot index into non-list type {:?}", t_ty)),
                }
            }
        }
    }

    pub fn define(&mut self, name: &str, ty: PeelType, is_mut: bool) {
        let scope = self.scopes.last_mut().unwrap();
        scope.variables.insert(name.to_string(), VariableInfo { ty, is_mut });
    }

    fn resolve(&self, name: &str) -> Result<&VariableInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.variables.get(name) {
                return Ok(info);
            }
        }
        Err(anyhow!("Undefined variable '{}'", name))
    }

    fn begin_scope(&mut self) {
        self.scopes.push(Scope { variables: HashMap::new() });
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }
}
