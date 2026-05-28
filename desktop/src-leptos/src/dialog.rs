// Directory picker for the Leptos/WASM frontend.
//
// The native panel is opened by a Rust command (`pick_palace_dir` in main.rs),
// not from JS. A pure-WASM build has no `window.__TAURI__.dialog` sugar (reaching
// for it crashes), and the raw `plugin:dialog|open` invoke deadlocks because the
// panel never reaches the main thread. Invoking our own Rust command rides the
// same transport the dashboard reads already use, and the plugin's Rust API
// dispatches the panel correctly.

use crate::tauri_bindings::invoke;
use serde::Serialize;

#[derive(Serialize)]
struct PickArgs<'a> {
    title: &'a str,
}

/// Open a native directory picker. Returns `Ok(None)` if the user cancels.
pub async fn open_directory(title: &str) -> Result<Option<String>, String> {
    invoke::<_, Option<String>>("pick_palace_dir", &PickArgs { title }).await
}
