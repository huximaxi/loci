use crate::models::InferenceStatus;
use crate::tauri_bindings::invoke_unit;
use leptos::*;
use leptos_router::A;

#[component]
pub fn Settings() -> impl IntoView {
    // Live probe — refetches on demand via "check again".
    let inference: Resource<(), Option<InferenceStatus>> = create_resource(
        || (),
        |_| async move {
            let r: Result<InferenceStatus, String> =
                invoke_unit("check_inference_available").await;
            r.ok()
        },
    );

    let check_again = move |_| inference.refetch();

    view! {
        <main class="wizard">
            <header class="wizard-header">
                <A href="/" attr:class="back">"< back"</A>
                <h2>"Settings"</h2>
            </header>

            <section class="wizard-body settings-body">
                <p class="settings-section-label">"inference"</p>

                // ── Local garden (Ollama) ─────────────────────────────────────
                <div class="settings-card">
                    <div class="settings-card-row">
                        <div class="settings-card-left">
                            <span class="settings-card-name">"local garden"</span>
                            <span class="settings-card-meta">"Ollama · runs on this machine"</span>
                        </div>
                        {move || {
                            match inference.get() {
                                None => view! {
                                    <span class="brain-chip brain-checking">"checking…"</span>
                                }.into_view(),
                                Some(None) => view! {
                                    <span class="brain-chip brain-asleep">"error"</span>
                                }.into_view(),
                                Some(Some(s)) => {
                                    if s.has_local {
                                        let n = s.local_models.len();
                                        view! {
                                            <span class="brain-chip brain-awake">
                                                {format!("awake · {} model{}", n, if n == 1 { "" } else { "s" })}
                                            </span>
                                        }.into_view()
                                    } else if s.ollama_running {
                                        view! {
                                            <span class="brain-chip brain-asleep">"running · no models"</span>
                                        }.into_view()
                                    } else {
                                        view! {
                                            <span class="brain-chip brain-asleep">"asleep"</span>
                                        }.into_view()
                                    }
                                }
                            }
                        }}
                    </div>
                    {move || {
                        match inference.get() {
                            Some(Some(s)) if !s.local_models.is_empty() => view! {
                                <p class="settings-models">
                                    {s.local_models.join("  ·  ")}
                                </p>
                            }.into_view(),
                            Some(Some(s)) if s.ollama_running => view! {
                                <p class="settings-hint warn">
                                    "No models installed. Run "
                                    <code>"ollama pull llama3.2"</code>
                                    " in a terminal."
                                </p>
                            }.into_view(),
                            Some(Some(_)) => view! {
                                <p class="settings-hint">
                                    "Ollama is not running. "
                                    <a href="https://ollama.ai" target="_blank" rel="noopener">
                                        "Download Ollama"
                                    </a>
                                    " — then start it with "<code>"ollama serve"</code>"."
                                </p>
                            }.into_view(),
                            _ => view! { <span></span> }.into_view(),
                        }
                    }}
                </div>

                // ── Online garden (Claude CLI) ────────────────────────────────
                <div class="settings-card">
                    <div class="settings-card-row">
                        <div class="settings-card-left">
                            <span class="settings-card-name">"online garden"</span>
                            <span class="settings-card-meta">"Claude Code CLI · your Anthropic license"</span>
                        </div>
                        {move || {
                            match inference.get() {
                                None => view! {
                                    <span class="brain-chip brain-checking">"checking…"</span>
                                }.into_view(),
                                Some(None) => view! {
                                    <span class="brain-chip brain-asleep">"error"</span>
                                }.into_view(),
                                Some(Some(s)) => {
                                    if s.has_claude {
                                        view! {
                                            <span class="brain-chip brain-awake">"found"</span>
                                        }.into_view()
                                    } else {
                                        view! {
                                            <span class="brain-chip brain-asleep">"not found"</span>
                                        }.into_view()
                                    }
                                }
                            }
                        }}
                    </div>
                    {move || {
                        match inference.get() {
                            Some(Some(s)) if s.has_claude => view! {
                                <p class="settings-hint muted">
                                    "Requests go to Anthropic's API on your own license. "
                                    "Nothing leaves this machine without your explicit choice."
                                </p>
                            }.into_view(),
                            Some(Some(_)) => view! {
                                <p class="settings-hint">
                                    "Claude Code CLI not detected. "
                                    <a href="https://claude.ai/code" target="_blank" rel="noopener">
                                        "Install Claude Code"
                                    </a>
                                    " — then restart loci wizard."
                                </p>
                            }.into_view(),
                            _ => view! { <span></span> }.into_view(),
                        }
                    }}
                </div>

                <div class="settings-actions">
                    <button on:click=check_again>"check again"</button>
                </div>
            </section>
        </main>
    }
}
