use topcoat::{
    view::{component, view},
    Result,
};

#[component]
pub async fn footer() -> Result {
    let version = format!("v{}", env!("CARGO_PKG_VERSION"));

    view! {
        <footer class="fixed bottom-3 right-4 z-50">
            <p
                class="text-xs tracking-tight text-foreground hover:text-primary"
            >
                (version)
            </p>
        </footer>
    }
}
