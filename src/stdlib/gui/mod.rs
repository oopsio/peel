use crate::runtime::value::{NativeFunc, PeelValue};
use anyhow::anyhow;
use std::collections::HashMap;
use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_float};
use std::sync::Arc;

unsafe extern "C" {
    fn peel_gui_init(width: c_int, height: c_int, title: *const c_char);
    fn peel_gui_shutdown();
    fn peel_gui_should_close() -> c_int;
    fn peel_gui_poll_events();
    fn peel_gui_render();
    fn peel_gui_window_begin(title: *const c_char, x: c_int, y: c_int, w: c_int, h: c_int, flags: c_int) -> c_int;
    fn peel_gui_window_end();
    fn peel_gui_layout_row_dynamic(height: c_float, cols: c_int);
    fn peel_gui_label(text: *const c_char, align: c_int);
    fn peel_gui_button(label: *const c_char) -> c_int;
}

pub fn register() -> HashMap<String, PeelValue> {
    let mut methods = HashMap::new();

    methods.insert(
        "init".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "init".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    let w = args.get(0).and_then(|v| v.as_int()).unwrap_or(800) as i32;
                    let h = args.get(1).and_then(|v| v.as_int()).unwrap_or(600) as i32;
                    let title = args.get(2).and_then(|v| v.as_string()).unwrap_or("Peel GUI".to_string());
                    let c_title = CString::new(title).unwrap();
                    unsafe { peel_gui_init(w, h, c_title.as_ptr()) };
                    Ok(PeelValue::Void)
                })
            }),
        })),
    );

    methods.insert(
        "shutdown".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "shutdown".to_string(),
            handler: Arc::new(|_| {
                Box::pin(async move {
                    unsafe { peel_gui_shutdown() };
                    Ok(PeelValue::Void)
                })
            }),
        })),
    );

    methods.insert(
        "should_close".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "should_close".to_string(),
            handler: Arc::new(|_| {
                Box::pin(async move {
                    let res = unsafe { peel_gui_should_close() };
                    Ok(PeelValue::Bool(res != 0))
                })
            }),
        })),
    );

    methods.insert(
        "poll".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "poll".to_string(),
            handler: Arc::new(|_| {
                Box::pin(async move {
                    unsafe { peel_gui_poll_events() };
                    Ok(PeelValue::Void)
                })
            }),
        })),
    );

    methods.insert(
        "render".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "render".to_string(),
            handler: Arc::new(|_| {
                Box::pin(async move {
                    unsafe { peel_gui_render() };
                    Ok(PeelValue::Void)
                })
            }),
        })),
    );

    methods.insert(
        "window_begin".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "window_begin".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    let title = args.get(0).and_then(|v| v.as_string()).unwrap_or("Window".to_string());
                    let x = args.get(1).and_then(|v| v.as_int()).unwrap_or(0) as i32;
                    let y = args.get(2).and_then(|v| v.as_int()).unwrap_or(0) as i32;
                    let w = args.get(3).and_then(|v| v.as_int()).unwrap_or(200) as i32;
                    let h = args.get(4).and_then(|v| v.as_int()).unwrap_or(200) as i32;
                    let c_title = CString::new(title).unwrap();
                    let res = unsafe { peel_gui_window_begin(c_title.as_ptr(), x, y, w, h, 0) };
                    Ok(PeelValue::Bool(res != 0))
                })
            }),
        })),
    );

    methods.insert(
        "window_end".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "window_end".to_string(),
            handler: Arc::new(|_| {
                Box::pin(async move {
                    unsafe { peel_gui_window_end() };
                    Ok(PeelValue::Void)
                })
            }),
        })),
    );

    methods.insert(
        "layout_row_dynamic".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "layout_row_dynamic".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    let h = args.get(0).and_then(|v| v.as_float()).unwrap_or(30.0) as f32;
                    let cols = args.get(1).and_then(|v| v.as_int()).unwrap_or(1) as i32;
                    unsafe { peel_gui_layout_row_dynamic(h, cols) };
                    Ok(PeelValue::Void)
                })
            }),
        })),
    );

    methods.insert(
        "label".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "label".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let Some(PeelValue::String(text)) = args.get(0) {
                        let c_text = CString::new(text.as_str()).unwrap();
                        unsafe { peel_gui_label(c_text.as_ptr(), 0) }; // 0 = NK_TEXT_LEFT
                        Ok(PeelValue::Void)
                    } else {
                        Err(anyhow!("gui.label expects a string"))
                    }
                })
            }),
        })),
    );

    methods.insert(
        "button".to_string(),
        PeelValue::NativeFunction(Arc::new(NativeFunc {
            name: "button".to_string(),
            handler: Arc::new(|args| {
                Box::pin(async move {
                    if let Some(PeelValue::String(label)) = args.get(0) {
                        let c_label = CString::new(label.as_str()).unwrap();
                        let res = unsafe { peel_gui_button(c_label.as_ptr()) };
                        Ok(PeelValue::Bool(res != 0))
                    } else {
                        Err(anyhow!("gui.button expects a string"))
                    }
                })
            }),
        })),
    );

    methods
}
