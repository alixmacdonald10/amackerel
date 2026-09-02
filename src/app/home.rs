use std::sync::Arc;

use topcoat::{
    context::{app_context, Cx},
    icon::{icon, iconify::iconify_icon},
    router::page,
    runtime::shard,
    view::{attributes, class, view},
    Result,
};

use crate::{
    app::{INLINE_NAV_LINK_CSS, PARAGRAPH_CSS},
    cache::{generate_cache_key, TTLCache},
    components::{
        alert::{alert, alert_description, alert_title, AlertVariant},
        card::{card, card_description, card_footer, card_header, card_title},
        empty::empty,
    },
    config::AppConfig,
    projects::{load_projects, RepositoryMeta},
    utils::io::github::CURATED_REPOS,
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
    let key = generate_cache_key("load_projects", CURATED_REPOS.join(":").as_str());
    let projects = if let Some(value) = cache.get::<Vec<RepositoryMeta>>(&key) {
        Ok(value.to_vec())
    } else {
        let client = app_context::<reqwest::Client>(cx);
        let app_config = app_context::<AppConfig>(cx);
        load_projects(app_config, client, CURATED_REPOS).await
    };

    match projects {
        Ok(list) if list.is_empty() => view! { empty() },
        Ok(list) => {
            let elapsed = now.elapsed();
            tracing::info!("Func call time elapsed: {elapsed:#?}");

            // TODO: This is silly and always updates the cache
            let _ = cache.insert(&key, list.clone());

            view! { <ul>
                for RepositoryMeta { name, description, languages, stars, url, .. } in list {
                    <li>
                        <a href=(url) target="_blank" rel="noopener">
                        card(
                             attrs: attributes! { class=(class!(
                                 "group m-4 overflow-hidden",
                                 "transition duration-300 ease-in-out",
                                 "hover:border-ring hover:-translate-y-1 hover:scale-105 hover:shadow-sm",
                             )) },
                             card_header(
                                 card_title(
                                     attrs: attributes! { class="text-xl font-bold transition-colors group-hover:text-primary" },
                                     <span class="mr-2">(name)</span>
                                     if stars > 0 {
                                         <span class="text-sm font-normal text-muted-foreground/75">
                                             ("★ ")
                                             (stars)
                                         </span>
                                     }
                                 )
                                 card_description(
                                     attrs: attributes! { class="text-base" },
                                     (description)
                                 )
                             )
                             if languages.is_some() {
                                 card_footer(
                                     attrs: attributes! { class="flex-wrap justify-center gap-2" },
                                     for language in &languages.unwrap_or(vec![]) {
                                         <span
                                             class="rounded-xl px-2 py-1 text-sm text-muted-foreground bg-muted-foreground/10"
                                         >
                                             (language.as_str())
                                         </span>
                                     }
                                 )
                             }
                        )
                        </a>
                    </li>
                }
            </ul> }
        }
        Err(err) => {
            tracing::error!("{}", err.to_string());
            view! {
                alert(
                     variant: AlertVariant::Destructive,
                     icon(data: iconify_icon!("hugeicons:fish-off"))
                     alert_title("Failed to fetch data")
                     alert_description((err.to_string()))

                )
            }
        }
    }
}
