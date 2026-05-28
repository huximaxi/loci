use leptos::*;
use leptos_router::A;

#[component]
pub fn Onboarding() -> impl IntoView {
    view! {
        <main class="onboarding">
            <header class="onboarding-header">
                <h1>"loci wizard"</h1>
                <p class="lede">"A palace is a place you remember from."</p>
            </header>

            <nav class="card-grid" aria-label="palace choice">
                <A href="/load" attr:class="card card-primary">
                    <span class="card-eyebrow">"open"</span>
                    <span class="card-title">"a palace I keep"</span>
                    <span class="card-body">
                        "Point at a directory with a "
                        <code>"CLAUDE.md"</code>
                        " and a "
                        <code>"_palace/"</code>
                        " inside."
                    </span>
                </A>

                <A href="/create" attr:class="card">
                    <span class="card-eyebrow">"start"</span>
                    <span class="card-title">"a palace I do not yet keep"</span>
                    <span class="card-body">
                        "Choose a parent directory. A fresh "
                        <code>"_palace/"</code>
                        " will take root there."
                    </span>
                </A>

                <A href="/about" attr:class="card card-quiet">
                    <span class="card-eyebrow">"read"</span>
                    <span class="card-title">"what is a palace?"</span>
                    <span class="card-body">"One short page, no decisions asked."</span>
                </A>
            </nav>

            <footer class="onboarding-footer">
                <em>"Not yet is not forever."</em>
            </footer>
        </main>
    }
}
