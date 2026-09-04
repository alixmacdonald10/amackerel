//! Site shell: the one layout every page is wrapped in, plus the shared assets.

mod about;
mod home;
mod unmatched;

use topcoat::{
    asset::{asset, Asset},
    context::Cx,
    font::fontsource::fontsource_font,
    router::{error::NotFoundError, layout, request::uri},
    tailwind,
    view::{class, view, StaticClass},
    Result,
};

use crate::components::{footer::footer, header::header};

pub const LOGO: Asset = asset!("public/favicon.svg");
pub const MAX_WIDTH: &str = "max-w-[720px]";
pub const NAV_LINK_CSS: StaticClass = class!(
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
// Unlike the header's links, these sit inside body copy that is already
// `text-foreground`, so they carry the accent at rest to read as links at all.
pub const INLINE_NAV_LINK_CSS: StaticClass = class!(
    "inline-block",
    "transition",
    "delay-50",
    "duration-200",
    "ease-in-out",
    "text-primary",
    "underline",
    "underline-offset-4",
    "decoration-primary/40",
    "hover:decoration-primary",
    "hover:-translate-y-1",
    "hover:scale-110",
    "focus-visible:outline-2",
    "focus-visible:outline-offset-2",
    "focus-visible:outline-ring"
);
pub const HIGHLIGHT_CSS: StaticClass = class!("font-bold", "text-primary");
pub const PARAGRAPH_CSS: StaticClass = class!("mt-4", "mb-4", "break-normal");
pub const MED_IMAGE_CSS: StaticClass = class!("w-full", "max-w-[420px]");

/// The base page layout
#[layout("/")]
async fn root_layout(cx: &Cx, slot: Result) -> Result {
    // A layout sees the inner page's error before it becomes a response, so
    // catching NotFoundError here replaces the router's default 404 body.
    let content = match slot {
        Err(error) if error.downcast_ref::<NotFoundError>().is_some() => {
            view! { crate::components::not_found::not_found() }
        }
        content => content,
    }?;

    view! {
        <!DOCTYPE html>
        <html lang="en" class="dark">
            <head>
                <meta charset="utf-8">
                <meta name="viewport" content="width=device-width, initial-scale=1">
                <title>(title_for(uri(cx).path()))</title>
                <link rel="icon" type="image/png" href=(LOGO)>
                topcoat::dev::script()
                topcoat::font::link(font: fontsource_font!(GEIST_MONO))
                <link rel="stylesheet" href=(tailwind::stylesheet!())>
            </head>
            <body class=(class!(MAX_WIDTH, "w-full", "mx-auto", "flex", "flex-col", "items-center", "text-center"))>
                header()
                <main class=(class!("flex", "flex-col", "items-center"))>
                    (content)
                </main>
                footer()
            </body>
        </html>
    }
}

fn title_for(path: &str) -> &'static str {
    match path {
        "/" => "A Macdonald — Projects",
        "/about" => "A Macdonald — About",
        _ => "A Macdonald — 404",
    }
}
