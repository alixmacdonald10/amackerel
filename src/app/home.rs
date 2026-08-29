use std::sync::Arc;

use topcoat::{
    context::{app_context, Cx},
    router::page,
    runtime::shard,
    view::view,
    Result,
};

use crate::{
    app::{INLINE_NAV_LINK_CSS, MED_IMAGE_CSS, NOT_FOUND_IMG, NO_PROJECTS_IMG, PARAGRAPH_CSS},
    cache::TTLCache,
    projects::{load_projects, RepositoryMeta},
};

/// The homepage. The GitHub fetch is awaited directly in the component — no
/// server function, no resource, no `<Suspense>`, because rendering happens on
/// the server that owns the cache.
#[page("/")]
async fn home() -> Result {
    view! {
        signal toggle_project_reload = true;

        <p class=(PARAGRAPH_CSS)>
            "Trawl through the shoal of projects I've been tinkering with "
            "or learn more "
            <a class=(INLINE_NAV_LINK_CSS) href="/about">"about me"</a>
            "."
        </p>
        <div>
            project_results(reload: $(toggle_project_reload.get()))
        </div>
    }
}

#[shard]
async fn project_results(cx: &Cx, reload: bool) -> Result {
    tracing::info!("Refreshing projects");
    let now = std::time::Instant::now();

    // TODO: a better way to impl this caching
    let cache = app_context::<Arc<TTLCache>>(cx);
    let key = "load_projects";
    let projects = if let Some(value) = cache.get::<Vec<RepositoryMeta>>(key) {
        Ok(value.to_vec())
    } else {
        load_projects(cx).await
    };

    match projects {
        Ok(list) if list.is_empty() => view! { <div
            class="flex flex-col items-center text-center gap-4 py-12"
        >
            <img
                src=(NO_PROJECTS_IMG)
                alt="No projects yet"
                class=(MED_IMAGE_CSS)
            >
            <p class="text-lg text-[var(--color-muted)] m-0">
                "Nothing here yet, I'm still fishing for ideas."
            </p>
        </div> },
        Ok(list) => {
            let elapsed = now.elapsed();
            tracing::info!("Func call time elapsed: {elapsed:#?}");

            // TODO: This is silly and always updates the cache
            let _ = cache.insert(key, list.clone());

            view! { <ul>
                for RepositoryMeta { name, description, languages, stars, url, .. } in list {
                    <li>
                        <div
                            class="max-w-[720px] rounded-lg overflow-hidden shadow-lg m-4 p-4 hover:border hover:border-[var(--color-accent)] transition duration-300 ease-in-out hover:-translate-y-1 hover:scale-105 hover:shadow-xl"
                        >
                            <a href=(url) target="_blank" rel="noopener">
                                <div class="font-bold text-xl mb-2">
                                    <span class="mr-2">(name)</span>
                                    if stars > 0 {
                                        <span class="text-sm text-[var(--color-muted)]/75">
                                            ("★ ")
                                            (stars)
                                        </span>
                                    }
                                </div>
                                <p class="text-base mb-4">(description)</p>
                                if languages.is_some() {
                                    <div class="text-[var(--color-muted)] mb-2">
                                        for language in &languages.unwrap_or(vec![]) {
                                            <span
                                                class="rounded-xl p-2 mr-2 text-sm text-[var(--color-muted)] bg-[var(--color-muted)]/10"
                                            >
                                                (language.as_str())
                                            </span>
                                        }
                                    </div>
                                }
                            </a>
                        </div>
                    </li>
                }
            </ul> }
        }
        Err(err) => {
            tracing::error!("{}", err.to_string());
            view! {
                <div
                        class="flex flex-col items-center text-center gap-4 py-12"
                    >
                        <img
                            src=(NOT_FOUND_IMG)
                            alt="Failed to load projects"
                            class=(MED_IMAGE_CSS)
                        >
                        <p class="text-lg text-[var(--color-muted)] m-0">
                            "An error has occured. Couldn't reel in the projects — try again later."
                        </p>
                        // TODO: ADD A POPUP ALERT
                        <div>(err.to_string())</div>
                    </div>
            }
        }
    }
}
