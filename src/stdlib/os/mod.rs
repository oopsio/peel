use crate::runtime::value::{NativeFunc, PeelValue};
use anyhow::anyhow;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_double, c_longlong};
use std::sync::Arc;

unsafe extern "C" {
    fn peel_os_get_env(name: *const c_char) -> *mut c_char;
    fn peel_os_set_env(name: *const c_char, value: *const c_char) -> c_int;
    fn peel_os_cwd() -> *mut c_char;
    fn peel_os_platform() -> *mut c_char;
    fn peel_os_arch() -> *mut c_char;
    fn peel_os_uptime() -> c_longlong;
    fn peel_os_hostname() -> *mut c_char;
    fn peel_os_cpu_usage() -> c_double;
    fn peel_os_total_memory() -> u64;
    fn peel_os_free_memory() -> u64;
    fn peel_os_free(ptr: *mut std::ffi::c_void);
}

fn c_to_rust_string(ptr: *mut c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    let c_str = unsafe { CStr::from_ptr(ptr) };
    let r_str = c_str.to_string_lossy().into_owned();
    unsafe { peel_os_free(ptr as *mut std::ffi::c_void) };
    Some(r_str)
}

pub fn register() -> HashMap<String, PeelValue> {
    let mut methods = HashMap::new();

    methods.insert(
        "getenv".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "getenv".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let Some(PeelValue::String(name)) = args.get(0) {
                        let c_name = CString::new(name.as_str()).unwrap();
                        let res_ptr = unsafe { peel_os_get_env(c_name.as_ptr()) };
                        if let Some(res) = c_to_rust_string(res_ptr) {
                            Ok(PeelValue::String(res))
                        } else {
                            Ok(PeelValue::Void)
                        }
                    } else {
                        Err(anyhow!("os.getenv expects a string"))
                    }
                })
            }),
        })),
    );

    methods.insert(
        "setenv".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "setenv".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let (Some(PeelValue::String(name)), Some(PeelValue::String(val))) = (args.get(0), args.get(1)) {
                        let c_name = CString::new(name.as_str()).unwrap();
                        let c_val = CString::new(val.as_str()).unwrap();
                        let res = unsafe { peel_os_set_env(c_name.as_ptr(), c_val.as_ptr()) };
                        Ok(PeelValue::Bool(res == 0))
                    } else {
                        Err(anyhow!("os.setenv expects name and value as strings"))
                    }
                })
            }),
        })),
    );

    methods.insert(
        "cwd".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "cwd".to_string(),
            handler: Arc::new(|_| {
                Box::pin(async move {
                    let res_ptr = unsafe { peel_os_cwd() };
                    if let Some(res) = c_to_rust_string(res_ptr) {
                        Ok(PeelValue::String(res))
                    } else {
                        Err(anyhow!("Failed to get current directory"))
                    }
                })
            }),
        })),
    );

    methods.insert(
        "platform".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "platform".to_string(),
            handler: Arc::new(|_| {
                Box::pin(async move {
                    let res_ptr = unsafe { peel_os_platform() };
                    if let Some(res) = c_to_rust_string(res_ptr) {
                        Ok(PeelValue::String(res))
                    } else {
                        Err(anyhow!("Failed to get platform"))
                    }
                })
            }),
        })),
    );

    methods.insert(
        "arch".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "arch".to_string(),
            handler: Arc::new(|_| {
                Box::pin(async move {
                    let res_ptr = unsafe { peel_os_arch() };
                    if let Some(res) = c_to_rust_string(res_ptr) {
                        Ok(PeelValue::String(res))
                    } else {
                        Err(anyhow!("Failed to get architecture"))
                    }
                })
            }),
        })),
    );

    methods.insert(
        "uptime".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "uptime".to_string(),
            handler: Arc::new(|_| {
                Box::pin(async move {
                    let res = unsafe { peel_os_uptime() };
                    Ok(PeelValue::Int(res as i64))
                })
            }),
        })),
    );

    methods.insert(
        "hostname".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "hostname".to_string(),
            handler: Arc::new(|_| {
                Box::pin(async move {
                    let res_ptr = unsafe { peel_os_hostname() };
                    if let Some(res) = c_to_rust_string(res_ptr) {
                        Ok(PeelValue::String(res))
                    } else {
                        Err(anyhow!("Failed to get hostname"))
                    }
                })
            }),
        })),
    );

    methods.insert(
        "cpu_usage".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "cpu_usage".to_string(),
            handler: Arc::new(|_| {
                Box::pin(async move {
                    let res = unsafe { peel_os_cpu_usage() };
                    Ok(PeelValue::Float(res))
                })
            }),
        })),
    );

    methods.insert(
        "total_memory".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "total_memory".to_string(),
            handler: Arc::new(|_| {
                Box::pin(async move {
                    let res = unsafe { peel_os_total_memory() };
                    Ok(PeelValue::Int(res as i64))
                })
            }),
        })),
    );

    methods.insert(
        "free_memory".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "free_memory".to_string(),
            handler: Arc::new(|_| {
                Box::pin(async move {
                    let res = unsafe { peel_os_free_memory() };
                    Ok(PeelValue::Int(res as i64))
                })
            }),
        })),
    );

    methods
}
