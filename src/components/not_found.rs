use reqwest::StatusCode;
use topcoat::{
    asset::{asset, Asset},
    view::{component, view},
    Result,
};

use crate::app::{INLINE_NAV_LINK_CSS, MED_IMAGE_CSS};

pub const NOT_FOUND_IMG: Asset = asset!("public/404.png");

#[component]
pub async fn not_found() -> Result {
    view! {
        (StatusCode::NOT_FOUND)
        <section class="flex flex-col items-center text-center gap-4 py-12">
            <img
                src=(NOT_FOUND_IMG)
                alt="404 — page not found"
                class=(MED_IMAGE_CSS)
            >
            <p class="text-lg text-muted-foreground">
                "This page swam away."
            </p>
            <a class=(INLINE_NAV_LINK_CSS) href="/">"Back to shore"</a>
        </section>
    }
}
