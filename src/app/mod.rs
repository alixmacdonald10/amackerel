//! Site shell: the one layout every page is wrapped in, plus the shared assets.

mod about;
mod home;

use topcoat::{
    asset::{asset, Asset},
    context::Cx,
    router::{
        error::{not_found, NotFoundError},
        layout, page, uri, StatusCode,
    },
    tailwind,
    view::view,
    Result,
};

pub const LOGO: Asset = asset!("public/favicon-light.png");
pub const NOT_FOUND_IMG: Asset = asset!("public/404.png");
pub const NO_PROJECTS_IMG: Asset = asset!("public/no-posts.png");

/// Wraps every page
///
/// Document shell, nav, footer, and the branded 404 that a page's `NotFoundError` renders into.
#[layout("/")]
async fn root_layout(cx: &Cx, slot: Result) -> Result {
    // A layout sees the inner page's error before it becomes a response, so
    // catching NotFoundError here replaces the router's default 404 body.
    let content = match slot {
        Err(error) if error.downcast_ref::<NotFoundError>().is_some() => view! {
            (StatusCode::NOT_FOUND)
            <section class="flex flex-col items-center text-center gap-4 py-12">
                <img
                    src=(NOT_FOUND_IMG)
                    alt="404 — page not found"
                    class="w-full max-w-[420px]"
                >
                <h1 class="text-3xl font-bold m-0">"404"</h1>
                <p class="text-lg text-[var(--muted)] m-0">
                    "This page swam away."
                </p>
                <a href="/">"Back to shore"</a>
            </section>
        },
        content => content,
    }?;

    let version = format!("v{}", env!("CARGO_PKG_VERSION"));

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <title>(title_for(uri(cx).path()))</title>
                <link rel="icon" type="image/png" href=(LOGO)>
                <link rel="stylesheet" href=(tailwind::stylesheet!())>
                topcoat::dev::script()
            </head>
            <body>
                <header class="max-w-[720px] mx-auto px-5 py-4 flex flex-col items-center text-center gap-2 border-b border-[var(--border)]">
                    <a href="/">
                        <img src=(LOGO) alt="A Macdonald" class="w-full max-w-[320px]">
                    </a>
                    <p class="text-sm text-[var(--muted)] m-0">
                        "Hooked on keeping it simple"
                    </p>
                    <p class="text-xs text-[var(--muted)] m-0 italic">
                        "Beware of fish related puns"
                    </p>
                    <nav class="site-nav flex gap-6 text-[0.95rem]">
                        <a href="/"><span>"Projects"</span></a>
                        <a href="/about"><span>"About"</span></a>
                        <a href="https://github.com/alixmacdonald10" target="_blank" rel="noopener"><span>"GitHub"</span></a>
                    </nav>
                </header>
                <main>(content)</main>
                <footer class="fixed bottom-3 right-4 z-50">
                    <p class="text-xs font-mono tracking-tight text-[var(--muted)] opacity-60 hover:opacity-100 transition-opacity m-0">
                        (version)
                    </p>
                </footer>
            </body>
        </html>
    }
}

/// Catches every unmatched path so the branded 404 renders inside the layout.
///
/// The router's own not-found response bypasses layouts entirely, so a page has
/// to raise the error for `root_layout` to catch it.
#[page("/{*path}")]
async fn unmatched() -> Result {
    Err(not_found().into())
}

fn title_for(path: &str) -> &'static str {
    match path {
        "/" => "A Macdonald — Projects",
        "/about" => "A Macdonald — About",
        _ => "A Macdonald — 404",
    }
}
