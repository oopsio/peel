use crate::runtime::value::{PeelValue, NativeFunc};
use crate::runtime::interpreter::RuntimeResult;
use std::sync::Arc;
use std::time::Instant;
use std::collections::HashMap;
use std::sync::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    static ref TIMERS: Mutex<HashMap<String, Instant>> = Mutex::new(HashMap::new());
}

pub fn register() -> HashMap<String, PeelValue> {
    let mut methods = HashMap::new();

    methods.insert("log".to_string(), create_log_fn("LOG"));
    methods.insert("info".to_string(), create_log_fn("INFO"));
    methods.insert("warn".to_string(), create_log_fn("WARN"));
    methods.insert("error".to_string(), create_log_fn("ERROR"));

    methods.insert("clear".to_string(), PeelValue::NativeFunction(Arc::new(NativeFunc {
        name: "clear".to_string(),
        handler: Arc::new(|_| Box::pin(async move {
            print!("\x1B[2J\x1B[1;1H");
            Ok(PeelValue::Void)
        })),
    })));

    methods.insert("time".to_string(), PeelValue::NativeFunction(Arc::new(NativeFunc {
        name: "time".to_string(),
        handler: Arc::new(|args| Box::pin(async move {
            let label = if let Some(PeelValue::String(s)) = args.get(0) { s.clone() } else { "default".to_string() };
            TIMERS.lock().unwrap().insert(label, Instant::now());
            Ok(PeelValue::Void)
        })),
    })));

    methods.insert("timeEnd".to_string(), PeelValue::NativeFunction(Arc::new(NativeFunc {
        name: "timeEnd".to_string(),
        handler: Arc::new(|args| Box::pin(async move {
            let label = if let Some(PeelValue::String(s)) = args.get(0) { s.clone() } else { "default".to_string() };
            if let Some(start) = TIMERS.lock().unwrap().remove(&label) {
                println!("{}: {}ms", label, start.elapsed().as_millis());
            }
            Ok(PeelValue::Void)
        })),
    })));

    methods
}

fn create_log_fn(level: &'static str) -> PeelValue {
    PeelValue::NativeFunction(Arc::new(NativeFunc {
        name: level.to_lowercase(),
        handler: Arc::new(move |args| Box::pin(async move {
            if level != "LOG" { print!("[{}] ", level); }
            for arg in args {
                match arg {
                    PeelValue::String(s) => print!("{} ", s),
                    PeelValue::Int(i) => print!("{} ", i),
                    PeelValue::Float(f) => print!("{} ", f),
                    PeelValue::Bool(b) => print!("{} ", b),
                    _ => print!("{:?} ", arg),
                }
            }
            println!();
            Ok(PeelValue::Void)
        })),
    }))
}
