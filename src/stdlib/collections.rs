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

pub fn register_map() -> HashMap<String, PeelValue> {
    let mut methods = HashMap::new();
    methods.insert(
        "set".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "set".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let (Some(PeelValue::Map(m)), Some(PeelValue::String(k)), Some(v)) = (args.get(0), args.get(1), args.get(2)) {
                        m.lock().unwrap().insert(k.clone(), v.clone());
                        Ok(PeelValue::Void)
                    } else {
                        Err(anyhow!("Map.set expects a Map instance, a string key, and a value"))
                    }
                })
            }),
        })),
    );
    methods.insert(
        "get".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "get".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let (Some(PeelValue::Map(m)), Some(PeelValue::String(k))) = (args.get(0), args.get(1)) {
                        Ok(m.lock().unwrap().get(k).cloned().unwrap_or(PeelValue::Void))
                    } else {
                        Err(anyhow!("Map.get expects a Map instance and a string key"))
                    }
                })
            }),
        })),
    );
    methods.insert(
        "has".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "has".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let (Some(PeelValue::Map(m)), Some(PeelValue::String(k))) = (args.get(0), args.get(1)) {
                        Ok(PeelValue::Bool(m.lock().unwrap().contains_key(k)))
                    } else {
                        Err(anyhow!("Map.has expects a Map instance and a string key"))
                    }
                })
            }),
        })),
    );
    methods.insert(
        "delete".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "delete".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let (Some(PeelValue::Map(m)), Some(PeelValue::String(k))) = (args.get(0), args.get(1)) {
                        Ok(PeelValue::Bool(m.lock().unwrap().remove(k).is_some()))
                    } else {
                        Err(anyhow!("Map.delete expects a Map instance and a string key"))
                    }
                })
            }),
        })),
    );
    methods
}

pub fn register_set() -> HashMap<String, PeelValue> {
    let mut methods = HashMap::new();
    methods.insert(
        "add".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "add".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let (Some(PeelValue::Set(s)), Some(val)) = (args.get(0), args.get(1)) {
                        s.lock().unwrap().insert(val.clone());
                        Ok(PeelValue::Void)
                    } else {
                        Err(anyhow!("Set.add expects a Set instance and a value"))
                    }
                })
            }),
        })),
    );
    methods.insert(
        "has".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "has".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let (Some(PeelValue::Set(s)), Some(val)) = (args.get(0), args.get(1)) {
                        Ok(PeelValue::Bool(s.lock().unwrap().contains(val)))
                    } else {
                        Err(anyhow!("Set.has expects a Set instance and a value"))
                    }
                })
            }),
        })),
    );
    methods.insert(
        "delete".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "delete".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let (Some(PeelValue::Set(s)), Some(val)) = (args.get(0), args.get(1)) {
                        Ok(PeelValue::Bool(s.lock().unwrap().remove(val)))
                    } else {
                        Err(anyhow!("Set.delete expects a Set instance and a value"))
                    }
                })
            }),
        })),
    );
    methods
}

fn get_obj_id(obj: &PeelValue) -> Result<usize, anyhow::Error> {
    match obj {
        PeelValue::Object { fields, .. } => Ok(Arc::as_ptr(fields) as usize),
        PeelValue::List(l) => Ok(Arc::as_ptr(l) as usize),
        PeelValue::Map(m) => Ok(Arc::as_ptr(m) as usize),
        PeelValue::Set(s) => Ok(Arc::as_ptr(s) as usize),
        PeelValue::Function(f) => Ok(Arc::as_ptr(f) as usize),
        _ => Err(anyhow!("WeakMap/WeakSet keys must be objects")),
    }
}

pub fn register_weak_map() -> HashMap<String, PeelValue> {
    let mut methods = HashMap::new();
    methods.insert(
        "set".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "set".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let (Some(PeelValue::WeakMap(m)), Some(key), Some(v)) = (args.get(0), args.get(1), args.get(2)) {
                        let id = get_obj_id(key)?;
                        m.lock().unwrap().insert(id, v.clone());
                        Ok(PeelValue::Void)
                    } else {
                        Err(anyhow!("WeakMap.set expects a WeakMap instance, an object key, and a value"))
                    }
                })
            }),
        })),
    );
    methods.insert(
        "get".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "get".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let (Some(PeelValue::WeakMap(m)), Some(key)) = (args.get(0), args.get(1)) {
                        let id = get_obj_id(key)?;
                        Ok(m.lock().unwrap().get(&id).cloned().unwrap_or(PeelValue::Void))
                    } else {
                        Err(anyhow!("WeakMap.get expects a WeakMap instance and an object key"))
                    }
                })
            }),
        })),
    );
    methods.insert(
        "has".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "has".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let (Some(PeelValue::WeakMap(m)), Some(key)) = (args.get(0), args.get(1)) {
                        let id = get_obj_id(key)?;
                        Ok(PeelValue::Bool(m.lock().unwrap().contains_key(&id)))
                    } else {
                        Err(anyhow!("WeakMap.has expects a WeakMap instance and an object key"))
                    }
                })
            }),
        })),
    );
    methods.insert(
        "delete".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "delete".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let (Some(PeelValue::WeakMap(m)), Some(key)) = (args.get(0), args.get(1)) {
                        let id = get_obj_id(key)?;
                        Ok(PeelValue::Bool(m.lock().unwrap().remove(&id).is_some()))
                    } else {
                        Err(anyhow!("WeakMap.delete expects a WeakMap instance and an object key"))
                    }
                })
            }),
        })),
    );
    methods
}

pub fn register_weak_set() -> HashMap<String, PeelValue> {
    let mut methods = HashMap::new();
    methods.insert(
        "add".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "add".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let (Some(PeelValue::WeakSet(s)), Some(val)) = (args.get(0), args.get(1)) {
                        let id = get_obj_id(val)?;
                        s.lock().unwrap().insert(id);
                        Ok(PeelValue::Void)
                    } else {
                        Err(anyhow!("WeakSet.add expects a WeakSet instance and an object value"))
                    }
                })
            }),
        })),
    );
    methods.insert(
        "has".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "has".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let (Some(PeelValue::WeakSet(s)), Some(val)) = (args.get(0), args.get(1)) {
                        let id = get_obj_id(val)?;
                        Ok(PeelValue::Bool(s.lock().unwrap().contains(&id)))
                    } else {
                        Err(anyhow!("WeakSet.has expects a WeakSet instance and an object value"))
                    }
                })
            }),
        })),
    );
    methods.insert(
        "delete".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "delete".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let (Some(PeelValue::WeakSet(s)), Some(val)) = (args.get(0), args.get(1)) {
                        let id = get_obj_id(val)?;
                        Ok(PeelValue::Bool(s.lock().unwrap().remove(&id)))
                    } else {
                        Err(anyhow!("WeakSet.delete expects a WeakSet instance and an object value"))
                    }
                })
            }),
        })),
    );
    methods
}
