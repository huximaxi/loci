use crate::models::{ActivePalace, CronJobSnapshot, HandoverEntry};
use crate::tauri_bindings::{invoke, listen};
use leptos::*;
use leptos_router::A;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[derive(Serialize)]
struct PathArg<'a> {
    palace_path: &'a str,
}

#[derive(Serialize)]
struct HandoversArg<'a> {
    palace_path: &'a str,
    limit: usize,
}

#[component]
pub fn Dashboard() -> impl IntoView {
    let active = expect_context::<RwSignal<ActivePalace>>();
    let (refetch_tick, set_refetch_tick) = create_signal(0u64);
    let (watcher_err, set_watcher_err) = create_signal::<Option<String>>(None);

    // Tracks the path the watcher is currently bound to + the unlisten handle.
    // When `active.path` changes we tear down the old listener BEFORE wiring a
    // new one. Skipping that step cascades ghost handlers on every switch.
    let last_path: StoredValue<Option<String>> = store_value(None);
    let active_unlisten: StoredValue<Option<js_sys::Function>> = store_value(None);

    let dashboard_data = create_resource(
        move || (active.get().path.clone(), refetch_tick.get()),
        |(path_opt, _tick)| async move {
            let Some(path) = path_opt else {
                return DashboardData::default();
            };
            let crons: Vec<CronJobSnapshot> = invoke(
                "read_cron_states",
                &PathArg { palace_path: &path },
            )
            .await
            .unwrap_or_default();
            let handovers: Vec<HandoverEntry> = invoke(
                "read_handovers",
                &HandoversArg {
                    palace_path: &path,
                    limit: 6,
                },
            )
            .await
            .unwrap_or_default();
            DashboardData { crons, handovers }
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

            <Suspense fallback=move || view! { <p class="muted">"Reading state…"</p> }>
                {move || dashboard_data.get().map(|data| view! {
                    <DashboardSections data=data />
                })}
            </Suspense>
        </main>
    }
}

#[component]
fn DashboardSections(data: DashboardData) -> impl IntoView {
    let ciq = data.crons.iter().find_map(|j| j.ciq.map(|v| (v, j.ciq_delta)));
    let alerts = data.crons.iter().find_map(|j| j.alert_count).unwrap_or(0);
    let heartbeats_ok = data
        .crons
        .iter()
        .filter(|j| j.job.starts_with("heartbeat-"))
        .filter(|j| j.status.as_deref() == Some("ok"))
        .count();
    let heartbeats_total = data
        .crons
        .iter()
        .filter(|j| j.job.starts_with("heartbeat-"))
        .count();
    let crons = data.crons.clone();
    let handovers = data.handovers.clone();

    view! {
        <section class="kpi-strip">
            <div class="kpi">
                <span class="kpi-label">"CIQ"</span>
                {ciq.map(|(v, delta)| view! {
                    <span class="kpi-value">{format!("{:.1}", v)}</span>
                    {delta.map(|d| view! {
                        <span class={if d >= 0.0 { "kpi-delta up" } else { "kpi-delta down" }}>
                            {format!("{}{:.1}", if d >= 0.0 { "+" } else { "" }, d)}
                        </span>
                    })}
                }).unwrap_or_else(|| view! {
                    <span class="kpi-value muted">"n/a"</span>
                    <span></span>
                })}
            </div>
            <div class="kpi">
                <span class="kpi-label">"alerts"</span>
                <span class={if alerts > 0 { "kpi-value warn" } else { "kpi-value" }}>
                    {alerts}
                </span>
            </div>
            <div class="kpi">
                <span class="kpi-label">"heartbeats ok"</span>
                <span class="kpi-value">{format!("{}/{}", heartbeats_ok, heartbeats_total)}</span>
            </div>
        </section>

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
                                view! {
                                    <tr>
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct DashboardData {
    crons: Vec<CronJobSnapshot>,
    handovers: Vec<HandoverEntry>,
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
