use topcoat::{
    asset::{asset, Asset},
    view::{component, view},
    Result,
};

use crate::app::MED_IMAGE_CSS;

pub const NO_DATA_IMG: Asset = asset!("public/no-posts.png");

#[component]
pub async fn empty(#[default] tagline: Option<&str>) -> Result {
    view! {
        <div
            class="flex flex-col items-center text-center gap-4 py-12"
        >
            <img
                src=(NO_DATA_IMG)
                alt="No data yet"
                class=(MED_IMAGE_CSS)
            >
            <p class="text-base text-muted-foreground">
                (tagline.unwrap_or("Nothing here yet, I'm still fishing for ideas."))
            </p>
        </div>
    }
}
