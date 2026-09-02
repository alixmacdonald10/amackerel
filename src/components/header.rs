use topcoat::{
    view::{class, component, view},
    Result,
};

use crate::{
    app::{LOGO, NAV_LINK_CSS},
    components::separator::{separator, SeparatorOrientation},
    utils::io::github::GITHUB_URL,
};

const LOGO_WIDTH: &str = "max-w-[320px]";

#[component]
pub async fn header() -> Result {
    let url = GITHUB_URL.as_str();
    view! {
        <header
            class=(class!("w-full", "flex", "flex-col", "items-center", "px-5", "py-4", "gap-2", "mb-8"))
        >
            <a href="/" class=(class!("mb-4"))>
                <img src=(LOGO) alt="A Macdonald" class=(class!("w-full", LOGO_WIDTH))>
            </a>
            <p class=(class!("text-sm", "text-muted-foreground"))>
                "Hooked on keeping it simple"
            </p>
            <p class=(class!("text-xs", "text-muted-foreground", "italic"))>
                "Beware of fish related puns"
            </p>
            <nav
                class=(class!("flex", "justify-center", "items-center", "gap-8", "text-base", "text-muted-foreground", "mt-4"))
            >
                <a class=(NAV_LINK_CSS) href="/"><span>"Projects"</span></a>
                <a class=(NAV_LINK_CSS) href="/about"><span>"About"</span></a>
                <a
                    class=(NAV_LINK_CSS)
                    href=(url)
                    target="_blank"
                    rel="noopener"
                >
                    <span>"GitHub"</span>
                </a>
            </nav>
            separator(orientation: SeparatorOrientation::Horizontal)
        </header>
    }
}
