use crate::runtime::value::{NativeFunc, PeelValue};
use rand::Rng;
use std::collections::HashMap;
use std::sync::Arc;

pub fn register() -> HashMap<String, PeelValue> {
    let mut methods = HashMap::new();

    // Constants
    methods.insert("PI".to_string(), PeelValue::Float(std::f64::consts::PI));
    methods.insert("E".to_string(), PeelValue::Float(std::f64::consts::E));

    // Methods
    methods.insert("abs".to_string(), create_math_fn_1("abs", |x| x.abs()));
    methods.insert("sqrt".to_string(), create_math_fn_1("sqrt", |x| x.sqrt()));
    methods.insert("sin".to_string(), create_math_fn_1("sin", |x| x.sin()));
    methods.insert("cos".to_string(), create_math_fn_1("cos", |x| x.cos()));
    methods.insert("tan".to_string(), create_math_fn_1("tan", |x| x.tan()));
    methods.insert(
        "floor".to_string(),
        create_math_fn_1("floor", |x| x.floor()),
    );
    methods.insert("ceil".to_string(), create_math_fn_1("ceil", |x| x.ceil()));
    methods.insert(
        "round".to_string(),
        create_math_fn_1("round", |x| x.round()),
    );
    methods.insert("log".to_string(), create_math_fn_1("log", |x| x.ln()));

    methods.insert(
        "pow".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "pow".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    let base = match args.get(0) {
                        Some(PeelValue::Float(f)) => *f,
                        Some(PeelValue::Int(i)) => *i as f64,
                        _ => return Err(anyhow::anyhow!("Math.pow expects two numbers")),
                    };
                    let exp = match args.get(1) {
                        Some(PeelValue::Float(f)) => *f,
                        Some(PeelValue::Int(i)) => *i as f64,
                        _ => return Err(anyhow::anyhow!("Math.pow expects two numbers")),
                    };
                    Ok(PeelValue::Float(base.powf(exp)))
                })
            }),
        })),
    );

    methods.insert(
        "random".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "random".to_string(),
            handler: Arc::new(|_| {
                Box::pin(async move {
                    let mut rng = rand::thread_rng();
                    Ok(PeelValue::Float(rng.r#gen()))
                })
            }),
        })),
    );

    methods
}

fn create_math_fn_1(name: &'static str, f: fn(f64) -> f64) -> PeelValue {
    PeelValue::NativeFunction(Arc::new(NativeFunc {
        name: name.to_string(),
        handler: Arc::new(move |args| {
            Box::pin(async move {
                let val = match args.get(0) {
                    Some(PeelValue::Float(f)) => *f,
                    Some(PeelValue::Int(i)) => *i as f64,
                    _ => return Err(anyhow::anyhow!("Math.{} expects a number", name)),
                };
                Ok(PeelValue::Float(f(val)))
            })
        }),
    }))
}
