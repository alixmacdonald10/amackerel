use topcoat::{router::page, view::view, Result};

use super::{NOT_FOUND_IMG, NO_PROJECTS_IMG};
use crate::projects::{load_projects, ProjectMeta};

/// The homepage. The GitHub fetch is awaited directly in the component — no
/// server function, no resource, no `<Suspense>`, because rendering happens on
/// the server that owns the cache.
#[page("/")]
async fn home() -> Result {
    let projects = load_projects().await;

    view! {
        <section class="bio">
            <p class="text-lg leading-relaxed text-[var(--muted)] m-0">
                "Trawl through the shoal of projects I've been tinkering with "
                "or learn more "<a href="/about">"about me"</a>"."
            </p>
        </section>
        <h2 class="section-title">"Projects"</h2>
        match projects {
            Ok(list) if list.is_empty() => <section class="flex flex-col items-center text-center gap-4 py-12">
                <img src=(NO_PROJECTS_IMG) alt="No projects yet" class="w-full max-w-[420px]">
                <p class="text-lg text-[var(--muted)] m-0">
                    "Nothing here yet, I'm still fishing for ideas."
                </p>
            </section>,
            Ok(list) => <ul class="post-list">
                for ProjectMeta { name, description, languages, stars, url, .. } in list {
                    <li class="post-card">
                        <a class="card-link" href=(url) target="_blank" rel="noopener">
                            <h3>(name)</h3>
                            if !languages.is_empty() || stars > 0 {
                                <p class="card-meta">
                                    for language in &languages {
                                        <span class="lang">(language.as_str())</span>
                                    }
                                    if stars > 0 {
                                        <span class="stars">("★ ")(stars)</span>
                                    }
                                </p>
                            }
                            <p class="excerpt">(description)</p>
                        </a>
                    </li>
                }
            </ul>,
            Err(_) => <section class="flex flex-col items-center text-center gap-4 py-12">
                <img src=(NOT_FOUND_IMG) alt="Failed to load projects" class="w-full max-w-[420px]">
                <p class="text-lg text-[var(--muted)] m-0">
                    "Couldn't reel in the projects — try again later."
                </p>
            </section>,
        }
    }
}
