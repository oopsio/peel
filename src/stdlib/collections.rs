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

    methods.insert(
        "toLowerCase".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "toLowerCase".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let Some(PeelValue::String(s)) = args.get(0) {
                        Ok(PeelValue::String(s.to_lowercase()))
                    } else {
                        Err(anyhow!("String.toLowerCase expects a string instance"))
                    }
                })
            }),
        })),
    );

    methods.insert(
        "trim".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "trim".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let Some(PeelValue::String(s)) = args.get(0) {
                        Ok(PeelValue::String(s.trim().to_string()))
                    } else {
                        Err(anyhow!("String.trim expects a string instance"))
                    }
                })
            }),
        })),
    );

    methods.insert(
        "split".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "split".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let (Some(PeelValue::String(s)), Some(PeelValue::String(sep))) = (args.get(0), args.get(1)) {
                        let parts: Vec<PeelValue> = s.split(sep).map(|p| PeelValue::String(p.to_string())).collect();
                        Ok(PeelValue::List(Arc::new(std::sync::Mutex::new(parts))))
                    } else {
                        Err(anyhow!("String.split(separator: string) expects a string and a separator"))
                    }
                })
            }),
        })),
    );

    methods.insert(
        "replace".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "replace".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let (Some(PeelValue::String(s)), Some(PeelValue::String(old)), Some(PeelValue::String(new))) = (args.get(0), args.get(1), args.get(2)) {
                        Ok(PeelValue::String(s.replacen(old, new, 1)))
                    } else {
                        Err(anyhow!("String.replace(old, new) expects string arguments"))
                    }
                })
            }),
        })),
    );

    methods.insert(
        "replaceAll".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "replaceAll".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let (Some(PeelValue::String(s)), Some(PeelValue::String(old)), Some(PeelValue::String(new))) = (args.get(0), args.get(1), args.get(2)) {
                        Ok(PeelValue::String(s.replace(old, new)))
                    } else {
                        Err(anyhow!("String.replaceAll(old, new) expects string arguments"))
                    }
                })
            }),
        })),
    );

    methods.insert(
        "substring".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "substring".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let (Some(PeelValue::String(s)), Some(PeelValue::Int(start))) = (args.get(0), args.get(1)) {
                        let start_val = *start as usize;
                        let end_val = match args.get(2) {
                            Some(PeelValue::Int(e)) => *e as usize,
                            _ => s.len()
                        };
                        let start_idx = start_val.min(s.len());
                        let end_idx = end_val.min(s.len()).max(start_idx);
                        Ok(PeelValue::String(s[start_idx..end_idx].to_string()))
                    } else {
                        Err(anyhow!("String.substring(start, [end]) expects at least a start index"))
                    }
                })
            }),
        })),
    );

    methods.insert(
        "includes".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "includes".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let (Some(PeelValue::String(s)), Some(PeelValue::String(sub))) = (args.get(0), args.get(1)) {
                        Ok(PeelValue::Bool(s.contains(sub)))
                    } else {
                        Err(anyhow!("String.includes(substring) expects a string argument"))
                    }
                })
            }),
        })),
    );

    methods.insert(
        "repeat".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "repeat".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let (Some(PeelValue::String(s)), Some(PeelValue::Int(n))) = (args.get(0), args.get(1)) {
                        Ok(PeelValue::String(s.repeat(*n as usize)))
                    } else {
                        Err(anyhow!("String.repeat(n) expects an integer argument"))
                    }
                })
            }),
        })),
    );

    methods.insert(
        "test".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "test".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let (Some(PeelValue::String(s)), Some(PeelValue::String(pat))) = (args.get(0), args.get(1)) {
                        let re = regex::Regex::new(pat).map_err(|e| anyhow!("Invalid regex pattern: {}", e))?;
                        Ok(PeelValue::Bool(re.is_match(s)))
                    } else {
                        Err(anyhow!("String.test(pattern) expects a regex pattern string"))
                    }
                })
            }),
        })),
    );

    methods.insert(
        "match".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "match".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let (Some(PeelValue::String(s)), Some(PeelValue::String(pat))) = (args.get(0), args.get(1)) {
                        let re = regex::Regex::new(pat).map_err(|e| anyhow!("Invalid regex pattern: {}", e))?;
                        let matches: Vec<PeelValue> = re.captures_iter(s)
                            .map(|cap| PeelValue::String(cap[0].to_string()))
                            .collect();
                        Ok(PeelValue::List(Arc::new(std::sync::Mutex::new(matches))))
                    } else {
                        Err(anyhow!("String.match(pattern) expects a regex pattern string"))
                    }
                })
            }),
        })),
    );

    methods.insert(
        "padStart".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "padStart".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let (Some(PeelValue::String(s)), Some(PeelValue::Int(len))) = (args.get(0), args.get(1)) {
                        let pad_char = match args.get(2) {
                            Some(PeelValue::String(p)) => p.clone(),
                            _ => " ".to_string()
                        };
                        let target_len = *len as usize;
                        if s.len() >= target_len {
                            Ok(PeelValue::String(s.clone()))
                        } else {
                            let mut res = s.clone();
                            while res.len() < target_len {
                                res.insert_str(0, &pad_char);
                            }
                            if res.len() > target_len {
                                res = res[res.len() - target_len..].to_string();
                            }
                            Ok(PeelValue::String(res))
                        }
                    } else {
                        Err(anyhow!("String.padStart(length, [char]) expects at least a length"))
                    }
                })
            }),
        })),
    );

    methods.insert(
        "padEnd".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "padEnd".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let (Some(PeelValue::String(s)), Some(PeelValue::Int(len))) = (args.get(0), args.get(1)) {
                        let pad_char = match args.get(2) {
                            Some(PeelValue::String(p)) => p.clone(),
                            _ => " ".to_string()
                        };
                        let target_len = *len as usize;
                        if s.len() >= target_len {
                            Ok(PeelValue::String(s.clone()))
                        } else {
                            let mut res = s.clone();
                            while res.len() < target_len {
                                res.push_str(&pad_char);
                            }
                            res.truncate(target_len);
                            Ok(PeelValue::String(res))
                        }
                    } else {
                        Err(anyhow!("String.padEnd(length, [char]) expects at least a length"))
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
