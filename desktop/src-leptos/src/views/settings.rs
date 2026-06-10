use crate::models::{ActivePalace, DeltaEntry, InferenceStatus, UpdateReport};
use crate::tauri_bindings::{invoke, invoke_unit};
use leptos::*;
use leptos_router::A;
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CheckArgs<'a> {
    palace_path: &'a str,
    include_candidates: bool,
}

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

    // ── palace-update: the delta checker ──────────────────────────────────────
    // Explicit-trigger only: a create_action (fires on dispatch/click), never a
    // Resource (which fires on mount = the auto-check we deliberately avoid).
    let active = expect_context::<RwSignal<ActivePalace>>();
    let (show_candidates, set_show_candidates) = create_signal(false);
    let check = create_action(move |args: &(String, bool)| {
        let (path, include_candidates) = args.clone();
        async move {
            invoke::<_, UpdateReport>(
                "check_for_updates",
                &CheckArgs { palace_path: &path, include_candidates },
            )
            .await
            .ok()
        }
    });
    let on_check = move |_| {
        let path = active.get().path.unwrap_or_default();
        check.dispatch((path, show_candidates.get()));
    };

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

                // ── updates: what's yours to let in ───────────────────────────
                <p class="settings-section-label">"updates"</p>
                <div class="settings-card">
                    <div class="settings-card-row">
                        <div class="settings-card-left">
                            <span class="settings-card-name">"what\u{2019}s yours to let in"</span>
                            <span class="settings-card-meta">
                                "compare the loci methodology · read-only, nothing applied"
                            </span>
                        </div>
                        <button on:click=on_check>"check"</button>
                    </div>
                    <label class="settings-toggle">
                        <input
                            type="checkbox"
                            prop:checked=show_candidates
                            on:change=move |ev| set_show_candidates.set(event_target_checked(&ev))
                        />
                        " show candidates (opt-in)"
                    </label>
                    <div class="update-output">
                        {move || {
                            if check.pending().get() {
                                view! {
                                    <span class="brain-chip brain-checking">"checking…"</span>
                                }
                                .into_view()
                            } else {
                                match check.value().get() {
                                    None => view! { <span></span> }.into_view(),
                                    Some(None) => view! {
                                        <p class="settings-hint warn">"Could not check just now."</p>
                                    }
                                    .into_view(),
                                    Some(Some(report)) => render_report(report),
                                }
                            }
                        }}
                    </div>
                </div>
            </section>
        </main>
    }
}

/// Render the update report as the two-half recognition diff: your version on
/// one side, only the newer entries lit on the other. Read-only; no apply path.
fn render_report(r: UpdateReport) -> View {
    let local = r
        .local_version
        .clone()
        .unwrap_or_else(|| "an unknown version".to_string());
    match r.status.as_str() {
        "behind" => {
            let n = r.entries.len();
            let head = format!(
                "You are on {}. {} thing{} grew since.",
                local,
                n,
                if n == 1 { "" } else { "s" }
            );
            view! {
                <div class="update-result">
                    <p class="update-head">{head}</p>
                    <ul class="update-entries">{render_entries(&r.entries)}</ul>
                </div>
            }
            .into_view()
        }
        "current" => view! {
            <p class="settings-hint">
                "You are current on " <strong>{r.latest_version.clone()}</strong>
                ". Nothing to let in."
            </p>
        }
        .into_view(),
        "unknown" => view! {
            <div class="update-result">
                <p class="settings-hint">
                    "Could not read your palace version. Here is what the latest carries. "
                    "Run a full palace-update to compare."
                </p>
                <ul class="update-entries">{render_entries(&r.entries)}</ul>
            </div>
        }
        .into_view(),
        // "unavailable" and any unexpected status: fail-closed, show local version.
        _ => view! {
            <p class="settings-hint warn">
                "Could not reach the methodology source. You are on "
                <strong>{local}</strong> "."
            </p>
        }
        .into_view(),
    }
}

fn render_entries(entries: &[DeltaEntry]) -> View {
    entries
        .iter()
        .map(|e| {
            let items = e
                .items
                .iter()
                .map(|it| {
                    let summary = if it.summary.is_empty() {
                        String::new()
                    } else {
                        format!(": {}", it.summary)
                    };
                    view! {
                        <li class="update-item"><strong>{it.title.clone()}</strong>{summary}</li>
                    }
                })
                .collect_view();
            let cand = e
                .is_candidate
                .then(|| view! { <span class="update-badge">"candidate"</span> });
            view! {
                <li class="update-entry">
                    <div class="update-entry-head">
                        <span class="update-version">{format!("v{}", e.version)}</span>
                        <span class="update-date">{e.date.clone()}</span>
                        {cand}
                    </div>
                    <ul class="update-items">{items}</ul>
                </li>
            }
        })
        .collect_view()
}
