use topcoat::{
    view::{component, view},
    Result,
};

#[component]
pub async fn empty(#[default] tagline: Option<&str>) -> Result {
    view! {
        <div
            class="flex flex-col items-center text-center gap-4 py-12"
        >
            <p class="text-base text-muted-foreground">
                (tagline.unwrap_or("Nothing here yet, I'm still fishing for ideas."))
            </p>
        </div>
    }
}
