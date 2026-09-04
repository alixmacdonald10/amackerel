use reqwest::StatusCode;
use topcoat::{
    view::{component, view},
    Result,
};

use crate::app::INLINE_NAV_LINK_CSS;

#[component]
pub async fn not_found() -> Result {
    view! {
        (StatusCode::NOT_FOUND)
        <section class="flex flex-col items-center text-center gap-4 py-12">
            <p class="text-lg text-muted-foreground">
                "This page swam away."
            </p>
            <a class=(INLINE_NAV_LINK_CSS) href="/">"Back to shore"</a>
        </section>
    }
}
