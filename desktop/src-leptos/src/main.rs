use leptos::*;
use leptos_router::*;

mod dialog;
mod models;
mod tauri_bindings;
mod views;

use models::ActivePalace;
use views::about::About;
use views::dashboard::Dashboard;
use views::onboarding::Onboarding;
use views::setup_create::SetupCreate;
use views::setup_load::SetupLoad;

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App /> });
}

#[component]
fn App() -> impl IntoView {
    // Application state: which palace is active. Phase 2 sets it via load/scaffold;
    // Phase 3 dashboard + Phase 4 map consume it.
    provide_context(create_rw_signal(ActivePalace::default()));

    view! {
        <Router>
            <Routes>
                <Route path="/" view=Onboarding />
                <Route path="/load" view=SetupLoad />
                <Route path="/create" view=SetupCreate />
                <Route path="/about" view=About />
                <Route path="/dashboard" view=Dashboard />
            </Routes>
        </Router>
    }
}
