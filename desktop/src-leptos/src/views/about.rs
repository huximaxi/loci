use crate::views::greeter::reset_greeter;
use leptos::*;
use leptos_router::A;

#[component]
pub fn About() -> impl IntoView {
    let show_greeter = expect_context::<RwSignal<bool>>();
    let probe = move |_| {
        reset_greeter();
        show_greeter.set(true);
    };

    view! {
        <main class="about">
            <header class="wizard-header">
                <A href="/" attr:class="back">"< back"</A>
                <h2>"What is a palace?"</h2>
            </header>

            <section class="about-body">
                <p>
                    "A palace is a directory you have agreed to remember from. It holds a "
                    <code>"CLAUDE.md"</code>" at its root and a "<code>"_palace/"</code>" beside it. "
                    "Inside "<code>"_palace/"</code>" live rooms: small contexts that hold their own files and their own attention."
                </p>
                <p>
                    "Nothing about a palace is mystical. It is a convention for where your context lives. "
                    "Crystals are facts you stop re-deriving. Plants are themes you choose to water. Cron jobs are "
                    "little programs that keep the palace honest while you sleep."
                </p>
                <p>
                    "loci wizard reads a palace. It does not write to one without permission. "
                    "When you load a palace, the wizard checks its shape, opens the dashboard, and watches for changes."
                </p>
                <p class="muted">
                    "Your palace is yours. The wizard is a lens, not a landlord."
                </p>

                <div style="margin-top: 20px; padding-top: 14px; border-top: 1px solid var(--scholar-border);">
                    <p class="muted" style="font-size: 11px; margin-bottom: 6px;">"dev probe"</p>
                    <button on:click=probe>"show greeter"</button>
                </div>
            </section>
        </main>
    }
}
