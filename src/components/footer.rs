use topcoat::{
    view::{class, component, view, StaticClass},
    Result,
};

const VERSION_CSS: StaticClass = class!(
    "text-xs",
    "tracking-tight",
    "text-foreground",
    "transition",
    "delay-50",
    "duration-200",
    "ease-in-out",
    "hover:text-primary",
    "hover:-translate-y-1",
    "hover:scale-110",
    "focus-visible:outline-2",
    "focus-visible:outline-offset-2",
    "focus-visible:outline-ring"
);

#[component]
pub async fn footer() -> Result {
    let version = format!("v{}", env!("CARGO_PKG_VERSION"));

    view! {
        <footer class="fixed bottom-3 right-4 z-50">
            <p class=(VERSION_CSS)>
                (version)
            </p>
        </footer>
    }
}
