use crate::dialog::open_directory;
use crate::models::{ActivePalace, PalaceManifest};
use crate::tauri_bindings::invoke;
use leptos::*;
use leptos_router::{use_navigate, A};
use serde::Serialize;

#[derive(Serialize)]
struct PathArg<'a> {
    path: &'a str,
}

#[component]
pub fn SetupLoad() -> impl IntoView {
    let active = expect_context::<RwSignal<ActivePalace>>();
    let (status, set_status) = create_signal::<LoadStatus>(LoadStatus::Idle);
    let navigate = use_navigate();

    let pick_palace: Callback<()> = Callback::new({
        let navigate = navigate.clone();
        move |_: ()| {
            let navigate = navigate.clone();
            spawn_local(async move {
                set_status.set(LoadStatus::Picking);
                let picked = match open_directory(
                    "Open palace. Choose the directory that holds CLAUDE.md.",
                )
                .await
                {
                    Ok(Some(p)) => p,
                    Ok(None) => {
                        set_status.set(LoadStatus::Idle);
                        return;
                    }
                    Err(e) => {
                        set_status.set(LoadStatus::Error(e));
                        return;
                    }
                };

                set_status.set(LoadStatus::Validating);
                let valid: bool =
                    match invoke("validate_palace_path", &PathArg { path: &picked }).await {
                        Ok(v) => v,
                        Err(e) => {
                            set_status.set(LoadStatus::Error(format!("validate failed: {e}")));
                            return;
                        }
                    };
                if !valid {
                    set_status.set(LoadStatus::NotAPalace(picked));
                    return;
                }

                let manifest: PalaceManifest =
                    match invoke("load_palace", &PathArg { path: &picked }).await {
                        Ok(m) => m,
                        Err(e) => {
                            set_status.set(LoadStatus::Error(format!("load failed: {e}")));
                            return;
                        }
                    };

                active.set(ActivePalace {
                    path: Some(picked.clone()),
                    manifest: Some(manifest),
                });
                set_status.set(LoadStatus::Loaded);
                navigate("/dashboard", Default::default());
            });
        }
    });

    view! {
        <main class="wizard">
            <header class="wizard-header">
                <A href="/" attr:class="back">"< back"</A>
                <h2>"Open a palace I keep"</h2>
            </header>

            <section class="wizard-body">
                {move || match status.get() {
                    LoadStatus::Idle => view! {
                        <div>
                            <p>"Pick the directory whose name you would write if asked where your palace lives."</p>
                            <button class="primary" on:click=move |_| pick_palace.call(())>"Choose directory"</button>
                        </div>
                    }.into_view(),
                    LoadStatus::Picking => view! { <p class="muted">"Waiting for your choice…"</p> }.into_view(),
                    LoadStatus::Validating => view! { <p class="muted">"Checking the shape…"</p> }.into_view(),
                    LoadStatus::NotAPalace(p) => view! {
                        <div>
                            <p class="warn">"That directory does not yet have a palace shape."</p>
                            <p class="muted"><code>{p}</code></p>
                            <p>"A palace needs "<code>"CLAUDE.md"</code>" at the root and a "<code>"_palace/"</code>" directory inside."</p>
                            <button class="primary" on:click=move |_| pick_palace.call(())>"Try another"</button>
                            <a href="/create" class="secondary">"Start a fresh one instead"</a>
                        </div>
                    }.into_view(),
                    LoadStatus::Loaded => view! { <p>"Loaded. Opening…"</p> }.into_view(),
                    LoadStatus::Error(e) => view! {
                        <div>
                            <p class="warn">"Something is off."</p>
                            <p class="muted">{e}</p>
                            <button class="primary" on:click=move |_| pick_palace.call(())>"Try again"</button>
                        </div>
                    }.into_view(),
                }}
            </section>
        </main>
    }
}

#[derive(Clone, Debug)]
enum LoadStatus {
    Idle,
    Picking,
    Validating,
    NotAPalace(String),
    Loaded,
    Error(String),
}
