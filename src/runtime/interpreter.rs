use crate::ast::{Literal, Expr, Stmt, Op, Pattern};
use crate::runtime::value::{PeelValue, PeelFunc};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use anyhow::{Result, anyhow};
use async_recursion::async_recursion;

pub type RuntimeResult<T> = Result<T>;

pub struct Environment {
    parent: Option<Arc<RwLock<Environment>>>,
    values: HashMap<String, PeelValue>,
}

impl Environment {
    pub fn new() -> Self {
        Self { parent: None, values: HashMap::new() }
    }

    pub fn child(parent: Arc<RwLock<Environment>>) -> Self {
        Self { parent: Some(parent), values: HashMap::new() }
    }

    pub fn get(&self, name: &str) -> Option<PeelValue> {
        if let Some(val) = self.values.get(name) {
            Some(val.clone())
        } else if let Some(parent) = &self.parent {
            parent.read().unwrap().get(name)
        } else {
            None
        }
    }

    pub fn define(&mut self, name: String, val: PeelValue) {
        self.values.insert(name, val);
    }
}

pub struct Interpreter {
    pub env: Arc<RwLock<Environment>>,
    pub structs: Arc<RwLock<HashMap<String, Vec<String>>>>,
    pub methods: Arc<RwLock<HashMap<String, HashMap<String, PeelValue>>>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: Arc::new(RwLock::new(Environment::new())),
            structs: Arc::new(RwLock::new(HashMap::new())),
            methods: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    #[async_recursion]
    pub async fn eval_stmt(&mut self, stmt: &Stmt) -> RuntimeResult<PeelValue> {
        match stmt {
            Stmt::Let { name, init, .. } => {
                let val = self.eval_expr(init).await?;
                self.env.write().unwrap().define(name.clone(), val);
                Ok(PeelValue::Void)
            }
            Stmt::Assign { target, value } => {
                let val = self.eval_expr(value).await?;
                match target {
                    Expr::Ident(name) => {
                        self.env.write().unwrap().define(name.clone(), val);
                    }
                    Expr::FieldAccess { target, field } => {
                        let instance = self.eval_expr(target).await?;
                        if let PeelValue::Object { fields, .. } = instance {
                            fields.lock().unwrap().insert(field.clone(), val);
                        } else {
                            return Err(anyhow!("Cannot assign to field of non-object type"));
                        }
                    }
                    _ => return Err(anyhow!("Invalid assignment target")),
                }
                Ok(PeelValue::Void)
            }
            Stmt::Func(func) => {
                let peel_func = PeelFunc {
                    name: func.name.clone(),
                    params: func.params.iter().map(|p| p.name.clone()).collect(),
                    body: func.body.clone(),
                    _is_async: func.is_async,
                };
                self.env.write().unwrap().define(func.name.clone(), PeelValue::Function(Arc::new(peel_func)));
                Ok(PeelValue::Void)
            }
            Stmt::Expr(expr) => self.eval_expr(expr).await,
            Stmt::If { cond, then_branch, else_branch } => {
                let cond_val = self.eval_expr(cond).await?;
                if let PeelValue::Bool(true) = cond_val {
                    for s in then_branch {
                        let res = self.eval_stmt(s).await?;
                        if let PeelValue::Return(_) = res { return Ok(res); }
                    }
                } else if let Some(branch) = else_branch {
                    for s in branch {
                        let res = self.eval_stmt(s).await?;
                        if let PeelValue::Return(_) = res { return Ok(res); }
                    }
                }
                Ok(PeelValue::Void)
            }
            Stmt::Return(expr) => {
                let val = if let Some(e) = expr {
                    self.eval_expr(e).await?
                } else {
                    PeelValue::Void
                };
                Ok(PeelValue::Return(Box::new(val)))
            }
            Stmt::Struct { name, fields } => {
                let field_names = fields.iter().map(|(n, _)| n.clone()).collect();
                self.structs.write().unwrap().insert(name.clone(), field_names);
                Ok(PeelValue::Void)
            }
            Stmt::Impl { target, methods } => {
                let mut method_map = HashMap::new();
                for m in methods {
                    let peel_func = PeelFunc {
                        name: m.name.clone(),
                        params: m.params.iter().map(|p| p.name.clone()).collect(),
                        body: m.body.clone(),
                        _is_async: m.is_async,
                    };
                    method_map.insert(m.name.clone(), PeelValue::Function(Arc::new(peel_func)));
                }
                self.methods.write().unwrap().entry(target.clone()).or_insert_with(HashMap::new).extend(method_map);
                Ok(PeelValue::Void)
            }
            _ => Ok(PeelValue::Void),
        }
    }

    #[async_recursion]
    pub async fn eval_expr(&mut self, expr: &Expr) -> RuntimeResult<PeelValue> {
        match expr {
            Expr::Literal(lit) => match lit {
                Literal::Int(i) => Ok(PeelValue::Int(*i)),
                Literal::Float(f) => Ok(PeelValue::Float(*f)),
                Literal::String(s) => Ok(PeelValue::String(s.clone())),
                Literal::Bool(b) => Ok(PeelValue::Bool(*b)),
                Literal::None => Ok(PeelValue::Void),
            },
            Expr::Ident(name) => self.env.read().unwrap().get(name).ok_or(anyhow!("Undefined variable '{}'", name)),
            Expr::Binary { left, op, right } => {
                let l = self.eval_expr(left).await?;
                let r = self.eval_expr(right).await?;
                match (l, r) {
                    (PeelValue::Int(a), PeelValue::Int(b)) => match op {
                        Op::Add => Ok(PeelValue::Int(a + b)),
                        Op::Sub => Ok(PeelValue::Int(a - b)),
                        Op::Mul => Ok(PeelValue::Int(a * b)),
                        Op::Div => Ok(PeelValue::Int(a / b)),
                        Op::Eq => Ok(PeelValue::Bool(a == b)),
                        Op::Ne => Ok(PeelValue::Bool(a != b)),
                        Op::Lt => Ok(PeelValue::Bool(a < b)),
                        Op::Gt => Ok(PeelValue::Bool(a > b)),
                        Op::Le => Ok(PeelValue::Bool(a <= b)),
                        Op::Ge => Ok(PeelValue::Bool(a >= b)),
                        _ => Err(anyhow!("Invalid operator for integers")),
                    },
                    (PeelValue::String(a), PeelValue::String(b)) => match op {
                        Op::Add => Ok(PeelValue::String(format!("{}{}", a, b))),
                        Op::Eq => Ok(PeelValue::Bool(a == b)),
                        _ => Err(anyhow!("Invalid operator for strings")),
                    },
                    _ => Err(anyhow!("Type mismatch in binary operation")),
                }
            }
            Expr::Call { callee, args } => {
                // Method Dispatch Check
                let mut method_to_call = None;
                let mut instance_val = None;

                if let Expr::FieldAccess { target, field } = callee.as_ref() {
                    let instance = self.eval_expr(target).await?;
                    let struct_name = match &instance {
                        PeelValue::Object { struct_name: Some(s_name), .. } => Some(s_name.clone()),
                        PeelValue::String(_) => Some("String".to_string()),
                        PeelValue::Int(_) | PeelValue::Float(_) => Some("Number".to_string()),
                        PeelValue::Bool(_) => Some("Boolean".to_string()),
                        PeelValue::List(_) => Some("Array".to_string()),
                        PeelValue::Map(_) => Some("Map".to_string()),
                        _ => None,
                    };

                    if let Some(s_name) = struct_name {
                        let methods = self.methods.read().unwrap();
                        if let Some(type_methods) = methods.get(&s_name) {
                            if let Some(method) = type_methods.get(field) {
                                method_to_call = Some(method.clone());
                                instance_val = Some(instance);
                            }
                        }
                    }
                }

                if let (Some(method), Some(instance)) = (method_to_call, instance_val) {
                    let mut arg_vals = vec![instance.clone()];
                    for arg in args {
                        arg_vals.push(self.eval_expr(arg).await?);
                    }
                    
                    match method {
                        PeelValue::Function(f) => {
                            let child_env = Arc::new(RwLock::new(Environment::child(self.env.clone())));
                            for (i, param_name) in f.params.iter().enumerate() {
                                child_env.write().unwrap().define(param_name.clone(), arg_vals[i].clone());
                            }
                            let mut sub_interpreter = Interpreter { 
                                env: child_env, 
                                structs: self.structs.clone(), 
                                methods: self.methods.clone() 
                            };
                            let mut last = PeelValue::Void;
                            for stmt in &f.body {
                                last = sub_interpreter.eval_stmt(stmt).await?;
                                if let PeelValue::Return(v) = last { return Ok(*v); }
                            }
                            Ok(last)
                        }
                        PeelValue::NativeFunction(f) => {
                            (f.handler)(arg_vals).await
                        }
                        _ => Err(anyhow!("Method is not a function")),
                    }
                } else {
                    let func_val = self.eval_expr(callee).await?;
                    let mut arg_vals = Vec::new();
                    for arg in args { arg_vals.push(self.eval_expr(arg).await?); }

                    match func_val {
                        PeelValue::Function(f) => {
                            let child_env = Arc::new(RwLock::new(Environment::child(self.env.clone())));
                            for (i, param) in f.params.iter().enumerate() {
                                child_env.write().unwrap().define(param.clone(), arg_vals[i].clone());
                            }
                            let mut sub_interpreter = Interpreter { 
                                env: child_env,
                                structs: self.structs.clone(),
                                methods: self.methods.clone()
                            };
                            let mut last = PeelValue::Void;
                            for stmt in &f.body {
                                last = sub_interpreter.eval_stmt(stmt).await?;
                                if let PeelValue::Return(v) = last { return Ok(*v); }
                            }
                            Ok(last)
                        }
                        PeelValue::NativeFunction(f) => {
                            (f.handler)(arg_vals).await
                        }
                        _ => Err(anyhow!("Cannot call non-function")),
                    }
                }
            }
            Expr::Await(inner) => self.eval_expr(inner).await, // Simple await implementation
            Expr::StructLiteral { name, fields } => {
                let mut field_values = HashMap::new();
                for (f_name, e) in fields {
                    field_values.insert(f_name.clone(), self.eval_expr(e).await?);
                }
                Ok(PeelValue::Object {
                    struct_name: Some(name.clone()),
                    fields: Arc::new(std::sync::Mutex::new(field_values)),
                })
            }
            Expr::ObjectLiteral { fields } => {
                let mut map = HashMap::new();
                for (name, e) in fields {
                    map.insert(name.clone(), self.eval_expr(e).await?);
                }
                Ok(PeelValue::Object {
                    struct_name: None,
                    fields: Arc::new(std::sync::Mutex::new(map)),
                })
            }
            Expr::FieldAccess { target, field } => {
                let val = self.eval_expr(target).await?;
                if let PeelValue::Object { fields, .. } = val {
                    let lock = fields.lock().unwrap();
                    Ok(lock.get(field).cloned().ok_or(anyhow!("Field '{}' not found", field))?)
                } else {
                    Err(anyhow!("Cannot access field '{}' on non-object", field))
                }
            }
            Expr::ArrayLiteral(elements) => {
                let mut vals = Vec::new();
                for e in elements {
                    vals.push(self.eval_expr(e).await?);
                }
                Ok(PeelValue::List(Arc::new(std::sync::Mutex::new(vals))))
            }
            Expr::Index { target, index } => {
                let t_val = self.eval_expr(target).await?;
                let i_val = self.eval_expr(index).await?;
                match (&t_val, &i_val) {
                    (PeelValue::List(l), PeelValue::Int(i)) => {
                        let lock = l.lock().unwrap();
                        let idx = *i;
                        if idx < 0 || idx as usize >= lock.len() {
                            return Err(anyhow!("Index {} out of bounds", idx));
                        }
                        Ok(lock[idx as usize].clone())
                    }
                    _ => Err(anyhow!("Invalid index operation: {:?}[{:?}]", t_val, i_val))
                }
            }
            Expr::Match { expr, arms } => {
                let val = self.eval_expr(expr).await?;
                for arm in arms {
                    if self.match_pattern(&val, &arm.pattern) {
                        return self.eval_expr(&arm.body).await;
                    }
                }
                Err(anyhow!("No match arm found for value {:?}", val))
            }
            Expr::Return(inner) => {
                let val = if let Some(e) = inner {
                    self.eval_expr(e).await?
                } else {
                    PeelValue::Void
                };
                Ok(PeelValue::Return(Box::new(val)))
            }
            Expr::EnumLiteral { name, inner } => {
                let val = if let Some(e) = inner {
                    Some(Box::new(self.eval_expr(e).await?))
                } else {
                    None
                };
                match name.as_str() {
                    "Ok" => Ok(PeelValue::Result(Ok(val.unwrap_or(Box::new(PeelValue::Void))))),
                    "Err" => Ok(PeelValue::Result(Err(val.unwrap_or(Box::new(PeelValue::Void))))),
                    "Some" => Ok(PeelValue::Option(Some(val.unwrap_or(Box::new(PeelValue::Void))))),
                    _ => Ok(PeelValue::Enum(name.clone(), val)),
                }
            }
            _ => Ok(PeelValue::Void),
        }
    }

    fn match_pattern(&self, val: &PeelValue, pat: &Pattern) -> bool {
        match (val, pat) {
            (_, Pattern::Wildcard) => true,
            (PeelValue::Int(v), Pattern::Literal(crate::ast::Literal::Int(p))) => v == p,
            (PeelValue::String(v), Pattern::Literal(crate::ast::Literal::String(p))) => v == p,
            (PeelValue::Bool(v), Pattern::Literal(crate::ast::Literal::Bool(p))) => v == p,
            (v, Pattern::Ident(name)) => {
                self.env.write().unwrap().define(name.clone(), v.clone());
                true
            }
            (PeelValue::Result(res), Pattern::Enum { name, inner }) => {
                match (res, name.as_str()) {
                    (Ok(v), "Ok") => {
                        if let Some(p) = inner { self.match_pattern(v, p) } else { true }
                    }
                    (Err(v), "Err") => {
                        if let Some(p) = inner { self.match_pattern(v, p) } else { true }
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }
}
