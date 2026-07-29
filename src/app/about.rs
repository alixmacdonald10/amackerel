use topcoat::{router::page, view::view, Result};

#[page("/about")]
async fn about() -> Result {
    view! {
        <article class="post about">
            <h1 class="text-4xl font-bold mb-4">"I'm Alix"</h1>

            <p class="about-stack">
                "Rust - Python - Kubernetes - Postgres"
            </p>

            <p class="about-lead">
                <span class="hl">"Senior software engineer"</span>
                " specialising in backend and infrastructure. "
                "Ex-Chartered Mechanical Engineer who decided to cast the net a little wider."
            </p>



            <p>
                "My guiding principle is "
                <span class="hl">"KISS"</span>
                "- Keep It Simple, Stupid. As developers we love making things harder "
                "than they need to be. I spend my time hunting down complexity and "
                "removing it. Simple systems are easier to reason about and the things "
                "you can reason about are the things you can actually work with."
            </p>

            <p>
                "I love "
                <span class="hl">"Rust"</span>
                " and have used it professionally since "
                <span class="hl">"2023"</span>
                ". "
                "I also have a soft spot for "
                <span class="hl">"Postgres"</span>
                " and "
                <span class="hl">"Kubernetes"</span>
                ", and I "
                "find "
                <span class="hl">"system design"</span>
                " genuinely rewarding. I'm perfectly "
                "good with "
                <span class="hl">"Python"</span>
                " too, I just miss Rust's compiler."
            </p>

            <p>
                "Before all this I picked up two Master's degrees, one in "
                <span class="hl">"Mechanical Engineering"</span>
                ", the other in "
                <span class="hl">"Astronautics & Space Engineering"</span>
                ". I had a solid career here, hence the \"ex-Chartered Mechanical Engineer\", "
                "but i was reeled in by the infinite scope of software engineering, and havn't looked back!"
            </p>

            <p>
                "Have a look through my "
                <a
                    href="https://github.com/alixmacdonald10"
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
