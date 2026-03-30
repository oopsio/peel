use crate::runtime::value::{NativeFunc, PeelValue};
use anyhow::anyhow;
use serde_json::{Number as JsonNumber, Value as JsonValue};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub fn register() -> HashMap<String, PeelValue> {
    let mut methods = HashMap::new();

    methods.insert(
        "parse".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "parse".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let Some(PeelValue::String(s)) = args.get(0) {
                        let json: JsonValue = serde_json::from_str(s).map_err(|e| anyhow!(e))?;
                        Ok(json_to_peel(json))
                    } else {
                        Err(anyhow!("JSON.parse expects a string"))
                    }
                })
            }),
        })),
    );

    methods.insert(
        "stringify".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "stringify".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let Some(val) = args.get(0) {
                        let json = peel_to_json(val);
                        let s = serde_json::to_string(&json).map_err(|e| anyhow!(e))?;
                        Ok(PeelValue::String(s))
                    } else {
                        Err(anyhow!("JSON.stringify expects a value"))
                    }
                })
            }),
        })),
    );

    methods
}

fn json_to_peel(json: JsonValue) -> PeelValue {
    match json {
        JsonValue::Null => PeelValue::Void,
        JsonValue::Bool(b) => PeelValue::Bool(b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                PeelValue::Int(i)
            } else {
                PeelValue::Float(n.as_f64().unwrap_or(0.0))
            }
        }
        JsonValue::String(s) => PeelValue::String(s),
        JsonValue::Array(arr) => PeelValue::List(Arc::new(Mutex::new(
            arr.into_iter().map(json_to_peel).collect(),
        ))),
        JsonValue::Object(obj) => {
            let mut fields = HashMap::new();
            for (k, v) in obj {
                fields.insert(k, json_to_peel(v));
            }
            PeelValue::Object {
                struct_name: None,
                fields: Arc::new(Mutex::new(fields)),
            }
        }
    }
}

fn peel_to_json(peel: &PeelValue) -> JsonValue {
    match peel {
        PeelValue::Void => JsonValue::Null,
        PeelValue::Bool(b) => JsonValue::Bool(*b),
        PeelValue::Int(i) => JsonValue::Number(JsonNumber::from(*i)),
        PeelValue::Float(f) => JsonNumber::from_f64(*f)
            .map(JsonValue::Number)
            .unwrap_or(JsonValue::Null),
        PeelValue::String(s) => JsonValue::String(s.clone()),
        PeelValue::List(arr) => {
            let arr = arr.lock().unwrap();
            JsonValue::Array(arr.iter().map(peel_to_json).collect())
        }
        PeelValue::Object { fields, .. } => {
            let mut obj = serde_json::Map::new();
            let fields = fields.lock().unwrap();
            for (k, v) in fields.iter() {
                obj.insert(k.clone(), peel_to_json(v));
            }
            JsonValue::Object(obj)
        }
        PeelValue::Map(map) => {
            let mut obj = serde_json::Map::new();
            let map = map.lock().unwrap();
            for (k, v) in map.iter() {
                obj.insert(k.clone(), peel_to_json(v));
            }
            JsonValue::Object(obj)
        }
        _ => JsonValue::Null,
    }
}
