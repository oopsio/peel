use crate::ast::{Expr, Literal, Op, Pattern, Stmt};
use crate::runtime::value::{PeelFunc, PeelValue};
use anyhow::{Result, anyhow};
use async_recursion::async_recursion;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub type RuntimeResult<T> = Result<T>;

pub struct Environment {
    parent: Option<Arc<RwLock<Environment>>>,
    pub values: HashMap<String, PeelValue>,
    pub exports: std::collections::HashSet<String>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            parent: None,
            values: HashMap::new(),
            exports: std::collections::HashSet::new(),
        }
    }

    pub fn child(parent: Arc<RwLock<Environment>>) -> Self {
        Self {
            parent: Some(parent),
            values: HashMap::new(),
            exports: std::collections::HashSet::new(),
        }
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
    pub module_cache: Arc<RwLock<HashMap<String, Arc<RwLock<Environment>>>>>,
    pub current_path: std::path::PathBuf,
    pub libraries: Arc<RwLock<Vec<Arc<libloading::Library>>>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: Arc::new(RwLock::new(Environment::new())),
            structs: Arc::new(RwLock::new(HashMap::new())),
            methods: Arc::new(RwLock::new(HashMap::new())),
            module_cache: Arc::new(RwLock::new(HashMap::new())),
            current_path: std::env::current_dir().unwrap_or_default(),
            libraries: Arc::new(RwLock::new(Vec::new())),
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
                self.env
                    .write()
                    .unwrap()
                    .define(func.name.clone(), PeelValue::Function(Arc::new(peel_func)));
                Ok(PeelValue::Void)
            }
            Stmt::Expr(expr) => self.eval_expr(expr).await,
            Stmt::If {
                cond,
                then_branch,
                else_branch,
            } => {
                let cond_val = self.eval_expr(cond).await?;
                if let PeelValue::Bool(true) = cond_val {
                    for s in then_branch {
                        let res = self.eval_stmt(s).await?;
                        if let PeelValue::Return(_) = res {
                            return Ok(res);
                        }
                    }
                } else if let Some(branch) = else_branch {
                    for s in branch {
                        let res = self.eval_stmt(s).await?;
                        if let PeelValue::Return(_) = res {
                            return Ok(res);
                        }
                    }
                }
                Ok(PeelValue::Void)
            }
            Stmt::While { cond, body } => {
                while let PeelValue::Bool(true) = self.eval_expr(cond).await? {
                    for s in body {
                        let res = self.eval_stmt(s).await?;
                        if let PeelValue::Return(_) = res {
                            return Ok(res);
                        }
                    }
                }
                Ok(PeelValue::Void)
            }
            Stmt::For { var, iter, body } => {
                let iter_val = self.eval_expr(iter).await?;
                if let PeelValue::List(l) = iter_val {
                    let items = l.lock().unwrap().clone();
                    for item in items {
                        self.env.write().unwrap().define(var.clone(), item);
                        for s in body {
                            let res = self.eval_stmt(s).await?;
                            if let PeelValue::Return(_) = res {
                                return Ok(res);
                            }
                        }
                    }
                } else {
                    return Err(anyhow!("Cannot iterate over non-array type"));
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
                self.structs
                    .write()
                    .unwrap()
                    .insert(name.clone(), field_names);
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
                self.methods
                    .write()
                    .unwrap()
                    .entry(target.clone())
                    .or_insert_with(HashMap::new)
                    .extend(method_map);
                Ok(PeelValue::Void)
            }
            Stmt::Export(inner) => {
                let res = self.eval_stmt(inner).await?;
                match inner.as_ref() {
                    Stmt::Let { name, .. } => {
                        self.env.write().unwrap().exports.insert(name.clone());
                    }
                    Stmt::Func(f) => {
                        self.env.write().unwrap().exports.insert(f.name.clone());
                    }
                    Stmt::Struct { name, .. } => {
                        self.env.write().unwrap().exports.insert(name.clone());
                    }
                    Stmt::ExternBlock { declarations, .. } => {
                        for f in declarations {
                            self.env.write().unwrap().exports.insert(f.name.clone());
                        }
                    }
                    _ => {}
                }
                Ok(res)
            }
            Stmt::ExternBlock {
                lang,
                body,
                declarations,
            } => {
                let uuid_str = uuid::Uuid::new_v4().to_string().replace("-", "");
                let temp_dir = std::env::temp_dir();
                let dll_path = temp_dir.join(format!("peel_ffi_{}.dll", uuid_str));

                if lang == "C" {
                    let c_path = temp_dir.join(format!("tmp_ffi_{}.c", uuid_str));
                    std::fs::write(&c_path, body)
                        .map_err(|e| anyhow!("Failed to write C FFI code: {}", e))?;

                    let status = std::process::Command::new("gcc")
                        .args(&[
                            std::ffi::OsStr::new("-O3"),
                            std::ffi::OsStr::new("-shared"),
                            std::ffi::OsStr::new("-fPIC"),
                            std::ffi::OsStr::new("-o"),
                            dll_path.as_os_str(),
                            c_path.as_os_str(),
                        ])
                        .status()
                        .map_err(|e| anyhow!("Failed to execute gcc. Is it installed? {:?}", e))?;
                    if !status.success() {
                        return Err(anyhow!("gcc failed to compile C FFI code"));
                    }
                } else if lang == "nasm" {
                    let asm_path = temp_dir.join(format!("tmp_ffi_{}.asm", uuid_str));
                    let obj_path = temp_dir.join(format!("tmp_ffi_{}.obj", uuid_str));
                    std::fs::write(&asm_path, body)
                        .map_err(|e| anyhow!("Failed to write NASM FFI code: {}", e))?;

                    let status = std::process::Command::new("nasm")
                        .args(&[
                            std::ffi::OsStr::new("-f"),
                            std::ffi::OsStr::new("win64"),
                            asm_path.as_os_str(),
                            std::ffi::OsStr::new("-o"),
                            obj_path.as_os_str(),
                        ])
                        .status()
                        .map_err(|e| anyhow!("Failed to execute nasm. Is it installed? {:?}", e))?;
                    if !status.success() {
                        return Err(anyhow!("nasm failed to assemble FFI code"));
                    }

                    let link_status = std::process::Command::new("gcc")
                        .args(&[
                            std::ffi::OsStr::new("-shared"),
                            obj_path.as_os_str(),
                            std::ffi::OsStr::new("-o"),
                            dll_path.as_os_str(),
                        ])
                        .status()
                        .map_err(|e| {
                            anyhow!("Failed to execute gcc for linking nasm object. {:?}", e)
                        })?;
                    if !link_status.success() {
                        return Err(anyhow!("gcc failed to link NASM object code"));
                    }
                } else {
                    return Err(anyhow!("Unsupported FFI language: {}", lang));
                }

                let lib = unsafe {
                    Arc::new(
                        libloading::Library::new(&dll_path)
                            .map_err(|e| anyhow!("Failed to load dynamic library: {}", e))?,
                    )
                };
                self.libraries.write().unwrap().push(lib.clone());

                for decl in declarations {
                    let fn_name = decl.name.clone();
                    let num_args = decl.params.len();
                    let ret_ty = decl.ret_ty.clone();

                    let lib_clone = lib.clone();

                    let handler = Arc::new(move |args: Vec<PeelValue>| {
                        let lib_c = lib_clone.clone();
                        let f_name = fn_name.clone();
                        let n_args = num_args;
                        let r_ty = ret_ty.clone();

                        Box::pin(async move {
                            unsafe {
                                let func_ptr: libloading::Symbol<*const ()> = lib_c
                                    .get(f_name.as_bytes())
                                    .map_err(|e| anyhow!("DLSym failed for {}: {}", f_name, e))?;
                                let ptr = *func_ptr;

                                let mut int_args: [i64; 6] = [0; 6];
                                for (i, arg) in args.iter().enumerate().take(6) {
                                    int_args[i] = match arg {
                                        PeelValue::Int(v) => *v,
                                        PeelValue::Float(v) => v.to_bits() as i64,
                                        PeelValue::Bool(true) => 1,
                                        PeelValue::Bool(false) => 0,
                                        _ => 0,
                                    };
                                }

                                if r_ty == crate::ast::types::PeelType::Float {
                                    let res: f64 = match n_args {
                                        0 => {
                                            let f: unsafe extern "C" fn() -> f64 =
                                                std::mem::transmute(ptr);
                                            f()
                                        }
                                        1 => {
                                            let f: unsafe extern "C" fn(i64) -> f64 =
                                                std::mem::transmute(ptr);
                                            f(int_args[0])
                                        }
                                        2 => {
                                            let f: unsafe extern "C" fn(i64, i64) -> f64 =
                                                std::mem::transmute(ptr);
                                            f(int_args[0], int_args[1])
                                        }
                                        3 => {
                                            let f: unsafe extern "C" fn(i64, i64, i64) -> f64 =
                                                std::mem::transmute(ptr);
                                            f(int_args[0], int_args[1], int_args[2])
                                        }
                                        4 => {
                                            let f: unsafe extern "C" fn(i64, i64, i64, i64) -> f64 =
                                                std::mem::transmute(ptr);
                                            f(int_args[0], int_args[1], int_args[2], int_args[3])
                                        }
                                        _ => {
                                            return Err(anyhow!(
                                                "FFI functions currently support up to 4 arguments"
                                            ));
                                        }
                                    };
                                    Ok(PeelValue::Float(res))
                                } else {
                                    let res: i64 = match n_args {
                                        0 => {
                                            let f: unsafe extern "C" fn() -> i64 =
                                                std::mem::transmute(ptr);
                                            f()
                                        }
                                        1 => {
                                            let f: unsafe extern "C" fn(i64) -> i64 =
                                                std::mem::transmute(ptr);
                                            f(int_args[0])
                                        }
                                        2 => {
                                            let f: unsafe extern "C" fn(i64, i64) -> i64 =
                                                std::mem::transmute(ptr);
                                            f(int_args[0], int_args[1])
                                        }
                                        3 => {
                                            let f: unsafe extern "C" fn(i64, i64, i64) -> i64 =
                                                std::mem::transmute(ptr);
                                            f(int_args[0], int_args[1], int_args[2])
                                        }
                                        4 => {
                                            let f: unsafe extern "C" fn(i64, i64, i64, i64) -> i64 =
                                                std::mem::transmute(ptr);
                                            f(int_args[0], int_args[1], int_args[2], int_args[3])
                                        }
                                        _ => {
                                            return Err(anyhow!(
                                                "FFI functions currently support up to 4 arguments"
                                            ));
                                        }
                                    };
                                    Ok(PeelValue::Int(res))
                                }
                            }
                        }) as futures::future::BoxFuture<'static, _>
                    });

                    self.env.write().unwrap().define(
                        decl.name.clone(),
                        PeelValue::NativeFunction(Arc::new(crate::runtime::value::NativeFunc {
                            name: decl.name.clone(),
                            handler,
                        })),
                    );
                }

                Ok(PeelValue::Void)
            }
            Stmt::Import { path, symbols } => {
                let resolved_path = self.resolve_module_path(path)?;
                let path_str = resolved_path.to_string_lossy().to_string();

                let module_env = {
                    let cache = self.module_cache.read().unwrap();
                    cache.get(&path_str).cloned()
                };

                let target_env = if let Some(env) = module_env {
                    env
                } else {
                    let source = std::fs::read_to_string(&resolved_path)
                        .map_err(|e| anyhow!("Failed to read module '{}': {}", path_str, e))?;

                    let mut parser = crate::parser::Parser::new(&source, &path_str);
                    let module = parser.parse_module()?;

                    let new_env = Arc::new(RwLock::new(Environment::new()));
                    crate::stdlib::register_stdlib(new_env.clone(), self.methods.clone());

                    let mut sub_interpreter = Interpreter {
                        env: new_env.clone(),
                        structs: self.structs.clone(),
                        methods: self.methods.clone(),
                        module_cache: self.module_cache.clone(),
                        current_path: resolved_path
                            .parent()
                            .unwrap_or(std::path::Path::new("."))
                            .to_path_buf(),
                        libraries: self.libraries.clone(),
                    };

                    for stmt in &module.stmts {
                        sub_interpreter.eval_stmt(stmt).await?;
                    }

                    self.module_cache
                        .write()
                        .unwrap()
                        .insert(path_str, new_env.clone());
                    new_env
                };

                let mut current_env = self.env.write().unwrap();
                let module_env_lock = target_env.read().unwrap();

                if let Some(syms) = symbols {
                    for sym in syms {
                        if module_env_lock.exports.contains(sym) {
                            if let Some(val) = module_env_lock.values.get(sym) {
                                current_env.define(sym.clone(), val.clone());
                            }
                        } else {
                            return Err(anyhow!(
                                "Module '{}' does not export symbol '{}'",
                                path,
                                sym
                            ));
                        }
                    }
                } else {
                    for sym in &module_env_lock.exports {
                        if let Some(val) = module_env_lock.values.get(sym) {
                            current_env.define(sym.clone(), val.clone());
                        }
                    }
                }
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
            Expr::Ident(name) => self
                .env
                .read()
                .unwrap()
                .get(name)
                .ok_or(anyhow!("Undefined variable '{}'", name)),
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
                    (PeelValue::String(a), b) => match op {
                        Op::Add => Ok(PeelValue::String(format!(
                            "{}{}",
                            a,
                            peel_value_to_string(&b)
                        ))),
                        _ => Err(anyhow!("Invalid operator for string and non-string")),
                    },
                    (a, PeelValue::String(b)) => match op {
                        Op::Add => Ok(PeelValue::String(format!(
                            "{}{}",
                            peel_value_to_string(&a),
                            b
                        ))),
                        _ => Err(anyhow!("Invalid operator for non-string and string")),
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
                        PeelValue::Object {
                            struct_name: Some(s_name),
                            ..
                        } => Some(s_name.clone()),
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
                            let child_env =
                                Arc::new(RwLock::new(Environment::child(self.env.clone())));
                            for (i, param_name) in f.params.iter().enumerate() {
                                child_env
                                    .write()
                                    .unwrap()
                                    .define(param_name.clone(), arg_vals[i].clone());
                            }
                            let mut sub_interpreter = Interpreter {
                                env: child_env,
                                structs: self.structs.clone(),
                                methods: self.methods.clone(),
                                module_cache: self.module_cache.clone(),
                                current_path: self.current_path.clone(),
                                libraries: self.libraries.clone(),
                            };
                            let mut last = PeelValue::Void;
                            for stmt in &f.body {
                                last = sub_interpreter.eval_stmt(stmt).await?;
                                if let PeelValue::Return(v) = last {
                                    return Ok(*v);
                                }
                            }
                            Ok(last)
                        }
                        PeelValue::NativeFunction(f) => (f.handler)(arg_vals).await,
                        _ => Err(anyhow!("Method is not a function")),
                    }
                } else {
                    let func_val = self.eval_expr(callee).await?;
                    let mut arg_vals = Vec::new();
                    for arg in args {
                        arg_vals.push(self.eval_expr(arg).await?);
                    }

                    match func_val {
                        PeelValue::Function(f) => {
                            let child_env =
                                Arc::new(RwLock::new(Environment::child(self.env.clone())));
                            for (i, param) in f.params.iter().enumerate() {
                                child_env
                                    .write()
                                    .unwrap()
                                    .define(param.clone(), arg_vals[i].clone());
                            }
                            let mut sub_interpreter = Interpreter {
                                env: child_env,
                                structs: self.structs.clone(),
                                methods: self.methods.clone(),
                                module_cache: self.module_cache.clone(),
                                current_path: self.current_path.clone(),
                                libraries: self.libraries.clone(),
                            };
                            let mut last = PeelValue::Void;
                            for stmt in &f.body {
                                last = sub_interpreter.eval_stmt(stmt).await?;
                                if let PeelValue::Return(v) = last {
                                    return Ok(*v);
                                }
                            }
                            Ok(last)
                        }
                        PeelValue::NativeFunction(f) => (f.handler)(arg_vals).await,
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
                    Ok(lock
                        .get(field)
                        .cloned()
                        .ok_or(anyhow!("Field '{}' not found", field))?)
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
                    _ => Err(anyhow!("Invalid index operation: {:?}[{:?}]", t_val, i_val)),
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
                    "Ok" => Ok(PeelValue::Result(Ok(
                        val.unwrap_or(Box::new(PeelValue::Void))
                    ))),
                    "Err" => Ok(PeelValue::Result(Err(
                        val.unwrap_or(Box::new(PeelValue::Void))
                    ))),
                    "Some" => Ok(PeelValue::Option(Some(
                        val.unwrap_or(Box::new(PeelValue::Void)),
                    ))),
                    _ => Ok(PeelValue::Enum(name.clone(), val)),
                }
            }
            Expr::TypeCast { expr, .. } => self.eval_expr(expr).await,
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
            (PeelValue::Result(res), Pattern::Enum { name, inner }) => match (res, name.as_str()) {
                (Ok(v), "Ok") => {
                    if let Some(p) = inner {
                        self.match_pattern(v, p)
                    } else {
                        true
                    }
                }
                (Err(v), "Err") => {
                    if let Some(p) = inner {
                        self.match_pattern(v, p)
                    } else {
                        true
                    }
                }
                _ => false,
            },
            _ => false,
        }
    }

    fn resolve_module_path(&self, path: &str) -> RuntimeResult<std::path::PathBuf> {
        if path.starts_with("./") || path.starts_with("../") {
            let mut full_path = self.current_path.clone();
            full_path.push(path);
            if !full_path.exists() && full_path.extension().is_none() {
                full_path.set_extension("pel");
            }
            if full_path.exists() {
                return Ok(full_path.canonicalize()?);
            }
        } else {
            // Check .peel/modules
            let mut base_mod_path = std::env::current_dir().unwrap_or_default();
            base_mod_path.push(".peel");
            base_mod_path.push("modules");
            base_mod_path.push(path);

            // Try [path].pel
            let mut file_path = base_mod_path.clone();
            file_path.set_extension("pel");
            if file_path.exists() {
                return Ok(file_path.canonicalize()?);
            }

            // Try [path]/index.pel
            let mut index_path = base_mod_path.clone();
            index_path.push("index.pel");
            if index_path.exists() {
                return Ok(index_path.canonicalize()?);
            }
        }
        Err(anyhow!("Module not found: {}", path))
    }
}

fn peel_value_to_string(val: &PeelValue) -> String {
    match val {
        PeelValue::Int(i) => i.to_string(),
        PeelValue::Float(f) => f.to_string(),
        PeelValue::String(s) => s.clone(),
        PeelValue::Bool(b) => b.to_string(),
        PeelValue::Void => "void".to_string(),
        PeelValue::List(l) => format!("{:?}", l.lock().unwrap()),
        PeelValue::Map(m) => format!("{:?}", m.lock().unwrap()),
        PeelValue::Object { struct_name, .. } => {
            if let Some(name) = struct_name {
                name.clone()
            } else {
                "object".to_string()
            }
        }
        PeelValue::Function(f) => format!("<fn {}>", f.name),
        _ => "unknown".to_string(),
    }
}
