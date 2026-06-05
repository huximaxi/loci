use crate::models::{
    ActivePalace, AlertItem, CronJobSnapshot, HandoverEntry, ManifestSummary, QuestlogItem,
};
use crate::tauri_bindings::{invoke, listen};
use leptos::*;
use leptos_router::A;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct PathArg<'a> {
    palace_path: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HandoversArg<'a> {
    palace_path: &'a str,
    limit: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CronDetailArg<'a> {
    palace_path: &'a str,
    key: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ChatQueryArg<'a> {
    prompt: &'a str,
    // "local" (Ollama, default) or "claude" (external · Anthropic, the online garden).
    provider: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HealthArg {
    // None → backend defaults to http://localhost:11434.
    base_url: Option<String>,
}

#[component]
pub fn Dashboard() -> impl IntoView {
    let active = expect_context::<RwSignal<ActivePalace>>();
    let (refetch_tick, set_refetch_tick) = create_signal(0u64);
    let (watcher_err, set_watcher_err) = create_signal::<Option<String>>(None);
    // The shared detail-pane focus. None = pane closed. Every drill-down tile
    // writes here; DetailPane reads here. One source of truth for "what's open".
    let selected = create_rw_signal::<Option<DetailTarget>>(None);
    // CiQ explainer modal. Some(key) = open, focused on the cron job key that
    // publishes the score. Separate from `selected`: the modal is a centered
    // glossary overlay, not a drill-down record in the side pane.
    let ciq_open = create_rw_signal::<Option<String>>(None);

    // Tracks the path the watcher is currently bound to + the unlisten handle.
    // When `active.path` changes we tear down the old listener BEFORE wiring a
    // new one. Skipping that step cascades ghost handlers on every switch.
    let last_path: StoredValue<Option<String>> = store_value(None);
    let active_unlisten: StoredValue<Option<js_sys::Function>> = store_value(None);

    // Four INDEPENDENT resources, one per dashboard section. Each fires its own
    // invoke eagerly on creation, so the four reads run CONCURRENTLY (total time
    // ≈ the slowest read, not the sum) and each section renders the instant its
    // own data lands — the "engine room boots box by box" behavior. `.get()`
    // returns None while a read is in flight, Some(_) once it resolves.
    let crons_res = create_resource(
        move || (active.get().path.clone(), refetch_tick.get()),
        |(path_opt, _tick)| async move {
            match path_opt {
                Some(path) => invoke::<_, Vec<CronJobSnapshot>>(
                    "read_cron_states",
                    &PathArg { palace_path: &path },
                )
                .await
                .unwrap_or_default(),
                None => Vec::new(),
            }
        },
    );
    let tasks_res = create_resource(
        move || (active.get().path.clone(), refetch_tick.get()),
        |(path_opt, _tick)| async move {
            match path_opt {
                Some(path) => invoke::<_, Vec<QuestlogItem>>(
                    "read_tasks",
                    &PathArg { palace_path: &path },
                )
                .await
                .unwrap_or_default(),
                None => Vec::new(),
            }
        },
    );
    let handovers_res = create_resource(
        move || (active.get().path.clone(), refetch_tick.get()),
        |(path_opt, _tick)| async move {
            match path_opt {
                Some(path) => invoke::<_, Vec<HandoverEntry>>(
                    "read_handovers",
                    &HandoversArg { palace_path: &path, limit: 6 },
                )
                .await
                .unwrap_or_default(),
                None => Vec::new(),
            }
        },
    );
    // read_manifest returns Err when no .schema/manifest.json exists. The inner
    // Option collapses that to None ("no manifest"), distinct from the outer None
    // ("still loading"). Empty != broken != loading.
    let manifest_res = create_resource(
        move || (active.get().path.clone(), refetch_tick.get()),
        |(path_opt, _tick)| async move {
            match path_opt {
                // Slim summary (counts + meta), not the full node graph: avoids a
                // ~180KB IPC + WASM deserialize that was the dashboard's load hitch.
                Some(path) => invoke::<_, ManifestSummary>(
                    "read_manifest_summary",
                    &PathArg { palace_path: &path },
                )
                .await
                .ok(),
                None => None,
            }
        },
    );

    // Watcher wiring effect. Fires once per distinct palace_path value.
    create_effect(move |_| {
        let path_opt = active.get().path;
        if last_path.get_value() == path_opt {
            return;
        }
        // Tear down before re-wire. Order: unlisten first, then drop our handle.
        if let Some(fn_handle) = active_unlisten.get_value() {
            let _ = fn_handle.call0(&JsValue::NULL);
            active_unlisten.set_value(None);
        }
        last_path.set_value(path_opt.clone());
        let Some(path) = path_opt else { return };
        spawn_local(async move {
            if let Err(e) =
                invoke::<_, ()>("start_state_watcher", &PathArg { palace_path: &path }).await
            {
                set_watcher_err.set(Some(format!("watcher failed: {e}")));
                return;
            }
            let cb = Closure::<dyn FnMut(JsValue)>::new(move |_event: JsValue| {
                set_refetch_tick.update(|n| *n += 1);
            });
            let unlisten_js = listen("state_changed", &cb).await;
            // The closure outlives this future via .forget(). Its memory is
            // bounded: at most one live closure per palace switch, and switches
            // are rare in the v0.6.0 single-palace flow.
            cb.forget();
            if let Ok(unlisten_js) = unlisten_js {
                if let Ok(unlisten_fn) = unlisten_js.dyn_into::<js_sys::Function>() {
                    active_unlisten.set_value(Some(unlisten_fn));
                }
            }
        });
    });

    view! {
        <main class="dashboard">
            <header class="wizard-header">
                <A href="/" attr:class="back">"< back"</A>
                <h2>"Palace dashboard"</h2>
                <span class="palace-path">
                    {move || active.get().path.clone().unwrap_or_else(|| "no palace".into())}
                </span>
            </header>

            {move || watcher_err.get().map(|e| view! {
                <p class="warn dashboard-warn">{e}</p>
            })}

            <ChatQuery companion=move || {
                active.get().manifest
                    .and_then(|m| m.companion)
                    .unwrap_or_else(|| "Loci".to_string())
            } />

            // Engine-room boot strip: a progress bar + verbose per-system chips
            // that light up as each read lands. Hides itself once all are online.
            {move || {
                let sys = [
                    ("cron jobs", crons_res.get().is_some()),
                    ("questlog", tasks_res.get().is_some()),
                    ("schema", manifest_res.get().is_some()),
                    ("handovers", handovers_res.get().is_some()),
                ];
                let total = sys.len();
                let online = sys.iter().filter(|(_, ok)| *ok).count();
                (online < total).then(|| view! {
                    <section class="boot-strip">
                        <div class="boot-head">
                            <span class="boot-title">"⚙ bringing the palace online"</span>
                            <span class="boot-count">{format!("{online}/{total} systems")}</span>
                        </div>
                        <div class="boot-bar">
                            <div class="boot-bar-fill"
                                style=format!("width:{}%", online * 100 / total)></div>
                        </div>
                        <div class="boot-systems">
                            {sys.into_iter().map(|(name, ok)| view! {
                                <span class=if ok { "boot-chip boot-online" } else { "boot-chip boot-loading" }>
                                    {if ok { "● " } else { "◌ " }}{name}
                                </span>
                            }).collect_view()}
                        </div>
                    </section>
                })
            }}

            // Each box boots dim, then lights up the instant its own data lands.
            // KPI strip + cron table share the crons read, so they light together.
            {move || match crons_res.get() {
                None => boot_box("cron systems").into_view(),
                Some(c) => view! {
                    <KpiStrip crons=c.clone() selected=selected ciq_open=ciq_open />
                    <CronSection crons=c selected=selected />
                }.into_view(),
            }}
            {move || match tasks_res.get() {
                None => boot_box("questlog").into_view(),
                Some(t) => view! { <QuestlogSection tasks=t selected=selected /> }.into_view(),
            }}
            {move || match manifest_res.get() {
                None => boot_box("schema manifest").into_view(),
                Some(m) => view! { <SchemaSection manifest=m /> }.into_view(),
            }}
            {move || match handovers_res.get() {
                None => boot_box("handovers").into_view(),
                Some(h) => view! { <HandoversSection handovers=h /> }.into_view(),
            }}

            <DetailPane selected=selected active=active />
            <CiqModal ciq_open=ciq_open active=active />
        </main>
    }
}

/// A dim placeholder box shown while a section's read is in flight. The instant
/// the section's resource resolves, the real section replaces this — the box
/// "lights up". Reuses the section box rhythm so the dashboard reads as an
/// engine room booting up, not a blank screen.
fn boot_box(name: &'static str) -> impl IntoView {
    view! {
        <section class="boot-box">
            <span class="boot-box-dot"></span>
            <span class="boot-box-label">{format!("bringing {name} online…")}</span>
        </section>
    }
}

#[component]
fn KpiStrip(
    crons: Vec<CronJobSnapshot>,
    selected: RwSignal<Option<DetailTarget>>,
    ciq_open: RwSignal<Option<String>>,
) -> impl IntoView {
    // Capture the job KEY alongside the value: the CiQ modal and the alert list
    // both lazy-fetch full state.json via read_cron_detail, which needs the key.
    let ciq = crons
        .iter()
        .find_map(|j| j.ciq.map(|v| (v, j.ciq_delta, j.key.clone())));
    let alerts = crons
        .iter()
        .find_map(|j| j.alert_count.map(|c| (c, j.key.clone())))
        .unwrap_or((0, String::new()));
    let heartbeats_ok = crons
        .iter()
        .filter(|j| j.job.starts_with("heartbeat-"))
        .filter(|j| j.status.as_deref() == Some("ok"))
        .count();
    let heartbeats_total = crons
        .iter()
        .filter(|j| j.job.starts_with("heartbeat-"))
        .count();

    view! {
        <section class="kpi-strip">
            {match ciq {
                Some((v, delta, key)) => view! {
                    <div class="kpi kpi-clickable" title="what is this number?"
                        on:click=move |_| ciq_open.set(Some(key.clone()))>
                        <span class="kpi-label">"CIQ" <span class="kpi-info">"ⓘ"</span></span>
                        <span class="kpi-value">{format!("{:.1}", v)}</span>
                        {delta.map(|d| view! {
                            <span class={if d >= 0.0 { "kpi-delta up" } else { "kpi-delta down" }}>
                                {format!("{}{:.1}", if d >= 0.0 { "+" } else { "" }, d)}
                            </span>
                        })}
                    </div>
                }.into_view(),
                None => view! {
                    <div class="kpi">
                        <span class="kpi-label">"CIQ"</span>
                        <span class="kpi-value muted">"n/a"</span>
                    </div>
                }.into_view(),
            }}
            {
                let (count, alert_key) = alerts;
                let clickable = count > 0 && !alert_key.is_empty();
                view! {
                    <div class="kpi" class:kpi-clickable=clickable
                        title=move || if clickable { "click to inspect alerts" } else { "" }
                        on:click=move |_| {
                            if clickable {
                                selected.set(Some(DetailTarget::AlertList {
                                    key: alert_key.clone(),
                                    count,
                                }));
                            }
                        }>
                        <span class="kpi-label">"alerts"</span>
                        <span class={if count > 0 { "kpi-value warn" } else { "kpi-value" }}>
                            {count}
                        </span>
                    </div>
                }
            }
            <div class="kpi">
                <span class="kpi-label">"heartbeats ok"</span>
                <span class="kpi-value">{format!("{}/{}", heartbeats_ok, heartbeats_total)}</span>
            </div>
        </section>
    }
}

#[component]
fn CronSection(
    crons: Vec<CronJobSnapshot>,
    selected: RwSignal<Option<DetailTarget>>,
) -> impl IntoView {
    view! {
        <section class="cron-section">
            <h3>"Cron jobs"</h3>
            {if crons.is_empty() {
                view! { <p class="muted">"No cron jobs yet."</p> }.into_view()
            } else {
                view! {
                    <table class="cron-table">
                        <thead>
                            <tr>
                                <th>"job"</th>
                                <th>"status"</th>
                                <th>"summary"</th>
                                <th>"last run"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {crons.into_iter().map(|j| {
                                let status = j.status.clone().unwrap_or_default();
                                let pulse = j.pulse.clone();
                                let badge_class = match status.as_str() {
                                    "ok" => "badge badge-ok",
                                    "warn" => "badge badge-warn",
                                    "error" | "err" => "badge badge-err",
                                    _ => "badge",
                                };
                                let target = DetailTarget::CronJob { key: j.key.clone(), label: j.job.clone() };
                                let target_click = target.clone();
                                view! {
                                    <tr class="row-clickable" title="click for detail"
                                        class:row-selected=move || selected.get().as_ref() == Some(&target)
                                        on:click=move |_| selected.set(Some(target_click.clone()))>
                                        <td><code>{j.job}</code></td>
                                        <td>
                                            <span class=badge_class>{status}</span>
                                            {pulse.map(|p| view! { <span class="pulse">{p}</span> })}
                                        </td>
                                        <td class="summary-cell">{j.summary.unwrap_or_default()}</td>
                                        <td class="muted small">{fmt_iso_to_local(j.last_run.as_deref())}</td>
                                    </tr>
                                }
                            }).collect_view()}
                        </tbody>
                    </table>
                }.into_view()
            }}
        </section>
    }
}

#[component]
fn QuestlogSection(
    tasks: Vec<QuestlogItem>,
    selected: RwSignal<Option<DetailTarget>>,
) -> impl IntoView {
    view! {
        <section class="questlog-section">
            <h3>"Questlog"</h3>
            {if tasks.is_empty() {
                view! { <p class="muted">"The board is quiet."</p> }.into_view()
            } else {
                group_by_track(tasks).into_iter().map(|(track, items)| {
                    let open_count = items.iter().filter(|t| !t.done).count();
                    view! {
                        <div class="quest-track">
                            <div class="quest-track-head">
                                <span class="quest-track-name">{track}</span>
                                <span class="quest-track-count muted small">{format!("{open_count} open")}</span>
                            </div>
                            <ul class="questlog-list">
                                {items.into_iter().map(|t| {
                                    let (badge_text, badge_class) = quest_state(&t);
                                    let is_done = t.done;
                                    let target = DetailTarget::Questlog(t.clone());
                                    let target_click = target.clone();
                                    view! {
                                        <li class="row-clickable" title="click for detail"
                                            class:done=is_done
                                            class:row-selected=move || selected.get().as_ref() == Some(&target)
                                            on:click=move |_| selected.set(Some(target_click.clone()))>
                                            <span class=badge_class>{badge_text}</span>
                                            <span class="quest-title">{t.title}</span>
                                        </li>
                                    }
                                }).collect_view()}
                            </ul>
                        </div>
                    }
                }).collect_view().into_view()
            }}
        </section>
    }
}

#[component]
fn SchemaSection(manifest: Option<ManifestSummary>) -> impl IntoView {
    view! {
        <section class="manifest-section">
            <h3>"Schema manifest"</h3>
            {match manifest {
                None => view! { <p class="muted">"No schema manifest yet."</p> }.into_view(),
                Some(m) => {
                    let (fresh_label, fresh_class) = manifest_freshness(&m.captured_ts_utc);
                    let tree_hash_short: String = m.tree_hash.chars().take(12).collect();
                    view! {
                        <div class="manifest-meta">
                            <span class="manifest-version">{format!("v{}", m.manifest_version)}</span>
                            <span class="muted">{m.vocabulary.clone()}</span>
                            <span class=fresh_class>{fresh_label}</span>
                        </div>
                        <table class="manifest-table">
                            <tbody>
                                <tr><td class="muted">"nodes"</td><td>{m.node_count}</td></tr>
                                <tr><td class="muted">"relations"</td><td>{m.relation_count}</td></tr>
                                <tr><td class="muted">"edges"</td><td>{m.edge_count}</td></tr>
                                <tr><td class="muted">"captured"</td><td class="small">{fmt_iso_to_local(Some(&m.captured_ts_utc))}</td></tr>
                                <tr><td class="muted">"tree hash"</td><td><code class="small">{tree_hash_short}</code></td></tr>
                            </tbody>
                        </table>
                    }.into_view()
                }
            }}
        </section>
    }
}

#[component]
fn HandoversSection(handovers: Vec<HandoverEntry>) -> impl IntoView {
    view! {
        <section class="handovers-section">
            <h3>"Recent handovers"</h3>
            {if handovers.is_empty() {
                view! { <p class="muted">"No handovers folder yet."</p> }.into_view()
            } else {
                view! {
                    <ul class="handovers-list">
                        {handovers.into_iter().map(|h| view! {
                            <li>
                                <span class="handover-name"><code>{h.filename}</code></span>
                                <span class="muted small">{fmt_mtime(h.mtime)}</span>
                            </li>
                        }).collect_view()}
                    </ul>
                }.into_view()
            }}
        </section>
    }
}

/// The shared detail surface. Reads `selected`; renders nothing when None.
/// Each DetailTarget variant gets one arm. This is the whole interactivity
/// contract: tiles write `selected`, this reads it. Map/chat later add arms.
#[component]
fn DetailPane(
    selected: RwSignal<Option<DetailTarget>>,
    active: RwSignal<ActivePalace>,
) -> impl IntoView {
    // Lazy cron-detail fetch, keyed on the selected cron key. Fires only when a
    // CronJob is focused; other targets carry their own data and skip the IPC.
    let cron_detail = create_resource(
        move || match selected.get() {
            // Both the cron drill-down and the alert list read the same state.json
            // (alerts live inside the publishing job's state). One resource serves
            // both, keyed on whichever target carries a dir key.
            Some(DetailTarget::CronJob { key, .. })
            | Some(DetailTarget::AlertList { key, .. }) => active.get().path.map(|p| (p, key)),
            _ => None,
        },
        |arg| async move {
            let Some((path, key)) = arg else { return None };
            let v: Result<serde_json::Value, String> = invoke(
                "read_cron_detail",
                &CronDetailArg { palace_path: &path, key: &key },
            )
            .await;
            Some(v)
        },
    );

    view! {
        {move || selected.get().map(|target| {
            let body = match target {
                DetailTarget::CronJob { label, .. } => view! {
                    <h4 class="detail-title"><code>{label}</code></h4>
                    <Suspense fallback=move || view! { <p class="muted">"Loading…"</p> }>
                        {move || cron_detail.get().flatten().map(|res| match res {
                            Ok(v) => view! {
                                <pre class="detail-json">{
                                    serde_json::to_string_pretty(&v)
                                        .unwrap_or_else(|_| "unrenderable".into())
                                }</pre>
                            }.into_view(),
                            Err(e) => view! { <p class="warn">{e}</p> }.into_view(),
                        })}
                    </Suspense>
                }.into_view(),
                DetailTarget::Questlog(item) => render_questlog_detail(item, selected),
                DetailTarget::AlertList { count, .. } => view! {
                    <h4 class="detail-title">{format!("Alerts · {count} surfaced")}</h4>
                    <p class="muted small detail-note">
                        "Read-only for now. Actioning an alert (mark solved) is a write \
                         to palace state, sequenced to Phase 4c behind Cipher guardrails."
                    </p>
                    <Suspense fallback=move || view! { <p class="muted">"Loading…"</p> }>
                        {move || cron_detail.get().flatten().map(|res| match res {
                            Ok(v) => render_alert_list(&v),
                            Err(e) => view! { <p class="warn">{e}</p> }.into_view(),
                        })}
                    </Suspense>
                }.into_view(),
            };
            view! {
                <aside class="detail-pane">
                    <button class="detail-close" on:click=move |_| selected.set(None)>"close ×"</button>
                    {body}
                </aside>
            }
        })}
    }
}

/// Detail body for a focused quest. Wizard register: a state line, the title,
/// the full body, and — the Outer Wilds move — the cron jobs this quest
/// references rendered as live chips. Clicking a chip re-focuses the pane on
/// that job via the same `selected` signal: the quest is a node, not a leaf.
fn render_questlog_detail(item: QuestlogItem, selected: RwSignal<Option<DetailTarget>>) -> View {
    let (state_word, state_class) = quest_state(&item);
    let refs = extract_cron_refs(&item.body);
    let chips = (!refs.is_empty()).then(|| {
        view! {
            <div class="detail-chips">
                <span class="muted small">"linked jobs: "</span>
                {refs.into_iter().map(|key| {
                    let target = DetailTarget::CronJob { key: key.clone(), label: key.clone() };
                    view! {
                        <button class="detail-chip"
                            on:click=move |_| selected.set(Some(target.clone()))>
                            {format!("↪ {key}")}
                        </button>
                    }
                }).collect_view()}
            </div>
        }
    });
    let body_display = strip_leading_bold(&item.body);
    view! {
        <h4 class="detail-title">{item.title}</h4>
        <p><span class=state_class>{state_word}</span></p>
        <div class="detail-body">{format_inline(&body_display)}</div>
        {chips}
    }
    .into_view()
}

/// Renders the `alerts` array from a publishing job's state.json as a list of
/// rows. Each alert shows severity, the job it fired on, the condition, and
/// whether it was posted. Read-only: no solve action (deferred to Phase 4c).
fn render_alert_list(detail: &serde_json::Value) -> View {
    let alerts: Vec<AlertItem> = detail
        .get("alerts")
        .cloned()
        .and_then(|a| serde_json::from_value(a).ok())
        .unwrap_or_default();
    if alerts.is_empty() {
        return view! { <p class="muted">"No alerts in this state."</p> }.into_view();
    }
    view! {
        <ul class="alert-list">
            {alerts.into_iter().map(|a| view! {
                <li class="alert-row">
                    <span class="alert-sev">{a.severity}</span>
                    <div class="alert-meta">
                        <span class="alert-job"><code>{a.job}</code></span>
                        <span class="alert-cond">{a.condition}</span>
                    </div>
                    <span class={if a.posted { "badge badge-ok" } else { "badge" }}>
                        {if a.posted { "posted" } else { "pending" }}
                    </span>
                </li>
            }).collect_view()}
        </ul>
    }
    .into_view()
}

/// The CiQ explainer. A centered modal (not the side pane) because it answers
/// "what IS this number?" rather than drilling into a record. Lazy-fetches the
/// publishing job's state.json and renders the weighted contributor breakdown
/// plus recent history, so the score reads as composition, not a magic number.
#[component]
fn CiqModal(
    ciq_open: RwSignal<Option<String>>,
    active: RwSignal<ActivePalace>,
) -> impl IntoView {
    let detail = create_resource(
        move || match (ciq_open.get(), active.get().path) {
            (Some(key), Some(path)) => Some((path, key)),
            _ => None,
        },
        |arg| async move {
            let Some((path, key)) = arg else { return None };
            let v: Result<serde_json::Value, String> =
                invoke("read_cron_detail", &CronDetailArg { palace_path: &path, key: &key }).await;
            Some(v)
        },
    );

    view! {
        {move || ciq_open.get().map(|_| view! {
            <div class="modal-backdrop" on:click=move |_| ciq_open.set(None)>
                // Stop propagation so clicking the card doesn't close the modal.
                <div class="modal-card" on:click=|ev| ev.stop_propagation()>
                    <button class="detail-close" on:click=move |_| ciq_open.set(None)>"close ×"</button>
                    <h4 class="detail-title">"CIQ — Co-intelligence Quotient"</h4>
                    <p class="modal-lede">
                        "A 0–100 composite of how well we're working together this window. \
                         It is a weighted blend of four signals — not a grade, a gauge. \
                         Each contributor below adds its weighted share to the total."
                    </p>
                    <Suspense fallback=move || view! { <p class="muted">"Loading…"</p> }>
                        {move || detail.get().flatten().map(|res| match res {
                            Ok(v) => render_ciq_breakdown(&v),
                            Err(e) => view! { <p class="warn">{e}</p> }.into_view(),
                        })}
                    </Suspense>
                </div>
            </div>
        })}
    }
}

/// Humanises a contributor key from ciq_contributors into a readable label.
fn ciq_contributor_label(key: &str) -> &'static str {
    match key {
        "co_int" => "Co-intelligence (eval score)",
        "cfs" => "Context fidelity (CFS)",
        "velocity" => "Velocity",
        "annex_coverage" => "Annex coverage",
        _ => "Other",
    }
}

/// Renders the contributor table + history from a coworking-eval state.json.
fn render_ciq_breakdown(detail: &serde_json::Value) -> View {
    let total = detail.get("ciq").and_then(|v| v.as_f64());
    let mut contributors: Vec<(String, f64, f64, f64)> = detail
        .get("ciq_contributors")
        .and_then(|c| c.as_object())
        .map(|obj| {
            obj.iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        v.get("value").and_then(|x| x.as_f64()).unwrap_or(0.0),
                        v.get("weight").and_then(|x| x.as_f64()).unwrap_or(0.0),
                        v.get("contribution").and_then(|x| x.as_f64()).unwrap_or(0.0),
                    )
                })
                .collect()
        })
        .unwrap_or_default();
    // Largest contribution first — the eye lands on what's driving the score.
    contributors.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal));

    let history: Vec<(String, f64)> = detail
        .get("ciq_history")
        .and_then(|h| h.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|e| {
                    Some((
                        e.get("date")?.as_str()?.to_string(),
                        e.get("ciq")?.as_f64()?,
                    ))
                })
                .collect()
        })
        .unwrap_or_default();

    view! {
        {total.map(|t| view! {
            <div class="ciq-total">
                <span class="kpi-value">{format!("{:.1}", t)}</span>
                <span class="muted small">" / 100"</span>
            </div>
        })}
        <table class="ciq-table">
            <thead>
                <tr><th>"signal"</th><th>"value"</th><th>"weight"</th><th>"adds"</th></tr>
            </thead>
            <tbody>
                {contributors.into_iter().map(|(k, value, weight, contribution)| view! {
                    <tr>
                        <td>{ciq_contributor_label(&k)}</td>
                        <td class="small">{format!("{:.2}", value)}</td>
                        <td class="small muted">{format!("{:.0}%", weight * 100.0)}</td>
                        <td>
                            <div class="ciq-bar-wrap" title=format!("{:.1} points", contribution)>
                                <div class="ciq-bar" style=format!("width:{:.0}%", contribution)></div>
                                <span class="ciq-bar-label">{format!("{:.1}", contribution)}</span>
                            </div>
                        </td>
                    </tr>
                }).collect_view()}
            </tbody>
        </table>
        {(!history.is_empty()).then(|| view! {
            <div class="ciq-history">
                <span class="muted small">"recent: "</span>
                {history.into_iter().take(6).map(|(date, ciq)| view! {
                    <span class="ciq-hist-pt" title=date>{format!("{:.0}", ciq)}</span>
                }).collect_view()}
            </div>
        })}
    }
    .into_view()
}

/// Phase 4a · chat as QUERY. A read-only question box: type, ask, read. Routes
/// to the `chat_query` Tauri command, which goes through the inference TRAIT
/// (not a vendor) and is fail-closed — an unreachable backend surfaces an honest
/// error, never a silent external call. No palace grounding yet (RAG deferred)
/// and no state mutation: this slice proves the wiring + UX.
///
/// The local garden is ACTIVE BY DEFAULT (privacy-by-default is satisfied by
/// locality). A videogame-style PAUSE button silences it momentarily so you can
/// roam settings / read-only browse; unpause returns to local garden mode. A
/// readiness chip probes on open so the brain's state (awake/asleep/paused) is
/// legible without typing.
#[component]
fn ChatQuery(
    /// Companion name for the attribution line. Falls back to "Loci" for
    /// legacy palaces that have no `> Companion:` in PALACE.md.
    #[prop(into)]
    companion: Signal<String>,
) -> impl IntoView {
    let (prompt, set_prompt) = create_signal(String::new());
    let (answer, set_answer) = create_signal::<Option<Result<String, String>>>(None);
    let (pending, set_pending) = create_signal(false);
    // Which brain: false = local garden (Ollama), true = Claude (external · Anthropic,
    // the user's own license). Crossing this line leaves the local garden, so the UI
    // marks it loudly and the per-answer attribution records which brain replied.
    let external = create_rw_signal(false);
    let (answer_external, set_answer_external) = create_signal(false);
    // Videogame pause: ephemeral runtime state, not persisted. When paused the
    // UI never calls chat_query — the brain is silenced at the source.
    let paused = create_rw_signal(false);
    // Readiness probe. Re-runs on mount and whenever the tick bumps (after a
    // query, or on unpause) so the chip reflects live reachability.
    let (health_tick, set_health_tick) = create_signal(0u32);
    let health = create_resource(
        move || health_tick.get(),
        |_| async move {
            let ok: Result<bool, String> =
                invoke("check_ollama_health", &HealthArg { base_url: None }).await;
            ok.unwrap_or(false)
        },
    );

    let ask = move || {
        let p = prompt.get();
        if paused.get() || p.trim().is_empty() || pending.get() {
            return;
        }
        let is_ext = external.get();
        set_pending.set(true);
        set_answer.set(None);
        spawn_local(async move {
            let provider = if is_ext { "claude" } else { "local" };
            let res: Result<String, String> =
                invoke("chat_query", &ChatQueryArg { prompt: &p, provider }).await;
            // Record which brain answered so the attribution line is honest even
            // if the toggle is flipped before the next ask.
            set_answer_external.set(is_ext);
            set_answer.set(Some(res));
            set_pending.set(false);
            // Refresh readiness: a failed call often means the garden went to sleep.
            set_health_tick.update(|n| *n += 1);
        });
    };

    // The readiness chip: paused wins, else awake/asleep from the live probe.
    let chip = move || {
        if paused.get() {
            return view! { <span class="brain-chip brain-paused">"paused"</span> }.into_view();
        }
        match health.get() {
            None => view! { <span class="brain-chip brain-checking">"checking…"</span> }.into_view(),
            Some(true) => view! { <span class="brain-chip brain-awake">"awake"</span> }.into_view(),
            Some(false) => view! { <span class="brain-chip brain-asleep">"asleep"</span> }.into_view(),
        }
    };

    view! {
        <section class="chat-query">
            <div class="chat-row">
                <span class="brain-label">"second brain"</span>
                {chip}
                // Brain switch: local garden (Ollama) ↔ Claude (external · Anthropic).
                <button
                    class=move || if external.get() { "brain-toggle brain-toggle-ext" } else { "brain-toggle" }
                    title="switch brain · local garden vs Claude (external API, your Anthropic license)"
                    on:click=move |_| external.update(|e| *e = !*e)>
                    {move || if external.get() { "▲ Claude" } else { "◆ local" }}
                </button>
                // Loud marker: you have left the local garden for an external API.
                {move || external.get().then(|| view! {
                    <span class="brain-ext-badge" title="requests go to Anthropic's API, billed to your license">
                        "external · Anthropic · your license"
                    </span>
                })}
                <input
                    class=move || if external.get() { "chat-input chat-input-ext" } else { "chat-input" }
                    type="text"
                    placeholder=move || if paused.get() {
                        "paused · unpause to ask"
                    } else if external.get() {
                        "ask Claude… (external · your Anthropic license)"
                    } else {
                        "ask the palace… (local garden)"
                    }
                    prop:value=move || prompt.get()
                    disabled=move || paused.get()
                    on:input=move |ev| set_prompt.set(event_target_value(&ev))
                    on:keydown=move |ev| if ev.key() == "Enter" { ask() }
                />
                <button class="chat-pause" title="pause the local brain (read-only roam)"
                    on:click=move |_| {
                        paused.update(|p| *p = !*p);
                        if !paused.get() { set_health_tick.update(|n| *n += 1); }
                    }>
                    {move || if paused.get() { "▶ resume" } else { "⏸ pause" }}
                </button>
                <button class="primary" disabled=move || pending.get() || paused.get()
                    on:click=move |_| ask()>
                    {move || if pending.get() { "asking…" } else { "ask" }}
                </button>
            </div>
            {move || answer.get().map(|res| {
                let ext = answer_external.get();
                view! {
                    // A separate line per brain: which brain answered, local vs external.
                    <div class=move || if ext { "chat-attrib chat-attrib-ext" } else { "chat-attrib" }>
                        {move || {
                            let name = companion.get();
                            if ext {
                                format!("▲ {name} · Claude · external · Anthropic · your license")
                            } else {
                                format!("◆ {name} · local garden")
                            }
                        }}
                    </div>
                    {match res {
                        Ok(text) => view! { <div class="chat-answer">{text}</div> }.into_view(),
                        Err(e) => view! {
                            <div class="chat-answer chat-error">
                                <strong>"fail-closed: "</strong>{e}
                            </div>
                        }.into_view(),
                    }}
                }
            })}
        </section>
    }
}

/// The quest body repeats its bold lead title (already shown as the heading).
/// Strip a leading `**...**` (and a trailing comma) so the body reads as prose.
fn strip_leading_bold(body: &str) -> String {
    let b = body.trim_start();
    if let Some(rest) = b.strip_prefix("**") {
        if let Some(end) = rest.find("**") {
            let after = rest[end + 2..].trim_start();
            let after = after.strip_prefix(',').unwrap_or(after).trim_start();
            return after.to_string();
        }
    }
    b.to_string()
}

/// Minimal inline markdown: renders `**bold**` and `` `code` `` spans, leaving
/// everything else as literal text. No dependency, no full markdown parser —
/// just the two markers our quest bodies actually use. Unmatched markers fall
/// through as literal characters.
fn format_inline(text: &str) -> View {
    let chars: Vec<char> = text.chars().collect();
    let mut out: Vec<View> = Vec::new();
    let mut buf = String::new();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '*' && i + 1 < chars.len() && chars[i + 1] == '*' {
            if let Some(end) = find_double_star(&chars, i + 2) {
                flush_text(&mut out, &mut buf);
                let inner: String = chars[i + 2..end].iter().collect();
                out.push(view! { <strong>{inner}</strong> }.into_view());
                i = end + 2;
                continue;
            }
        }
        if chars[i] == '`' {
            if let Some(end) = (i + 1..chars.len()).find(|&j| chars[j] == '`') {
                flush_text(&mut out, &mut buf);
                let inner: String = chars[i + 1..end].iter().collect();
                out.push(view! { <code>{inner}</code> }.into_view());
                i = end + 1;
                continue;
            }
        }
        buf.push(chars[i]);
        i += 1;
    }
    flush_text(&mut out, &mut buf);
    out.into_view()
}

fn flush_text(out: &mut Vec<View>, buf: &mut String) {
    if !buf.is_empty() {
        out.push(view! { <span>{std::mem::take(buf)}</span> }.into_view());
    }
}

fn find_double_star(chars: &[char], from: usize) -> Option<usize> {
    (from..chars.len().saturating_sub(1)).find(|&i| chars[i] == '*' && chars[i + 1] == '*')
}

/// Groups quests by track, preserving first-seen track order; within each track,
/// open quests sort before sealed ones.
fn group_by_track(tasks: Vec<QuestlogItem>) -> Vec<(String, Vec<QuestlogItem>)> {
    let mut groups: Vec<(String, Vec<QuestlogItem>)> = Vec::new();
    for t in tasks {
        if let Some(g) = groups.iter_mut().find(|(name, _)| name == &t.track) {
            g.1.push(t);
        } else {
            groups.push((t.track.clone(), vec![t]));
        }
    }
    for (_, items) in groups.iter_mut() {
        items.sort_by_key(|t| t.done);
    }
    groups
}

/// Wizard-register state for a quest. Open threads sit warm; ones open a long
/// time read as "waiting" (not scolded — a fact, no whip); done quests are
/// "sealed", a trace left behind rather than a checkmark farmed.
fn quest_state(item: &QuestlogItem) -> (&'static str, &'static str) {
    if item.done {
        return ("sealed", "badge badge-ok");
    }
    match lead_date_age_days(&item.title) {
        Some(age) if age > 21 => ("waiting", "badge badge-err"),
        _ => ("open", "badge badge-warn"),
    }
}

/// Quests often lead with "YYYY-MM-DD, ...". Returns the age in days of that
/// leading date, or None if the title doesn't start with one.
fn lead_date_age_days(title: &str) -> Option<i64> {
    let date_str = title.get(0..10)?;
    let b = date_str.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return None;
    }
    let then = js_sys::Date::new(&JsValue::from_str(date_str)).get_time();
    if then.is_nan() {
        return None;
    }
    Some(((js_sys::Date::now() - then) / (1000.0 * 60.0 * 60.0 * 24.0)).floor() as i64)
}

/// Pulls cron dir keys out of a quest body by scanning for `cron/<key>` path
/// fragments (e.g. "_palace/cron/palace-sync/state.json" → "palace-sync").
/// The key stops at the next path separator, so it matches the filesystem key
/// read_cron_detail expects. De-duplicated, in first-seen order.
fn extract_cron_refs(body: &str) -> Vec<String> {
    const NEEDLE: &str = "cron/";
    let mut keys: Vec<String> = Vec::new();
    let mut rest = body;
    while let Some(pos) = rest.find(NEEDLE) {
        let after = &rest[pos + NEEDLE.len()..];
        let key: String = after
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        if !key.is_empty() && !keys.contains(&key) {
            keys.push(key);
        }
        rest = after;
    }
    keys
}

/// What the detail pane is currently focused on. The shared-signal contract:
/// every drill-down surface (cron, questlog, and later the map/chat) sets this
/// one signal, and `DetailPane` matches on it. Adding a new drill-down surface
/// means adding a variant here and an arm in `DetailPane` — nothing else.
#[derive(Debug, Clone, PartialEq)]
enum DetailTarget {
    /// `key` is the filesystem dir key (path-safe); `label` is the display name.
    CronJob { key: String, label: String },
    Questlog(QuestlogItem),
    /// The list of surfaced alerts. `key` is the cron dir key that publishes the
    /// `alerts` array (e.g. "alert-watcher-daily"); `count` is shown in the title.
    AlertList { key: String, count: usize },
}

/// "2026-05-18T16:00:00.353085+00:00" maps to "2026-05-18  16:00".
/// Best-effort: on parse failure returns the original truncated to 16 chars.
fn fmt_iso_to_local(s: Option<&str>) -> String {
    let Some(s) = s else { return "n/a".into() };
    if let (Some(t_pos), true) = (s.find('T'), s.len() >= 16) {
        let date = &s[..t_pos];
        let time = &s[t_pos + 1..(t_pos + 6).min(s.len())];
        return format!("{date}  {time}");
    }
    s.chars().take(16).collect()
}

/// Maps the manifest's `captured_ts_utc` to a (label, css_class) freshness badge.
/// This is the panel's whole reason for existing: the manifest went stale silently
/// once, and a green dry-run hid it. The badge has to make "stale" legible at a glance.
///
/// `captured_ts_utc` is an ISO-8601 string, e.g. "2026-05-18T16:00:00+00:00".
/// `js_sys::Date::new(&JsValue::from_str(s)).get_time()` parses it to epoch ms,
/// and `js_sys::Date::now()` gives current epoch ms (see fmt_mtime for the pattern).
/// Return css_class one of: "badge badge-ok" / "badge badge-warn" / "badge badge-err".
fn manifest_freshness(captured_ts_utc: &str) -> (String, &'static str) {
    let then_ms = js_sys::Date::new(&JsValue::from_str(captured_ts_utc)).get_time();
    if then_ms.is_nan() {
        return ("unknown".into(), "badge");
    }
    let age_days = ((js_sys::Date::now() - then_ms) / (1000.0 * 60.0 * 60.0 * 24.0)).floor() as i64;
    match age_days {
        d if d < 7 => ("fresh".into(), "badge badge-ok"),
        d if d < 28 => (format!("{d}d old"), "badge badge-warn"),
        d => (format!("{d}d stale"), "badge badge-err"),
    }
}

/// Unix seconds to "X days ago" using js_sys::Date for current time.
fn fmt_mtime(secs: f64) -> String {
    let now_ms = js_sys::Date::now();
    let then_ms = secs * 1000.0;
    let delta_ms = now_ms - then_ms;
    let days = (delta_ms / (1000.0 * 60.0 * 60.0 * 24.0)).floor() as i64;
    match days {
        d if d < 0 => "just now".into(),
        0 => "today".into(),
        1 => "yesterday".into(),
        d if d < 7 => format!("{d}d ago"),
        d if d < 30 => format!("{}w ago", d / 7),
        d => format!("{}mo ago", d / 30),
    }
}
