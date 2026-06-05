use crate::dialog::open_directory;
use crate::models::{ActivePalace, InferenceStatus, PalaceManifest};
use crate::tauri_bindings::{invoke, invoke_unit};
use crate::views::greeter::mark_palace_greeted;
use crate::views::naming_ceremony::{CeremonyAnswers, NamingCeremony};
use leptos::*;
use leptos_router::{use_navigate, A};
use serde::Serialize;

#[derive(Serialize)]
struct PathArg<'a> {
    path: &'a str,
}

/// New-palace creation flow.
///
/// Flow: probe inference → [gate if none] → pick dir → NamingCeremony
///       → scaffold_palace_from_ceremony → dashboard
///
/// If the user has no model and chooses "start without AI", the ceremony is
/// skipped. A minimal palace scaffolds with defaults and `onboarding_complete:
/// false`, gating inference features until they complete onboarding later.
#[component]
pub fn SetupCreate() -> impl IntoView {
    let active = expect_context::<RwSignal<ActivePalace>>();
    let (status, set_status) = create_signal::<CreateStatus>(CreateStatus::CheckingModel);
    let navigate = use_navigate();

    // Step 2: ceremony complete OR no-AI scaffold → load → navigate.
    // Defined first so pick_no_ai can capture a clone.
    let on_ceremony_complete: Callback<CeremonyAnswers> = Callback::new({
        let navigate = navigate.clone();
        move |answers: CeremonyAnswers| {
            let parent_path = answers.parent_path.clone();
            set_status.set(CreateStatus::Building(parent_path.clone()));
            let navigate = navigate.clone();
            spawn_local(async move {
                let palace_path: String =
                    match invoke("scaffold_palace_from_ceremony", &answers).await {
                        Ok(p) => p,
                        Err(e) => {
                            set_status.set(CreateStatus::Error(format!(
                                "palace build failed: {e}"
                            )));
                            return;
                        }
                    };
                let manifest: PalaceManifest =
                    match invoke("load_palace", &PathArg { path: &palace_path }).await {
                        Ok(m) => m,
                        Err(e) => {
                            set_status.set(CreateStatus::Error(format!(
                                "load after ceremony: {e}"
                            )));
                            return;
                        }
                    };
                active.set(ActivePalace {
                    path: Some(palace_path.clone()),
                    manifest: Some(manifest),
                });
                mark_palace_greeted(&palace_path);
                navigate("/dashboard", Default::default());
            });
        }
    });

    // Step 1a: pick directory + show ceremony (normal AI path).
    let pick_directory: Callback<()> = Callback::new(move |_: ()| {
        spawn_local(async move {
            set_status.set(CreateStatus::Picking);
            let parent = match open_directory("Choose where your palace will live").await {
                Ok(Some(p)) => p,
                Ok(None) => {
                    set_status.set(CreateStatus::Idle);
                    return;
                }
                Err(e) => {
                    set_status.set(CreateStatus::Error(e));
                    return;
                }
            };
            set_status.set(CreateStatus::Ceremony(parent));
        });
    });

    // Step 1b: pick directory + skip ceremony (no-AI path).
    // Scaffolds a minimal palace with sentinel defaults; onboarding_complete = false.
    let pick_no_ai: Callback<()> = Callback::new({
        let on_cc = on_ceremony_complete.clone();
        move |_: ()| {
            let on_cc = on_cc.clone();
            spawn_local(async move {
                set_status.set(CreateStatus::Picking);
                let parent = match open_directory("Choose where your palace will live").await {
                    Ok(Some(p)) => p,
                    Ok(None) => {
                        // User cancelled: return to gate
                        set_status.set(CreateStatus::NoModel);
                        return;
                    }
                    Err(e) => {
                        set_status.set(CreateStatus::Error(e));
                        return;
                    }
                };
                // Bypass ceremony entirely.
                on_cc.call(CeremonyAnswers {
                    parent_path: parent,
                    holder_name: "unknown".to_string(),
                    present_answer: String::new(),
                    present_answer_refined: None,
                    garden_seed: String::new(),
                    greeter_name: "unnamed".to_string(),
                    onboarding_complete: false,
                });
            });
        }
    });

    // On mount: probe inference. Show gate if neither local nor Claude is available.
    create_effect(move |_| {
        spawn_local(async move {
            let result: Result<InferenceStatus, String> =
                invoke_unit("check_inference_available").await;
            match result {
                Ok(s) if s.has_local || s.has_claude => set_status.set(CreateStatus::Idle),
                _ => set_status.set(CreateStatus::NoModel),
            }
        });
    });

    view! {
        <main class="wizard">
            <header class="wizard-header">
                <A href="/" attr:class="back">"< back"</A>
                <h2>"Start a fresh palace"</h2>
            </header>

            <section class="wizard-body">
                {move || match status.get() {
                    CreateStatus::CheckingModel => view! {
                        <p class="muted">"Checking for a mind to think with…"</p>
                    }.into_view(),

                    CreateStatus::NoModel => view! {
                        <div class="model-gate">
                            <p class="model-gate-lead">
                                "The naming ceremony is a conversation. "
                                "It needs a local model (Ollama) or Claude Code."
                            </p>
                            <p class="muted">
                                "Neither was found. Set one up in settings, then come back."
                            </p>
                            <div class="model-gate-actions">
                                <A href="/settings" attr:class="button primary">
                                    "set up a model"
                                </A>
                                <button class="secondary" on:click=move |_| pick_no_ai.call(())>
                                    "start without AI"
                                </button>
                            </div>
                            <p class="model-gate-note muted">
                                "Skipping builds a minimal palace now. "
                                "The ceremony runs on next open once a model is ready."
                            </p>
                        </div>
                    }.into_view(),

                    CreateStatus::Idle => view! {
                        <div>
                            <p>
                                "Choose a parent directory. A "<code>"_palace/"</code>
                                " will grow inside it, shaped by a short conversation."
                            </p>
                            <p class="muted">"Nothing existing is touched."</p>
                            <button class="primary" on:click=move |_| pick_directory.call(())>
                                "Choose parent directory"
                            </button>
                        </div>
                    }.into_view(),

                    CreateStatus::Picking => view! {
                        <p class="muted">"Waiting for your choice…"</p>
                    }.into_view(),

                    CreateStatus::Ceremony(parent) => view! {
                        <NamingCeremony
                            parent_path=parent
                            on_complete=on_ceremony_complete
                        />
                    }.into_view(),

                    CreateStatus::Building(p) => view! {
                        <p class="muted">"Building palace at "<code>{p}</code>"…"</p>
                    }.into_view(),

                    CreateStatus::Error(e) => view! {
                        <div>
                            <p class="warn">"Something is off."</p>
                            <p class="muted">{e}</p>
                            <button class="primary" on:click=move |_| pick_directory.call(())>
                                "Try again"
                            </button>
                        </div>
                    }.into_view(),
                }}
            </section>
        </main>
    }
}

#[derive(Clone, Debug)]
enum CreateStatus {
    CheckingModel,
    Idle,
    NoModel,
    Picking,
    Ceremony(String),
    Building(String),
    Error(String),
}
