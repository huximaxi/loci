// Tauri v2 invoke + listen wrappers for Leptos/WASM.
//
// Tauri commands and events are accessed via the global object that Tauri
// injects when `app.withGlobalTauri = true` in tauri.conf.json:
//
//   window.__TAURI__.core.invoke(cmd, args) -> Promise<T>
//   window.__TAURI__.event.listen(event, handler) -> Promise<UnlistenFn>
//
// The wrappers below give Rust-typed access via serde-wasm-bindgen.

use serde::{de::DeserializeOwned, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(catch, js_namespace = ["__TAURI__", "core"], js_name = invoke)]
    async fn js_invoke(cmd: &str, args: JsValue) -> Result<JsValue, JsValue>;

    #[wasm_bindgen(catch, js_namespace = ["__TAURI__", "event"], js_name = listen)]
    async fn js_listen(event: &str, handler: &js_sys::Function) -> Result<JsValue, JsValue>;
}

fn js_err(v: JsValue) -> String {
    v.as_string().unwrap_or_else(|| format!("{:?}", v))
}

/// Invoke a Tauri command with typed args and typed return.
///
/// `Args` should be a struct whose field names match the Rust command parameters
/// (camelCase or as-is per serde rules).
pub async fn invoke<Args: Serialize, Ret: DeserializeOwned>(
    cmd: &str,
    args: &Args,
) -> Result<Ret, String> {
    let args_js = serde_wasm_bindgen::to_value(args)
        .map_err(|e| format!("serialise args for {cmd}: {e}"))?;
    let result = js_invoke(cmd, args_js).await.map_err(js_err)?;
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("deserialise result of {cmd}: {e}"))
}

/// Invoke a Tauri command that takes no arguments.
pub async fn invoke_unit<Ret: DeserializeOwned>(cmd: &str) -> Result<Ret, String> {
    let result = js_invoke(cmd, JsValue::UNDEFINED).await.map_err(js_err)?;
    serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("deserialise result of {cmd}: {e}"))
}

/// Listen for a Tauri event. Returns a JsValue that is the unlisten function
/// (call it via `js_sys::Function::call0(&this, &JsValue::NULL)` to detach).
///
/// The handler receives a `JsValue` representing the Tauri event object:
///   { event: string, id: number, payload: any }
pub async fn listen(event: &str, handler: &Closure<dyn FnMut(JsValue)>) -> Result<JsValue, String> {
    let js_handler = handler.as_ref().unchecked_ref::<js_sys::Function>();
    js_listen(event, js_handler).await.map_err(js_err)
}
