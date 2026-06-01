use leptos::*;
use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;

// Phrase pool — random rotation, no immediate repeats.
pub const PHRASES: &[&str] = &[
    "a palace is a place you remember from",
    "you already know this",
    "intelligence, disintermediated",
    "rooms within rooms within rooms",
    "the crystal holds",
    "the room is waiting",
    "the palace was always yours",
    "your thoughts already have a location",
    "you know where everything is",
    "what you remember, you keep",
    "knowledge lives where you live",
    "the trace is local, so is the truth",
];
pub const PHRASE_INTERVAL_MS: u32 = 1400;
pub const FIRST_RUN_TOTAL_MS: u32 = 4500;

// ── localStorage helpers ──────────────────────────────────────────────────────

pub fn loci_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

pub fn has_seen_greeter() -> bool {
    loci_storage()
        .and_then(|s| s.get_item("loci_greeter_shown").ok().flatten())
        .is_some()
}

pub fn mark_greeter_seen() {
    if let Some(s) = loci_storage() {
        let _ = s.set_item("loci_greeter_shown", "1");
    }
}

pub fn has_greeted_palace(path: &str) -> bool {
    loci_storage()
        .and_then(|s| s.get_item("loci_greeted_palaces").ok().flatten())
        .and_then(|json| serde_json::from_str::<Vec<String>>(&json).ok())
        .map(|paths| paths.iter().any(|p| p == path))
        .unwrap_or(false)
}

pub fn mark_palace_greeted(path: &str) {
    let Some(s) = loci_storage() else { return };
    let mut paths: Vec<String> = s
        .get_item("loci_greeted_palaces")
        .ok()
        .flatten()
        .and_then(|j| serde_json::from_str(&j).ok())
        .unwrap_or_default();
    if !paths.iter().any(|p| p == path) {
        paths.push(path.to_string());
        if let Ok(json) = serde_json::to_string(&paths) {
            let _ = s.set_item("loci_greeted_palaces", &json);
        }
    }
}

// Clears all greeter state — for dev probing from the About page.
pub fn reset_greeter() {
    if let Some(s) = loci_storage() {
        let _ = s.remove_item("loci_greeter_shown");
        let _ = s.remove_item("loci_greeted_palaces");
    }
}

// ── Async helpers ─────────────────────────────────────────────────────────────

pub async fn sleep_ms(ms: u32) {
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        web_sys::window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ms as i32)
            .unwrap();
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

// Defers until the next compositor frame — ensures GPU layers are created
// before animations begin. Fixes WKWebView intermittent animation drop.
pub async fn next_animation_frame() {
    let promise = js_sys::Promise::new(&mut |resolve, _| {
        web_sys::window()
            .unwrap()
            .request_animation_frame(&resolve)
            .unwrap();
    });
    let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
}

pub fn random_other(current: usize, len: usize) -> usize {
    if len <= 1 { return 0; }
    let r = (js_sys::Math::random() * (len - 1) as f64) as usize;
    if r >= current { r + 1 } else { r }
}

// ── Greeter component ─────────────────────────────────────────────────────────

#[component]
pub fn Greeter(show: WriteSignal<bool>) -> impl IntoView {
    let (phrase_idx, set_phrase_idx) = create_signal(0usize);
    let (phrase_fading, set_phrase_fading) = create_signal(false);

    // Animation gating: add class only after the first compositor frame.
    // Prevents WKWebView from dropping the animation on first paint.
    let (anim_ready, set_anim_ready) = create_signal(false);
    create_effect(move |_| {
        spawn_local(async move {
            next_animation_frame().await;
            set_anim_ready.set(true);
        });
    });

    // Rc<Cell> guard: outlives signal disposal when the component unmounts.
    let dismissed = Rc::new(Cell::new(false));

    // Phrase cycling: random, no immediate repeats.
    {
        let dismissed = dismissed.clone();
        spawn_local(async move {
            loop {
                sleep_ms(PHRASE_INTERVAL_MS).await;
                if dismissed.get() { return; }
                set_phrase_fading.set(true);
                sleep_ms(350).await;
                if dismissed.get() { return; }
                let next = random_other(phrase_idx.get_untracked(), PHRASES.len());
                set_phrase_idx.set(next);
                set_phrase_fading.set(false);
            }
        });
    }

    // Auto-dismiss.
    {
        let dismissed = dismissed.clone();
        spawn_local(async move {
            sleep_ms(FIRST_RUN_TOTAL_MS).await;
            if !dismissed.get() {
                dismissed.set(true);
                mark_greeter_seen();
                show.set(false);
            }
        });
    }

    let on_skip = {
        let dismissed = dismissed.clone();
        move |_| {
            if !dismissed.get() {
                dismissed.set(true);
                mark_greeter_seen();
                show.set(false);
            }
        }
    };

    let anim_cls = move |base: &'static str| move || {
        if anim_ready.get() {
            format!("{base} greeter-anim")
        } else {
            base.to_string()
        }
    };

    view! {
        <div class="greeter-overlay" role="dialog" aria-label="loci wizard entrance">
            <button class="greeter-skip" on:click=on_skip>"skip"</button>

            <div class="greeter-orb-wrap" aria-hidden="true">
                <div class=anim_cls("greeter-orb-ambient") />
                <div class=anim_cls("greeter-orb") />
                <div class=anim_cls("greeter-orb-halo") />
            </div>

            <p class=move || if phrase_fading.get() {
                "greeter-phrase greeter-phrase-fading"
            } else {
                "greeter-phrase"
            }>
                {move || PHRASES[phrase_idx.get()]}
            </p>

            <p class="greeter-notice">
                "ambient visual pulses · safe at any display · no health claims made"
            </p>
        </div>
    }
}
