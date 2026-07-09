use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes, A},
    hooks::use_params_map,
    ParamSegment, StaticSegment,
};

use crate::blog::{get_post, list_posts};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <link rel="icon" type="image/png" href="/favicon-light.png"/>
                <AutoReload options=options.clone() />
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
        <Stylesheet id="leptos" href="/pkg/amackerel.css"/>
        <Title text="A Macdonald — Blog"/>

        <Router>
            <Nav/>
            <main>
                <Routes fallback=|| view! { <p class="notice">"Page not found."</p> }>
                    <Route path=StaticSegment("") view=HomePage/>
                    <Route path=(StaticSegment("posts"), ParamSegment("slug")) view=PostPage/>
                    <Route path=StaticSegment("about") view=AboutPage/>
                </Routes>
            </main>
        </Router>
    }
}

#[component]
fn Nav() -> impl IntoView {
    view! {
        <header class="site-header">
            <A href="/">
                <span class="brand-row">
                    <img src="/favicon-light.png" alt="" class="logo"/>
                    <span class="brand">"A Macdonald"</span>
                </span>
                <span class="tagline">"Trying My Best To Keep It Stupidly Simple In A World Of Complexity"</span>
            </A>
            <nav>
                <A href="/">"Blog"</A>
                <A href="/about">"About"</A>
                <a href="https://github.com/alixmacdonald10" target="_blank" rel="noopener">"GitHub"</a>
            </nav>
        </header>
    }
}

#[component]
fn HomePage() -> impl IntoView {
    let posts = Resource::new(|| (), |_| async move { list_posts().await });

    view! {
        <section class="bio">
            <p>
                "Take a look at my musings and half-finished experiments below. "
            </p>
            <p> "Or you can always learn more "<A href="/about">"about me"</A></p>

        </section>
        <h2 class="section-title">"Posts"</h2>
        <Suspense fallback=move || view! { <p class="notice">"Loading posts…"</p> }>
            {move || posts.get().map(|res| match res {
                Ok(list) if list.is_empty() => {
                    view! { <p class="notice">"No posts yet! Check back in later."</p> }.into_any()
                }
                Ok(list) => view! {
                    <ul class="post-list">
                        {list.into_iter().map(|m| view! {
                            <li class="post-card">
                                <A href=format!("/posts/{}", m.slug)>
                                    <h3>{m.title}</h3>
                                </A>
                                <p class="date">{m.date}</p>
                                <p class="excerpt">{m.description}</p>
                            </li>
                        }).collect_view()}
                    </ul>
                }.into_any(),
                Err(_) => view! { <p class="notice">"Failed to load posts."</p> }.into_any(),
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
            <h1 class="about-lead">"I'm Alix"</h1>

            <p>
                "I have one guiding principle... "<span class="h1">"Keep It Simple Stupid (KISS)"</span>"."
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
                "I love "<span class="hl">"Rust"</span>", "
                <span class="hl">"Postgres"</span>" and "<span class="hl">"Kubernetes"</span>", and I "
                "find "<span class="hl">"system design"</span>" genuinely rewarding. I'm perfectly "
                "good with "<span class="hl">"Python"</span>" too — I just spend the whole time missing "
                "Rust's compiler."
            </p>

            <p>
                "Have a look through my "
                <a href="https://github.com/alixmacdonald10" target="_blank" rel="noopener">"GitHub"</a>
                " and "<A href="/">"blog posts"</A>
                " to see more of what I'm all about."
            </p>
        </article>
    }
}
