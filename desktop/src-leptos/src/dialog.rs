// Tauri v2 dialog wrapper. Accesses window.__TAURI__.dialog.open(options).
//
// Options shape (Tauri v2 docs):
//   { directory: bool, multiple: bool, title: string }
//
// Returns: selected path as String when single, null/undefined when cancelled,
//          array when multiple (not used here).

use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = ["__TAURI__", "dialog"], js_name = open)]
    async fn js_dialog_open(options: JsValue) -> Result<JsValue, JsValue>;
}

#[derive(Serialize)]
struct OpenDialogOptions<'a> {
    directory: bool,
    multiple: bool,
    title: &'a str,
}

/// Open a native directory picker. Returns `Ok(None)` if the user cancels.
pub async fn open_directory(title: &str) -> Result<Option<String>, String> {
    let opts = OpenDialogOptions {
        directory: true,
        multiple: false,
        title,
    };
    let opts_js = serde_wasm_bindgen::to_value(&opts)
        .map_err(|e| format!("serialise dialog opts: {e}"))?;
    let result = js_dialog_open(opts_js)
        .await
        .map_err(|e| format!("dialog open failed: {:?}", e))?;
    if result.is_null() || result.is_undefined() {
        return Ok(None);
    }
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("deserialise dialog result: {e}"))
        .map(Some)
}
