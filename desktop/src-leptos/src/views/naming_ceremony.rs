use crate::tauri_bindings::invoke;
use crate::views::greeter::{
    next_animation_frame, random_other, sleep_ms, FIRST_RUN_TOTAL_MS, PHRASE_INTERVAL_MS, PHRASES,
};
use leptos::*;
use serde::{Deserialize, Serialize};
use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen_futures::spawn_local;

// ── Types ────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Deserialize)]
pub struct NameOption {
    pub name: String,
    pub note: String,
}

/// All answers collected during the ceremony. Passed to `on_complete`.
/// The parent (setup_create.rs) invokes `scaffold_palace_from_ceremony` with this.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CeremonyAnswers {
    pub parent_path: String,
    pub holder_name: String,
    pub present_answer: String,
    pub present_answer_refined: Option<String>,
    pub garden_seed: String,
    pub greeter_name: String,
    /// false when the user skipped — palace scaffolds but gates on inference/chat
    /// until the ceremony is completed on next open.
    pub onboarding_complete: bool,
}

// ── Phase state machine ───────────────────────────────────────────────────────

#[derive(Clone, PartialEq, Debug)]
enum Phase {
    Arrival,
    Moment1,
    Moment2a,
    CheckingVagueness,
    Moment2b,
    Moment3,
    GeneratingNames,
    Naming,
    Confirmed,
    Done,
}

// ── Tauri arg shapes ─────────────────────────────────────────────────────────

#[derive(Serialize)]
struct VaguenessArg {
    answer: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NamesArg {
    holder_name: String,
    present_answer: String,
    garden_seed: String,
}

// ── Component ─────────────────────────────────────────────────────────────────

/// The Naming Ceremony: a 3-question guided conversation that builds the palace.
/// Replaces the silent scaffold→greeter sequence for new palace creation.
///
/// Flow: Arrival (phrase cycling) → Moment1 (name) → Moment2a/2b (present)
///   → Moment3 (garden seed) → Naming (greeter name) → Confirmed → Done
///
/// `on_complete` is called with fully collected CeremonyAnswers. The parent
/// then calls scaffold_palace_from_ceremony and navigates to /dashboard.
#[component]
pub fn NamingCeremony(
    parent_path: String,
    on_complete: Callback<CeremonyAnswers>,
) -> impl IntoView {
    // ── Orb animation gating (same rAF gate as greeter.rs) ────────────────────
    let (anim_ready, set_anim_ready) = create_signal(false);
    create_effect(move |_| {
        spawn_local(async move {
            next_animation_frame().await;
            set_anim_ready.set(true);
        });
    });

    // ── Phase ─────────────────────────────────────────────────────────────────
    let (phase, set_phase) = create_signal(Phase::Arrival);

    // ── Phrase cycling ────────────────────────────────────────────────────────
    let (phrase_idx, set_phrase_idx) = create_signal(0usize);
    let (phrase_fading, set_phrase_fading) = create_signal(false);
    let phrase_cycle_done = Rc::new(Cell::new(false));

    // ── Answer accumulators ───────────────────────────────────────────────────
    let holder_name = create_rw_signal(String::new());
    let present_answer = create_rw_signal(String::new());
    let present_answer_refined = create_rw_signal::<Option<String>>(None);
    let garden_seed = create_rw_signal(String::new());
    let greeter_name = create_rw_signal(String::new());
    let name_options = create_rw_signal::<Vec<NameOption>>(vec![]);

    // ── Input field ───────────────────────────────────────────────────────────
    let (input_value, set_input_value) = create_signal(String::new());
    let input_ref = create_node_ref::<html::Input>();

    // Store parent_path so closures can capture it
    let parent_path_store = store_value(parent_path);

    // ── Arrival: phrase cycling, auto-advance after one full cycle ────────────
    {
        let phrase_cycle_done = phrase_cycle_done.clone();
        spawn_local(async move {
            sleep_ms(FIRST_RUN_TOTAL_MS).await;
            if phase.get_untracked() == Phase::Arrival {
                phrase_cycle_done.set(true);
                set_phase.set(Phase::Moment1);
            }
        });
    }
    {
        let phrase_cycle_done = phrase_cycle_done.clone();
        spawn_local(async move {
            loop {
                sleep_ms(PHRASE_INTERVAL_MS).await;
                if phrase_cycle_done.get() || phase.get_untracked() != Phase::Arrival {
                    return;
                }
                set_phrase_fading.set(true);
                sleep_ms(350).await;
                if phrase_cycle_done.get() { return; }
                let next = random_other(phrase_idx.get_untracked(), PHRASES.len());
                set_phrase_idx.set(next);
                set_phrase_fading.set(false);
            }
        });
    }

    // ── Focus input when phase changes to an input phase ──────────────────────
    create_effect(move |_| {
        let p = phase.get();
        if matches!(p, Phase::Moment1 | Phase::Moment2a | Phase::Moment2b | Phase::Moment3) {
            if let Some(el) = input_ref.get() {
                let _ = el.focus();
            }
        }
    });

    // ── Advance through phases ────────────────────────────────────────────────
    let do_advance = move || {
        let current = phase.get_untracked();
        let input = input_value.get_untracked();

        match current {
            Phase::Arrival => {
                // Skip phrase cycle, go straight to Moment1
                set_phase.set(Phase::Moment1);
            }
            Phase::Moment1 => {
                let name = if input.trim().is_empty() {
                    "unnamed".to_string()
                } else {
                    input.trim().to_string()
                };
                holder_name.set(name);
                set_input_value.set(String::new());
                set_phase.set(Phase::Moment2a);
            }
            Phase::Moment2a => {
                let ans = input.trim().to_string();
                present_answer.set(ans.clone());
                set_input_value.set(String::new());
                set_phase.set(Phase::CheckingVagueness);
                spawn_local(async move {
                    let is_vague = invoke::<_, bool>(
                        "check_ceremony_vagueness",
                        &VaguenessArg { answer: ans },
                    )
                    .await
                    .unwrap_or(false);
                    set_phase.set(if is_vague { Phase::Moment2b } else { Phase::Moment3 });
                });
            }
            Phase::Moment2b => {
                if !input.trim().is_empty() {
                    present_answer_refined.set(Some(input.trim().to_string()));
                }
                set_input_value.set(String::new());
                set_phase.set(Phase::Moment3);
            }
            Phase::Moment3 => {
                let seed = input.trim().to_string();
                garden_seed.set(seed.clone());
                set_input_value.set(String::new());
                set_phase.set(Phase::GeneratingNames);
                let holder = holder_name.get_untracked();
                let present = present_answer.get_untracked();
                spawn_local(async move {
                    let names = invoke::<_, Vec<NameOption>>(
                        "generate_greeter_names",
                        &NamesArg {
                            holder_name: holder,
                            present_answer: present,
                            garden_seed: seed,
                        },
                    )
                    .await
                    .unwrap_or_default();
                    name_options.set(names);
                    set_phase.set(Phase::Naming);
                });
            }
            _ => {}
        }
    };

    // ── Name selection (Naming phase) ─────────────────────────────────────────
    let confirm_name = move |name: String| {
        greeter_name.set(name.clone());
        set_phase.set(Phase::Confirmed);
        spawn_local(async move {
            sleep_ms(1500).await;
            // TODO(human): decide what defaults to apply when answers are empty.
            // The spec says: "Greeter crystal uses 'unnamed' as name" when skipping
            // at the naming step. Other missing answers produce empty-titled crystals.
            // Decision: apply those defaults here at the component boundary,
            // or pass through and let scaffold_palace_from_ceremony handle empties?
            // Currently: passes through as-is; scaffold handles empty greeter_name.
            let answers = CeremonyAnswers {
                parent_path: parent_path_store.get_value(),
                holder_name: holder_name.get_untracked(),
                present_answer: present_answer.get_untracked(),
                present_answer_refined: present_answer_refined.get_untracked(),
                garden_seed: garden_seed.get_untracked(),
                greeter_name: name,
                onboarding_complete: true,
            };
            set_phase.set(Phase::Done);
            on_complete.call(answers);
        });
    };

    // ── Skip entire ceremony ──────────────────────────────────────────────────
    let do_skip = move |_| {
        let holder = {
            let h = holder_name.get_untracked();
            if h.trim().is_empty() { "unknown".to_string() } else { h }
        };
        let greeter = {
            let g = greeter_name.get_untracked();
            if g.trim().is_empty() { "unnamed".to_string() } else { g }
        };
        let answers = CeremonyAnswers {
            parent_path: parent_path_store.get_value(),
            holder_name: holder,
            present_answer: present_answer.get_untracked(),
            present_answer_refined: present_answer_refined.get_untracked(),
            garden_seed: String::new(), // no plant seeded on skip
            greeter_name: greeter,
            onboarding_complete: false,
        };
        set_phase.set(Phase::Done);
        on_complete.call(answers);
    };

    // ── Key handler: Enter advances, Escape skips ─────────────────────────────
    let on_key = {
        let do_advance = do_advance.clone();
        move |ev: web_sys::KeyboardEvent| match ev.key().as_str() {
            "Enter" => do_advance(),
            _ => {}
        }
    };

    // ── CSS helpers ───────────────────────────────────────────────────────────
    let anim_cls = move |base: &'static str| move || {
        if anim_ready.get() {
            format!("{base} greeter-anim")
        } else {
            base.to_string()
        }
    };

    // ── View ──────────────────────────────────────────────────────────────────
    view! {
        <div class="greeter-overlay" role="dialog" aria-label="naming ceremony">
            // Skip is always available
            <button class="greeter-skip" on:click=do_skip>"skip"</button>

            // Orb (same markup as greeter.rs — reuses CSS animations unchanged)
            <div class="greeter-orb-wrap" aria-hidden="true">
                <div class=anim_cls("greeter-orb-ambient") />
                <div class=anim_cls("greeter-orb") />
                <div class=anim_cls("greeter-orb-halo") />
            </div>

            // Phrase (shown during Arrival; hidden once first question appears)
            {move || {
                let p = phase.get();
                if p == Phase::Arrival {
                    view! {
                        <p class=move || if phrase_fading.get() {
                            "greeter-phrase greeter-phrase-fading"
                        } else {
                            "greeter-phrase"
                        }>
                            {move || PHRASES[phrase_idx.get()]}
                        </p>
                    }.into_view()
                } else {
                    view! { <div /> }.into_view()
                }
            }}

            // Question + input area
            {move || {
                let p = phase.get();
                match p {
                    Phase::Arrival => view! { <div /> }.into_view(),

                    Phase::Moment1 => view! {
                        <div class="ceremony-question">
                            <p class="ceremony-q">"What should I call you?"</p>
                            <input
                                node_ref=input_ref
                                class="ceremony-input"
                                type="text"
                                autocomplete="off"
                                prop:value=move || input_value.get()
                                on:input=move |ev| set_input_value.set(event_target_value(&ev))
                                on:keydown=on_key.clone()
                            />
                            <button class="ceremony-next" on:click=move |_| do_advance()>"→"</button>
                        </div>
                    }.into_view(),

                    Phase::Moment2a => view! {
                        <div class="ceremony-question">
                            <p class="ceremony-q">"What are you in the middle of right now?"</p>
                            <input
                                node_ref=input_ref
                                class="ceremony-input"
                                type="text"
                                autocomplete="off"
                                prop:value=move || input_value.get()
                                on:input=move |ev| set_input_value.set(event_target_value(&ev))
                                on:keydown=on_key.clone()
                            />
                            <button class="ceremony-next" on:click=move |_| do_advance()>"→"</button>
                        </div>
                    }.into_view(),

                    Phase::CheckingVagueness => view! {
                        <div class="ceremony-question">
                            <p class="ceremony-q ceremony-waiting">"…"</p>
                        </div>
                    }.into_view(),

                    Phase::Moment2b => view! {
                        <div class="ceremony-question">
                            <p class="ceremony-q">"What's been occupying most of your attention lately?"</p>
                            <input
                                node_ref=input_ref
                                class="ceremony-input"
                                type="text"
                                autocomplete="off"
                                prop:value=move || input_value.get()
                                on:input=move |ev| set_input_value.set(event_target_value(&ev))
                                on:keydown=on_key.clone()
                            />
                            <button class="ceremony-next" on:click=move |_| do_advance()>"→"</button>
                        </div>
                    }.into_view(),

                    Phase::Moment3 => view! {
                        <div class="ceremony-question">
                            <p class="ceremony-q">"One more."</p>
                            <p class="ceremony-sub">"What are you curious about, beyond your immediate work?"</p>
                            <input
                                node_ref=input_ref
                                class="ceremony-input"
                                type="text"
                                autocomplete="off"
                                prop:value=move || input_value.get()
                                on:input=move |ev| set_input_value.set(event_target_value(&ev))
                                on:keydown=on_key.clone()
                            />
                            <button class="ceremony-next" on:click=move |_| do_advance()>"→"</button>
                        </div>
                    }.into_view(),

                    Phase::GeneratingNames => view! {
                        <div class="ceremony-question">
                            <p class="ceremony-q ceremony-waiting">"thinking of a name…"</p>
                        </div>
                    }.into_view(),

                    Phase::Naming => {
                        let opts = name_options.get();
                        view! {
                            <div class="ceremony-naming">
                                <p class="ceremony-q">"I need a name to tend this place well."</p>
                                <div class="ceremony-name-options">
                                    {opts.into_iter().map(|opt| {
                                        let name = opt.name.clone();
                                        let name_for_click = name.clone();
                                        view! {
                                            <button
                                                class="ceremony-name-option"
                                                on:click=move |_| confirm_name(name_for_click.clone())
                                            >
                                                <span class="ceremony-name">{name}</span>
                                                <span class="ceremony-note">{opt.note.clone()}</span>
                                            </button>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                                <div class="ceremony-custom-name">
                                    <input
                                        class="ceremony-input"
                                        type="text"
                                        placeholder="something else entirely"
                                        prop:value=move || input_value.get()
                                        on:input=move |ev| set_input_value.set(event_target_value(&ev))
                                        on:keydown={
                                            move |ev: web_sys::KeyboardEvent| {
                                                if ev.key() == "Enter" {
                                                    let v = input_value.get_untracked();
                                                    if !v.trim().is_empty() {
                                                        confirm_name(v.trim().to_string());
                                                    }
                                                }
                                            }
                                        }
                                    />
                                </div>
                            </div>
                        }.into_view()
                    },

                    Phase::Confirmed => {
                        let name = greeter_name.get();
                        view! {
                            <div class="ceremony-confirmed">
                                <p class="ceremony-q">{format!("{name} — that's who I am now.")}</p>
                            </div>
                        }.into_view()
                    },

                    Phase::Done => view! { <div /> }.into_view(),
                }
            }}

            <p class="greeter-notice">
                "ambient visual pulses · safe at any display · no health claims made"
            </p>
        </div>
    }
}
