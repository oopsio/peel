use crate::runtime::interpreter::Environment;
use crate::runtime::value::{NativeFunc, PeelValue};
use anyhow::anyhow;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use tokio::time::sleep;

pub mod collections;
pub mod console;
pub mod json;
pub mod math;
pub mod sqlite;

pub fn register_stdlib(
    env: Arc<RwLock<Environment>>,
    methods: Arc<RwLock<HashMap<String, HashMap<String, PeelValue>>>>,
) {
    let mut e = env.write().unwrap();
    let mut m = methods.write().unwrap();

    // 1. Global Objects (Static Methods)
    e.define(
        "console".to_string(),
        PeelValue::Object {
            struct_name: None,
            fields: Arc::new(Mutex::new(console::register())),
        },
    );

    e.define(
        "Math".to_string(),
        PeelValue::Object {
            struct_name: None,
            fields: Arc::new(Mutex::new(math::register())),
        },
    );

    e.define(
        "JSON".to_string(),
        PeelValue::Object {
            struct_name: None,
            fields: Arc::new(Mutex::new(json::register())),
        },
    );

    e.define(
        "sqlite".to_string(),
        PeelValue::Object {
            struct_name: None,
            fields: Arc::new(Mutex::new(sqlite::register())),
        },
    );

    // 2. Prototypes (Instance Methods)
    // 2. Prototypes (Instance Methods)
    m.insert("String".to_string(), collections::register_string());
    m.insert("Array".to_string(), collections::register_array());
    m.insert("Map".to_string(), collections::register_map());
    m.insert("Set".to_string(), collections::register_set());
    m.insert("WeakMap".to_string(), collections::register_weak_map());
    m.insert("WeakSet".to_string(), collections::register_weak_set());

    // 2.5 Built-in Constructors
    e.define("Map".to_string(), PeelValue::NativeFunction(Arc::new(NativeFunc {
        name: "Map".to_string(),
        handler: Arc::new(|_args| {
            Box::pin(async move {
                Ok(PeelValue::Map(Arc::new(Mutex::new(HashMap::new()))))
            })
        }),
    })));

    e.define("Set".to_string(), PeelValue::NativeFunction(Arc::new(NativeFunc {
        name: "Set".to_string(),
        handler: Arc::new(|_args| {
            Box::pin(async move {
                Ok(PeelValue::Set(Arc::new(Mutex::new(std::collections::HashSet::new()))))
            })
        }),
    })));

    e.define("WeakMap".to_string(), PeelValue::NativeFunction(Arc::new(NativeFunc {
        name: "WeakMap".to_string(),
        handler: Arc::new(|_args| {
            Box::pin(async move {
                Ok(PeelValue::WeakMap(Arc::new(Mutex::new(HashMap::new()))))
            })
        }),
    })));

    e.define("WeakSet".to_string(), PeelValue::NativeFunction(Arc::new(NativeFunc {
        name: "WeakSet".to_string(),
        handler: Arc::new(|_args| {
            Box::pin(async move {
                Ok(PeelValue::WeakSet(Arc::new(Mutex::new(std::collections::HashSet::new()))))
            })
        }),
    })));

    // 3. Built-in Modules
    // fmt module
    let mut fmt_methods = HashMap::new();
    fmt_methods.insert(
        "println".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "println".to_string(),
            handler: Arc::new(|args: Vec<PeelValue>| {
                Box::pin(async move {
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
                })
            }),
        })),
    );
    e.define(
        "fmt".to_string(),
        PeelValue::Object {
            struct_name: None,
            fields: Arc::new(Mutex::new(fmt_methods)),
        },
    );

    // time module
    let mut time_methods = HashMap::new();
    time_methods.insert(
        "sleep".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "sleep".to_string(),
            handler: Arc::new(|args: Vec<PeelValue>| {
                Box::pin(async move {
                    if let Some(PeelValue::Int(ms)) = args.get(0) {
                        sleep(Duration::from_millis(*ms as u64)).await;
                    }
                    Ok(PeelValue::Void)
                })
            }),
        })),
    );
    e.define(
        "time".to_string(),
        PeelValue::Object {
            struct_name: None,
            fields: Arc::new(Mutex::new(time_methods)),
        },
    );

    // http module
    let mut http_methods = HashMap::new();
    http_methods.insert(
        "get".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "get".to_string(),
            handler: Arc::new(|args: Vec<PeelValue>| {
                Box::pin(async move {
                    if let Some(PeelValue::String(url)) = args.get(0) {
                        let client = reqwest::Client::builder()
                            .user_agent("Peel-Agent/0.1")
                            .build()
                            .map_err(|e| anyhow!(e))?;
                        let res = client.get(url).send().await.map_err(|e| anyhow!(e))?;
                        let body = res.text().await.map_err(|e| anyhow!(e))?;
                        Ok(PeelValue::Result(Ok(Box::new(PeelValue::String(body)))))
                    } else {
                        Err(anyhow!("http.get expects a string URL"))
                    }
                })
            }),
        })),
    );
    e.define(
        "http".to_string(),
        PeelValue::Object {
            struct_name: None,
            fields: Arc::new(Mutex::new(http_methods)),
        },
    );

    // fs module
    let mut fs_methods = HashMap::new();
    fs_methods.insert(
        "read_to_string".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "read_to_string".to_string(),
            handler: Arc::new(|args: Vec<PeelValue>| {
                Box::pin(async move {
                    if let Some(PeelValue::String(path)) = args.get(0) {
                        let content = tokio::fs::read_to_string(path)
                            .await
                            .map_err(|e| anyhow!(e))?;
                        Ok(PeelValue::Result(Ok(Box::new(PeelValue::String(content)))))
                    } else {
                        Err(anyhow!("fs.read_to_string expects a string path"))
                    }
                })
            }),
        })),
    );
    fs_methods.insert(
        "write_to_string".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "write_to_string".to_string(),
            handler: Arc::new(|args: Vec<PeelValue>| {
                Box::pin(async move {
                    if let (Some(PeelValue::String(path)), Some(PeelValue::String(content))) =
                        (args.get(0), args.get(1))
                    {
                        tokio::fs::write(path, content)
                            .await
                            .map_err(|e| anyhow!(e))?;
                        Ok(PeelValue::Void)
                    } else {
                        Err(anyhow!(
                            "fs.write_to_string expects string path and content"
                        ))
                    }
                })
            }),
        })),
    );
    e.define(
        "fs".to_string(),
        PeelValue::Object {
            struct_name: None,
            fields: Arc::new(Mutex::new(fs_methods)),
        },
    );
}
