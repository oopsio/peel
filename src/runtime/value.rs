use crate::ast::Stmt;
use futures::future::BoxFuture;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum PeelValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Void,
    Option(Option<Box<PeelValue>>),
    Result(Result<Box<PeelValue>, Box<PeelValue>>),
    List(Arc<Mutex<Vec<PeelValue>>>),
    Map(Arc<Mutex<HashMap<String, PeelValue>>>),
    Object {
        struct_name: Option<String>,
        fields: Arc<Mutex<HashMap<String, PeelValue>>>,
    },
    Function(Arc<PeelFunc>),
    NativeFunction(Arc<NativeFunc>),
    Return(Box<PeelValue>),
    Enum(String, Option<Box<PeelValue>>),
}

impl PartialEq for PeelValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (PeelValue::Int(a), PeelValue::Int(b)) => a == b,
            (PeelValue::Float(a), PeelValue::Float(b)) => a == b,
            (PeelValue::String(a), PeelValue::String(b)) => a == b,
            (PeelValue::Bool(a), PeelValue::Bool(b)) => a == b,
            (PeelValue::List(a), PeelValue::List(b)) => {
                Arc::ptr_eq(a, b) || *a.lock().unwrap() == *b.lock().unwrap()
            }
            (PeelValue::Map(a), PeelValue::Map(b)) => {
                Arc::ptr_eq(a, b) || *a.lock().unwrap() == *b.lock().unwrap()
            }
            (PeelValue::Void, PeelValue::Void) => true,
            _ => false,
        }
    }
}

#[allow(dead_code)]
pub struct PeelFunc {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
    pub _is_async: bool,
}

impl std::fmt::Debug for PeelFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PeelFunc")
            .field("name", &self.name)
            .finish()
    }
}

pub type NativeFn = Arc<
    dyn Fn(
            Vec<PeelValue>,
        ) -> BoxFuture<'static, crate::runtime::interpreter::RuntimeResult<PeelValue>>
        + Send
        + Sync,
>;

pub struct NativeFunc {
    pub name: String,
    pub handler: NativeFn,
}

impl std::fmt::Debug for NativeFunc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeFunc")
            .field("name", &self.name)
            .finish()
    }
}
