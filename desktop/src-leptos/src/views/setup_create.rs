use crate::dialog::open_directory;
use crate::models::{ActivePalace, PalaceManifest};
use crate::tauri_bindings::invoke;
use leptos::*;
use leptos_router::{use_navigate, A};
use serde::Serialize;

#[derive(Serialize)]
struct ParentPathArg<'a> {
    parent_path: &'a str,
}

#[derive(Serialize)]
struct PathArg<'a> {
    path: &'a str,
}

#[component]
pub fn SetupCreate() -> impl IntoView {
    let active = expect_context::<RwSignal<ActivePalace>>();
    let (status, set_status) = create_signal::<CreateStatus>(CreateStatus::Idle);
    let navigate = use_navigate();

    let pick_and_scaffold: Callback<()> = Callback::new(move |_: ()| {
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

            set_status.set(CreateStatus::Scaffolding(parent.clone()));
            let scaffolded: String = match invoke(
                "scaffold_palace",
                &ParentPathArg {
                    parent_path: &parent,
                },
            )
            .await
            {
                Ok(p) => p,
                Err(e) => {
                    set_status.set(CreateStatus::Error(format!("scaffold failed: {e}")));
                    return;
                }
            };

            let manifest: PalaceManifest =
                match invoke("load_palace", &PathArg { path: &scaffolded }).await {
                    Ok(m) => m,
                    Err(e) => {
                        set_status.set(CreateStatus::Error(format!("load after scaffold: {e}")));
                        return;
                    }
                };

            active.set(ActivePalace {
                path: Some(scaffolded.clone()),
                manifest: Some(manifest),
            });
            set_status.set(CreateStatus::Done(scaffolded));
        });
    });

    let go_dashboard: Callback<()> = Callback::new({
        let navigate = navigate.clone();
        move |_: ()| {
            navigate("/dashboard", Default::default());
        }
    });

    view! {
        <main class="wizard">
            <header class="wizard-header">
                <A href="/" attr:class="back">"< back"</A>
                <h2>"Start a fresh palace"</h2>
            </header>

            <section class="wizard-body">
                {move || match status.get() {
                    CreateStatus::Idle => view! {
                        <div>
                            <p>"Choose a parent directory. A "<code>"_palace/"</code>" will be planted inside, along with five rooms and a top-level "<code>"CLAUDE.md"</code>"."</p>
                            <p class="muted">"Nothing existing is touched. If a "<code>"CLAUDE.md"</code>" already lives there, it stays."</p>
                            <button class="primary" on:click=move |_| pick_and_scaffold.call(())>"Choose parent directory"</button>
                        </div>
                    }.into_view(),
                    CreateStatus::Picking => view! { <p class="muted">"Waiting for your choice…"</p> }.into_view(),
                    CreateStatus::Scaffolding(p) => view! {
                        <p class="muted">"Planting palace at "<code>{p}</code>"…"</p>
                    }.into_view(),
                    CreateStatus::Done(p) => view! {
                        <div>
                            <p>"Palace planted at "<code>{p.clone()}</code>"."</p>
                            <p class="muted">"Rooms: dev-room · hatchery · design-room · engine-room · library."</p>
                            <button class="primary" on:click=move |_| go_dashboard.call(())>"Enter the palace"</button>
                        </div>
                    }.into_view(),
                    CreateStatus::Error(e) => view! {
                        <div>
                            <p class="warn">"Something is off."</p>
                            <p class="muted">{e}</p>
                            <button class="primary" on:click=move |_| pick_and_scaffold.call(())>"Try again"</button>
                        </div>
                    }.into_view(),
                }}
            </section>
        </main>
    }
}

#[derive(Clone, Debug)]
enum CreateStatus {
    Idle,
    Picking,
    Scaffolding(String),
    Done(String),
    Error(String),
}
