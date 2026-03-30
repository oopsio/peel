use crate::runtime::value::{NativeFunc, PeelValue};
use anyhow::anyhow;
use std::collections::HashMap;
use std::sync::Arc;

pub fn register_string() -> HashMap<String, PeelValue> {
    let mut methods = HashMap::new();

    methods.insert(
        "toUpperCase".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "toUpperCase".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let Some(PeelValue::String(s)) = args.get(0) {
                        Ok(PeelValue::String(s.to_uppercase()))
                    } else {
                        Err(anyhow!("String.toUpperCase expects a string instance"))
                    }
                })
            }),
        })),
    );

    methods.insert(
        "len".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "len".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let Some(PeelValue::String(s)) = args.get(0) {
                        Ok(PeelValue::Int(s.len() as i64))
                    } else {
                        Err(anyhow!("String.len expects a string instance"))
                    }
                })
            }),
        })),
    );

    methods
}

pub fn register_array() -> HashMap<String, PeelValue> {
    let mut methods = HashMap::new();

    methods.insert(
        "len".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "len".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let Some(PeelValue::List(l)) = args.get(0) {
                        Ok(PeelValue::Int(l.lock().unwrap().len() as i64))
                    } else {
                        Err(anyhow!("Array.len expects an array instance"))
                    }
                })
            }),
        })),
    );

    methods.insert(
        "push".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "push".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let (Some(PeelValue::List(l)), Some(val)) = (args.get(0), args.get(1)) {
                        l.lock().unwrap().push(val.clone());
                        Ok(PeelValue::Void)
                    } else {
                        Err(anyhow!("Array.push expects an array instance and a value"))
                    }
                })
            }),
        })),
    );

    methods.insert(
        "pop".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "pop".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let Some(PeelValue::List(l)) = args.get(0) {
                        Ok(l.lock().unwrap().pop().unwrap_or(PeelValue::Void))
                    } else {
                        Err(anyhow!("Array.pop expects an array instance"))
                    }
                })
            }),
        })),
    );

    methods
}
