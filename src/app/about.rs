use topcoat::{router::page, view::view, Result};

#[page("/about")]
async fn about() -> Result {
    view! {
        <article class="post about">
            <h1 class="text-4xl font-bold mb-4">"I'm Alix"</h1>

            <p class="about-lead">
                "A "
                <span class="hl">"senior software engineer"</span>
                " specialising in backend and "
                "infrastructure, and an ex-Chartered Mechanical Engineer who decided to cast his net a little wider."
            </p>

            <p>
                "Throughout my career I've picked up two Master's degrees, one in "
                <span class="hl">"Mechanical Engineering"</span>
                " and the other in "
                <span class="hl">"Astronautics & Space Engineering"</span>
                " (pretty cool I know)."
            </p>

            <p>
                "I have one guiding principle... "
                <span class="hl">"Keep It Simple Stupid (KISS)"</span>
                "."
            </p>

            <p>
                "As Developers we love making things harder than they are. So I spend "
                "my time reasoning about problems, hunting complexity down and removing it."
            </p>

            <p>
                "I'm big on "
                <span class="hl">"security"</span>
                ". Simple systems are easier to "
                "reason about, and things you can reason about are things you can actually secure. "
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
