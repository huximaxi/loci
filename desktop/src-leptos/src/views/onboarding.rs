use crate::models::InferenceStatus;
use crate::tauri_bindings::invoke_unit;
use leptos::*;
use leptos_router::A;

#[component]
pub fn Onboarding() -> impl IntoView {
    // Non-blocking inference probe — shows a quiet notice if neither local nor
    // Claude is available. Never gates navigation; purely informative.
    let inference: Resource<(), Option<InferenceStatus>> = create_resource(
        || (),
        |_| async move {
            let r: Result<InferenceStatus, String> =
                invoke_unit("check_inference_available").await;
            r.ok()
        },
    );

    view! {
        <main class="onboarding">
            <header class="onboarding-header">
                <h1>"loci wizard"</h1>
                <p class="lede">"A palace is a place you remember from."</p>
            </header>

            <nav class="card-grid" aria-label="palace choice">
                <A href="/load" attr:class="card card-primary">
                    <span class="card-eyebrow">"open"</span>
                    <span class="card-title">"a palace I keep"</span>
                    <span class="card-body">
                        "Point at a directory with a "
                        <code>"PALACE.md"</code>
                        " (or "<code>"CLAUDE.md"</code>") and a "
                        <code>"_palace/"</code>
                        " inside."
                    </span>
                </A>

                <A href="/create" attr:class="card">
                    <span class="card-eyebrow">"start"</span>
                    <span class="card-title">"a palace I do not yet keep"</span>
                    <span class="card-body">
                        "Choose a parent directory. A fresh "
                        <code>"_palace/"</code>
                        " will take root there."
                    </span>
                </A>

                <A href="/about" attr:class="card card-quiet">
                    <span class="card-eyebrow">"read"</span>
                    <span class="card-title">"what is a palace?"</span>
                    <span class="card-body">"One short page, no decisions asked."</span>
                </A>
            </nav>

            <footer class="onboarding-footer">
                // Model status — only shows when probe has resolved to "no model".
                {move || {
                    match inference.get() {
                        Some(Some(s)) if !s.has_local && !s.has_claude => view! {
                            <span class="onboarding-model-notice">
                                "no model detected · "
                                <A href="/settings">"settings"</A>
                            </span>
                        }.into_view(),
                        _ => view! { <span></span> }.into_view(),
                    }
                }}
                <em>"Not yet is not forever."</em>
                <A href="/settings" attr:class="onboarding-settings-link">"settings"</A>
            </footer>
        </main>
    }
}
