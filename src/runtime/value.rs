use crate::ast::Stmt;
use futures::future::BoxFuture;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct PeelIterator(pub Arc<Mutex<Box<dyn std::iter::Iterator<Item = PeelValue> + Send + Sync>>>);

impl std::fmt::Debug for PeelIterator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PeelIterator")
    }
}

// Add GeneratorState clone/debug manually if needed, but it's defined in interpreter.rs
// I'll assume GeneratorState is Arc<Mutex<GeneratorState>> which is cloneable.

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
    Set(Arc<Mutex<std::collections::HashSet<PeelValue>>>),
    WeakMap(Arc<Mutex<HashMap<usize, PeelValue>>>),
    WeakSet(Arc<Mutex<std::collections::HashSet<usize>>>),
    Iterator(PeelIterator),
    Generator(Arc<Mutex<crate::runtime::interpreter::GeneratorState>>),
}

impl PeelValue {
    pub fn as_int(&self) -> Option<i64> {
        match self {
            PeelValue::Int(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_float(&self) -> Option<f64> {
        match self {
            PeelValue::Float(f) => Some(*f),
            PeelValue::Int(i) => Some(*i as f64),
            _ => None,
        }
    }

    pub fn as_string(&self) -> Option<String> {
        match self {
            PeelValue::String(s) => Some(s.clone()),
            _ => None,
        }
    }
}

impl Eq for PeelValue {}

impl std::hash::Hash for PeelValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            PeelValue::Int(i) => {
                0u8.hash(state);
                i.hash(state);
            }
            PeelValue::Float(f) => {
                1u8.hash(state);
                f.to_bits().hash(state);
            }
            PeelValue::String(s) => {
                2u8.hash(state);
                s.hash(state);
            }
            PeelValue::Bool(b) => {
                3u8.hash(state);
                b.hash(state);
            }
            PeelValue::Void => {
                4u8.hash(state);
            }
            PeelValue::List(l) => {
                5u8.hash(state);
                std::ptr::hash(Arc::as_ptr(l), state);
            }
            PeelValue::Map(m) => {
                6u8.hash(state);
                std::ptr::hash(Arc::as_ptr(m), state);
            }
            PeelValue::Object { fields, .. } => {
                7u8.hash(state);
                std::ptr::hash(Arc::as_ptr(fields), state);
            }
            PeelValue::Function(f) => {
                8u8.hash(state);
                std::ptr::hash(Arc::as_ptr(f), state);
            }
            PeelValue::Generator(g) => {
                8u8.hash(state);
                std::ptr::hash(Arc::as_ptr(g), state);
            }
            PeelValue::Iterator(i) => {
                10u8.hash(state);
                std::ptr::hash(Arc::as_ptr(&i.0), state);
            }
            _ => {
                9u8.hash(state);
            }
        }
    }
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
            (PeelValue::Set(a), PeelValue::Set(b)) => {
                Arc::ptr_eq(a, b) || *a.lock().unwrap() == *b.lock().unwrap()
            }
            (PeelValue::Object { fields: a, .. }, PeelValue::Object { fields: b, .. }) => {
                Arc::ptr_eq(a, b) || *a.lock().unwrap() == *b.lock().unwrap()
            }
            (PeelValue::WeakMap(a), PeelValue::WeakMap(b)) => {
                Arc::ptr_eq(a, b) || *a.lock().unwrap() == *b.lock().unwrap()
            }
            (PeelValue::WeakSet(a), PeelValue::WeakSet(b)) => {
                Arc::ptr_eq(a, b) || *a.lock().unwrap() == *b.lock().unwrap()
            }
            (PeelValue::Iterator(a), PeelValue::Iterator(b)) => Arc::ptr_eq(&a.0, &b.0),
            (PeelValue::Generator(a), PeelValue::Generator(b)) => Arc::ptr_eq(a, b),
            (PeelValue::Void, PeelValue::Void) => true,
            _ => false,
        }
    }
}

#[allow(dead_code)]
pub struct PeelFunc {
    pub name: String,
    pub params: Vec<crate::ast::Param>,
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
