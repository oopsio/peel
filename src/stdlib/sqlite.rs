use crate::runtime::value::{NativeFunc, PeelValue};
use anyhow::anyhow;
use rusqlite::{Connection, params_from_iter};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub fn register() -> HashMap<String, PeelValue> {
    let mut methods = HashMap::new();

    methods.insert(
        "open".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "open".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let Some(PeelValue::String(path)) = args.get(0) {
                        let conn = if path == ":memory:" {
                            Connection::open_in_memory().map_err(|e| anyhow!(e))?
                        } else {
                            Connection::open(path).map_err(|e| anyhow!(e))?
                        };

                        Ok(create_connection_object(conn))
                    } else {
                        Err(anyhow!("sqlite.open expects a string path"))
                    }
                })
            }),
        })),
    );

    methods
}

fn create_connection_object(conn: Connection) -> PeelValue {
    let conn = Arc::new(Mutex::new(Some(conn)));
    let mut fields = HashMap::new();

    // Internal method to get the connection, marked with _
    let conn_clone = conn.clone();
    fields.insert(
        "_get_conn".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "_get_conn".to_string(),
            handler: Arc::new(move |_| {
                let _conn = conn_clone.clone();
                Box::pin(async move {
                    Err(anyhow!(
                        "Internal method _get_conn cannot be called directly from Peel"
                    ))
                })
            }),
        })),
    );

    let conn_execute = conn.clone();
    fields.insert(
        "execute".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "execute".to_string(),
            handler: Arc::new(move |args| {
                let conn = conn_execute.clone();
                Box::pin(async move {
                    let sql = match args.get(0) {
                        Some(PeelValue::String(s)) => s,
                        _ => {
                            return Err(anyhow!(
                                "Connection.execute expects a SQL string as first argument"
                            ));
                        }
                    };

                    let params_val = args
                        .get(1)
                        .cloned()
                        .unwrap_or(PeelValue::List(Arc::new(Mutex::new(vec![]))));
                    let rusql_params = peel_to_rusqlite_params(params_val)?;

                    let conn_lock = conn.lock().unwrap();
                    let c = conn_lock
                        .as_ref()
                        .ok_or_else(|| anyhow!("Connection is closed"))?;

                    let rows = c
                        .execute(sql, params_from_iter(rusql_params))
                        .map_err(|e| anyhow!(e))?;
                    Ok(PeelValue::Int(rows as i64))
                })
            }),
        })),
    );

    let conn_query = conn.clone();
    fields.insert(
        "query".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "query".to_string(),
            handler: Arc::new(move |args| {
                let conn = conn_query.clone();
                Box::pin(async move {
                    let sql = match args.get(0) {
                        Some(PeelValue::String(s)) => s,
                        _ => {
                            return Err(anyhow!(
                                "Connection.query expects a SQL string as first argument"
                            ));
                        }
                    };

                    let params_val = args
                        .get(1)
                        .cloned()
                        .unwrap_or(PeelValue::List(Arc::new(Mutex::new(vec![]))));
                    let rusql_params = peel_to_rusqlite_params(params_val)?;

                    let conn_lock = conn.lock().unwrap();
                    let c = conn_lock
                        .as_ref()
                        .ok_or_else(|| anyhow!("Connection is closed"))?;

                    let mut stmt = c.prepare(sql).map_err(|e| anyhow!(e))?;
                    let names: Vec<String> =
                        stmt.column_names().iter().map(|s| s.to_string()).collect();

                    let rows = stmt
                        .query_map(params_from_iter(rusql_params), |row| {
                            let mut map = HashMap::new();
                            for (i, name) in names.iter().enumerate() {
                                let val: PeelValue = match row.get_ref(i)? {
                                    rusqlite::types::ValueRef::Null => PeelValue::Void,
                                    rusqlite::types::ValueRef::Integer(i) => PeelValue::Int(i),
                                    rusqlite::types::ValueRef::Real(f) => PeelValue::Float(f),
                                    rusqlite::types::ValueRef::Text(t) => {
                                        PeelValue::String(String::from_utf8_lossy(t).into_owned())
                                    }
                                    rusqlite::types::ValueRef::Blob(b) => {
                                        PeelValue::String(format!("<blob {} bytes>", b.len()))
                                    }
                                };
                                map.insert(name.clone(), val);
                            }
                            Ok(PeelValue::Object {
                                struct_name: None,
                                fields: Arc::new(Mutex::new(map)),
                            })
                        })
                        .map_err(|e| anyhow!(e))?;

                    let mut results = vec![];
                    for row in rows {
                        results.push(row.map_err(|e| anyhow!(e))?);
                    }

                    Ok(PeelValue::List(Arc::new(Mutex::new(results))))
                })
            }),
        })),
    );

    let conn_close = conn.clone();
    fields.insert(
        "close".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "close".to_string(),
            handler: Arc::new(move |_| {
                let conn = conn_close.clone();
                Box::pin(async move {
                    let mut conn_lock = conn.lock().unwrap();
                    *conn_lock = None;
                    Ok(PeelValue::Void)
                })
            }),
        })),
    );

    PeelValue::Object {
        struct_name: Some("Connection".to_string()),
        fields: Arc::new(Mutex::new(fields)),
    }
}

fn peel_to_rusqlite_params(val: PeelValue) -> anyhow::Result<Vec<rusqlite::types::Value>> {
    match val {
        PeelValue::List(l) => {
            let params: Vec<rusqlite::types::Value> = l
                .lock()
                .unwrap()
                .iter()
                .map(|item| match item {
                    PeelValue::Int(i) => rusqlite::types::Value::Integer(*i),
                    PeelValue::Float(f) => rusqlite::types::Value::Real(*f),
                    PeelValue::String(s) => rusqlite::types::Value::Text(s.clone()),
                    PeelValue::Bool(b) => rusqlite::types::Value::Integer(if *b { 1 } else { 0 }),
                    PeelValue::Void => rusqlite::types::Value::Null,
                    _ => rusqlite::types::Value::Null,
                })
                .collect();
            Ok(params)
        }
        _ => Err(anyhow!("Params must be a list")),
    }
}
