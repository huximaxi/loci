use leptos::*;
use leptos_router::*;

mod dialog;
mod models;
mod tauri_bindings;
mod views;

use models::ActivePalace;
use views::about::About;
use views::dashboard::Dashboard;
use views::greeter::{has_seen_greeter, Greeter};
use views::onboarding::Onboarding;
use views::settings::Settings;
use views::setup_create::SetupCreate;
use views::setup_load::SetupLoad;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App /> });
}

#[component]
fn App() -> impl IntoView {
    provide_context(create_rw_signal(ActivePalace::default()));

    // Global greeter visibility. Initialised from first-run check; also set by
    // setup_create and setup_load when entering a new palace for the first time.
    let show_greeter: RwSignal<bool> = create_rw_signal(!has_seen_greeter());
    provide_context(show_greeter);

    view! {
        <Show when=move || show_greeter.get() fallback=|| ()>
            <Greeter show=show_greeter.write_only() />
        </Show>
        <Router>
            <Routes>
                <Route path="/" view=Onboarding />
                <Route path="/load" view=SetupLoad />
                <Route path="/create" view=SetupCreate />
                <Route path="/about" view=About />
                <Route path="/settings" view=Settings />
                <Route path="/dashboard" view=Dashboard />
            </Routes>
        </Router>
    }
}
