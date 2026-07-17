use leptos::prelude::*;
use leptos_meta::{provide_meta_context, HashedStylesheet, MetaTags, Title};
use leptos_router::{
    components::{Route, Router, Routes, A},
    hooks::use_params_map,
    ParamSegment, StaticSegment,
};

use crate::blog::get_post;
use crate::projects::list_projects;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <link rel="icon" type="image/png" href="/favicon-light.png"/>
                <AutoReload options=options.clone() />
                <HashedStylesheet options=options.clone() id="leptos"/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="A Macdonald — Projects"/>

        <Router>
            <Nav/>
            <main>
                <Routes fallback=NotFound transition=true>
                    <Route path=StaticSegment("") view=HomePage/>
                    <Route path=(StaticSegment("posts"), ParamSegment("slug")) view=PostPage/>
                    <Route path=StaticSegment("about") view=AboutPage/>
                </Routes>
            </main>
            <Footer/>
        </Router>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <Title text="A Macdonald — 404"/>
        <section class="flex flex-col items-center text-center gap-4 py-12">
            <img
                src="/404.png"
                alt="404 — page not found"
                class="w-full max-w-[420px]"
            />
            <h1 class="text-3xl font-bold m-0">"404"</h1>
            <p class="text-lg text-[var(--muted)] m-0">
                "This page swam away."
            </p>
            <A href="/">"Back to shore"</A>
        </section>
    }
}

#[component]
fn Nav() -> impl IntoView {
    view! {
        <header class="max-w-[720px] mx-auto px-5 py-4 flex flex-col items-center text-center gap-2 border-b border-[var(--border)]">
            <A href="/">
                <img
                    src="/favicon-light.png"
                    alt="A Macdonald"
                    class="w-full max-w-[320px]"
                />
            </A>
            <p class="text-sm text-[var(--muted)] m-0">
                "Hooked on keeping it simple"
            </p>
            <p class="text-xs text-[var(--muted)] m-0 italic">
                "Beware of fish related puns"
            </p>            <nav class="flex gap-6 text-[0.95rem] [&_a]:text-[var(--muted)] [&_a]:no-underline [&_a:hover]:text-[var(--fg)]">
                <A href="/">"Projects"</A>
                <A href="/about">"About"</A>
                <a href="https://github.com/alixmacdonald10" target="_blank" rel="noopener">"GitHub"</a>
            </nav>
        </header>
    }
}

#[component]
fn Footer() -> impl IntoView {
    let version_text = format!("v{}", env!("CARGO_PKG_VERSION"));

    view! {
        <footer class="fixed bottom-3 right-4 z-50">
            <p class="text-xs font-mono tracking-tight text-[var(--muted)] opacity-60 hover:opacity-100 transition-opacity m-0">
                {version_text}
            </p>
        </footer>
    }
}
#[component]
fn HomePage() -> impl IntoView {
    let projects = Resource::new(|| (), |_| async move { list_projects().await });

    view! {
        <section class="bio">
            <p class="text-lg leading-relaxed text-[var(--muted)] m-0">
                "Trawl through the shoal of projects I've been tinkering with "
                "or learn more "<A href="/about">"about me"</A>"."
            </p>
        </section>
        <h2 class="section-title">"Projects"</h2>
        <Suspense fallback=move || view! { <p class="notice">"Loading projects…"</p> }>
            {move || projects.get().map(|res| match res {
                Ok(list) if list.is_empty() => view! {
                    <section class="flex flex-col items-center text-center gap-4 py-12">
                        <img
                            src="/no-posts.png"
                            alt="No projects yet"
                            class="w-full max-w-[420px]"
                        />
                        <p class="text-lg text-[var(--muted)] m-0">
                            "Nothing here yet, I'm still fishing for ideas."
                        </p>
                    </section>
                }.into_any(),
                Ok(list) => view! {
                    <ul class="post-list">
                        {list.into_iter().map(|m| {
                            let languages = m.languages.iter().map(|l| view! {
                                <span class="lang">{l.clone()}</span>
                            }).collect_view();
                            let stars = (m.stars > 0).then(|| view! {
                                <span class="stars">{format!("★ {}", m.stars)}</span>
                            });
                            let has_meta = !m.languages.is_empty() || m.stars > 0;
                            view! {
                                <li class="post-card">
                                    <a class="card-link" href=m.url target="_blank" rel="noopener">
                                        <h3>{m.name}</h3>
                                        {has_meta.then(|| view! {
                                            <p class="card-meta">{languages}{stars}</p>
                                        })}
                                        <p class="excerpt">{m.description}</p>
                                    </a>
                                </li>
                            }
                        }).collect_view()}
                    </ul>
                }.into_any(),
                Err(_) => view! {
                    <section class="flex flex-col items-center text-center gap-4 py-12">
                        <img
                            src="/404.png"
                            alt="Failed to load projects"
                            class="w-full max-w-[420px]"
                        />
                        <p class="text-lg text-[var(--muted)] m-0">
                            "Couldn't reel in the projects — try again later."
                        </p>
                    </section>
                }.into_any(),
            })}
        </Suspense>
    }
}

#[component]
fn PostPage() -> impl IntoView {
    let params = use_params_map();
    let post = Resource::new(
        move || params.read().get("slug").unwrap_or_default(),
        |slug| async move { get_post(slug).await },
    );

    view! {
        <Suspense fallback=move || view! { <p class="notice">"Loading…"</p> }>
            {move || post.get().map(|res| match res {
                Ok(Some(p)) => view! {
                    <article class="post">
                        <h1>{p.meta.title}</h1>
                        <p class="date">{p.meta.date}</p>
                        <div class="post-body" inner_html=p.html></div>
                    </article>
                }.into_any(),
                Ok(None) => view! { <p class="notice">"Post not found."</p> }.into_any(),
                Err(_) => view! { <p class="notice">"Failed to load post."</p> }.into_any(),
            })}
        </Suspense>
    }
}

#[component]
fn AboutPage() -> impl IntoView {
    view! {
        <article class="post about">
            <h1 class="text-4xl font-bold mb-4">"I'm Alix"</h1>

            <p class="about-lead">
                "A "<span class="hl">"senior software engineer"</span>" specialising in backend and "
                "infrastructure, and an ex-Chartered Mechanical Engineer who decided to cast his net a little wider."
            </p>

            <p>
                "Throughout my career I've picked up two Master's degrees, one in "<span class="hl">"Mechanical Engineering"</span>
                " and the other in "<span class="hl">"Astronautics & Space Engineering"</span>" (pretty cool I know)."
            </p>

            <p>
                "I have one guiding principle... "<span class="hl">"Keep It Simple Stupid (KISS)"</span>"."
            </p>

            <p>
                "As Developers we love making things harder than they are. So I spend "
                "my time reasoning about problems, hunting complexity down and removing it."
            </p>

            <p>
                "I'm big on "<span class="hl">"security"</span>". Simple systems are easier to "
                "reason about, and things you can reason about are things you can actually secure. "
            </p>

            <p>
                "I love "<span class="hl">"Rust"</span>" and have used it professionally since "<span class="hl">"2023"</span>". "
                "I also have a soft spot for "<span class="hl">"Postgres"</span>" and "<span class="hl">"Kubernetes"</span>", and I "
                "find "<span class="hl">"system design"</span>" genuinely rewarding. I'm perfectly "
                "good with "<span class="hl">"Python"</span>" too, I just miss Rust's compiler."
            </p>

            <p>
                "Have a look through my "
                <a href="https://github.com/alixmacdonald10" target="_blank" rel="noopener">"GitHub"</a>
                " to see more of what I'm all about."
            </p>
        </article>
    }
}
