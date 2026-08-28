use topcoat::{router::page, view::view, Result};

use super::{NOT_FOUND_IMG, NO_PROJECTS_IMG};
use crate::{
    app::{INLINE_NAV_LINK_CSS, MED_IMAGE_CSS, PARAGRAPH_CSS},
    projects::{load_projects, ProjectMeta},
};

/// The homepage. The GitHub fetch is awaited directly in the component — no
/// server function, no resource, no `<Suspense>`, because rendering happens on
/// the server that owns the cache.
#[page("/")]
async fn home() -> Result {
    let projects = load_projects().await;

    view! {
        <p class=(PARAGRAPH_CSS)>
            "Trawl through the shoal of projects I've been tinkering with "
            "or learn more "
            <a class=(INLINE_NAV_LINK_CSS) href="/about">"about me"</a>
            "."
        </p>
        <div>
            match projects {
                Ok(list) if list.is_empty() => <div
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
                </div>,
                Ok(list) => <ul>
                    for ProjectMeta { name, description, languages, stars, url, .. } in list {
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
                                    if !languages.is_empty() {
                                        <div class="text-[var(--color-muted)] mb-2">
                                            for language in &languages {
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
                </ul>,
                Err(_) => <div
                    class="flex flex-col items-center text-center gap-4 py-12"
                >
                    <img
                        src=(NOT_FOUND_IMG)
                        alt="Failed to load projects"
                        class=(MED_IMAGE_CSS)
                    >
                    <p class="text-lg text-[var(--color-muted)] m-0">
                        "Couldn't reel in the projects — try again later."
                    </p>
                </div>,
            }
        </div>
    }
}
