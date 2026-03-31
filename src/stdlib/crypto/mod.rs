use crate::runtime::value::{NativeFunc, PeelValue};
use anyhow::anyhow;
use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Arc;

unsafe extern "C" {
    fn peel_sha256(data: *const c_char) -> *mut c_char;
    fn peel_sha512(data: *const c_char) -> *mut c_char;
    fn peel_md5(data: *const c_char) -> *mut c_char;
    fn peel_hmac_sha256(data: *const c_char, key: *const c_char) -> *mut c_char;
    fn peel_aes_256_cbc_encrypt(data: *const c_char, key: *const c_char, iv: *const c_char) -> *mut c_char;
    fn peel_aes_256_cbc_decrypt(ciphertext: *const c_char, key: *const c_char, iv: *const c_char) -> *mut c_char;
    fn peel_crypto_free(ptr: *mut c_char);
}

fn c_to_rust_string(ptr: *mut c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    let c_str = unsafe { CStr::from_ptr(ptr) };
    let r_str = c_str.to_string_lossy().into_owned();
    unsafe { peel_crypto_free(ptr) };
    Some(r_str)
}

pub fn register() -> HashMap<String, PeelValue> {
    let mut methods = HashMap::new();

    // SHA256
    methods.insert(
        "sha256".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "sha256".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let Some(PeelValue::String(data)) = args.get(0) {
                        let c_data = CString::new(data.as_str()).unwrap();
                        let res_ptr = unsafe { peel_sha256(c_data.as_ptr()) };
                        if let Some(res) = c_to_rust_string(res_ptr) {
                            Ok(PeelValue::String(res))
                        } else {
                            Err(anyhow!("SHA256 failed"))
                        }
                    } else {
                        Err(anyhow!("crypto.sha256 expects a string"))
                    }
                })
            }),
        })),
    );

    // SHA512
    methods.insert(
        "sha512".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "sha512".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let Some(PeelValue::String(data)) = args.get(0) {
                        let c_data = CString::new(data.as_str()).unwrap();
                        let res_ptr = unsafe { peel_sha512(c_data.as_ptr()) };
                        if let Some(res) = c_to_rust_string(res_ptr) {
                            Ok(PeelValue::String(res))
                        } else {
                            Err(anyhow!("SHA512 failed"))
                        }
                    } else {
                        Err(anyhow!("crypto.sha512 expects a string"))
                    }
                })
            }),
        })),
    );

    // MD5
    methods.insert(
        "md5".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "md5".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let Some(PeelValue::String(data)) = args.get(0) {
                        let c_data = CString::new(data.as_str()).unwrap();
                        let res_ptr = unsafe { peel_md5(c_data.as_ptr()) };
                        if let Some(res) = c_to_rust_string(res_ptr) {
                            Ok(PeelValue::String(res))
                        } else {
                            Err(anyhow!("MD5 failed"))
                        }
                    } else {
                        Err(anyhow!("crypto.md5 expects a string"))
                    }
                })
            }),
        })),
    );

    // HMAC SHA256
    methods.insert(
        "hmac".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "hmac".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let (Some(PeelValue::String(data)), Some(PeelValue::String(key))) = (args.get(0), args.get(1)) {
                        let c_data = CString::new(data.as_str()).unwrap();
                        let c_key = CString::new(key.as_str()).unwrap();
                        let res_ptr = unsafe { peel_hmac_sha256(c_data.as_ptr(), c_key.as_ptr()) };
                        if let Some(res) = c_to_rust_string(res_ptr) {
                            Ok(PeelValue::String(res))
                        } else {
                            Err(anyhow!("HMAC failed"))
                        }
                    } else {
                        Err(anyhow!("crypto.hmac expects data and key strings"))
                    }
                })
            }),
        })),
    );

    // AES Encrypt (CBC)
    methods.insert(
        "aes_encrypt".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "aes_encrypt".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let (Some(PeelValue::String(data)), Some(PeelValue::String(key)), Some(PeelValue::String(iv))) = (args.get(0), args.get(1), args.get(2)) {
                        if key.len() != 32 { return Err(anyhow!("AES-256 requires 32-byte key")); }
                        if iv.len() != 16 { return Err(anyhow!("AES requires 16-byte IV")); }
                        
                        let c_data = CString::new(data.as_str()).unwrap();
                        let c_key = CString::new(key.as_str()).unwrap();
                        let c_iv = CString::new(iv.as_str()).unwrap();
                        let res_ptr = unsafe { peel_aes_256_cbc_encrypt(c_data.as_ptr(), c_key.as_ptr(), c_iv.as_ptr()) };
                        if let Some(res) = c_to_rust_string(res_ptr) {
                            Ok(PeelValue::String(res))
                        } else {
                            Err(anyhow!("AES encryption failed"))
                        }
                    } else {
                        Err(anyhow!("crypto.aes_encrypt expects data, key (32 bytes), and iv (16 bytes)"))
                    }
                })
            }),
        })),
    );

    // AES Decrypt (CBC)
    methods.insert(
        "aes_decrypt".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "aes_decrypt".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let (Some(PeelValue::String(ciphertext)), Some(PeelValue::String(key)), Some(PeelValue::String(iv))) = (args.get(0), args.get(1), args.get(2)) {
                        if key.len() != 32 { return Err(anyhow!("AES-256 requires 32-byte key")); }
                        if iv.len() != 16 { return Err(anyhow!("AES requires 16-byte IV")); }

                        let c_data = CString::new(ciphertext.as_str()).unwrap();
                        let c_key = CString::new(key.as_str()).unwrap();
                        let c_iv = CString::new(iv.as_str()).unwrap();
                        let res_ptr = unsafe { peel_aes_256_cbc_decrypt(c_data.as_ptr(), c_key.as_ptr(), c_iv.as_ptr()) };
                        if let Some(res) = c_to_rust_string(res_ptr) {
                            Ok(PeelValue::String(res))
                        } else {
                            Err(anyhow!("AES decryption failed"))
                        }
                    } else {
                        Err(anyhow!("crypto.aes_decrypt expects ciphertext, key (32 bytes), and iv (16 bytes)"))
                    }
                })
            }),
        })),
    );

    methods
}
