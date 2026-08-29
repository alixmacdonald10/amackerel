use topcoat::{router::page, view::view, Result};

use crate::app::{GITHUB_URL, HIGHLIGHT_CSS, INLINE_NAV_LINK_CSS, PARAGRAPH_CSS};

#[page("/about")]
async fn about() -> Result {
    view! {
        <article class="flex flex-col text-left max-w-[720px] font-mono mb-8">
            <h1 class="text-4xl font-bold mb-2">"I'm Alix"</h1>

            <p class="mb-8 text-sm text-[var(--color-muted)]">
                "Rust - Python - Kubernetes - Postgres"
            </p>

            <p class="font-bold pb-4">
                <span class=(HIGHLIGHT_CSS)>"Senior software engineer"</span>
                " specialising in backend and infrastructure. "
                "Ex-Chartered Mechanical Engineer who decided to cast the net a little wider."
            </p>

            <p class=(PARAGRAPH_CSS)>
                "My guiding principle is "
                <span class=(HIGHLIGHT_CSS)>"KISS"</span>
                " - Keep It Simple, Stupid. As developers we love making things harder "
                "than they need to be. I spend my time hunting down complexity and "
                "removing it. Simple systems are easier to reason about and the things "
                "you can reason about are the things you can actually work with."
            </p>

            <p class=(PARAGRAPH_CSS)>
                "I love "
                <span class=(HIGHLIGHT_CSS)>"Rust"</span>
                " and have used it professionally since "
                <span>"2023"</span>
                ". "
                "I also have a soft spot for "
                <span class=(HIGHLIGHT_CSS)>"Postgres"</span>
                " and "
                <span class=(HIGHLIGHT_CSS)>"Kubernetes"</span>
                ", and I "
                "find "
                <span class=(HIGHLIGHT_CSS)>"system design"</span>
                " genuinely rewarding. I'm perfectly "
                "good with "
                <span class=(HIGHLIGHT_CSS)>"Python"</span>
                " too, I just miss Rust's compiler."
            </p>

            <p class=(PARAGRAPH_CSS)>
                "Before all this I picked up two Master's degrees, one in "
                <span class=(HIGHLIGHT_CSS)>"Mechanical Engineering"</span>
                ", the other in "
                <span class=(HIGHLIGHT_CSS)>"Astronautics & Space Engineering"</span>
                ". I had a solid career here, hence the \"ex-Chartered Mechanical Engineer\", "
                "but i was reeled in by the infinite scope of software engineering, and havn't looked back!"
            </p>

            <p class=(PARAGRAPH_CSS)>
                "Have a look through my "
                <a
                    class=(INLINE_NAV_LINK_CSS)
                    href=(GITHUB_URL)
                    target="_blank"
                    rel="noopener"
                >
                    "GitHub"
                </a>
                " to see more of what I'm all about."
            </p>
        </article>
    }
}
