mod app;
mod projects;

use topcoat::{
    asset::{AssetBundle, RouterBuilderAssetExt},
    context::CxBuilder,
    router::{
        layer, Body, HeaderName, HeaderValue, Next, Response, Router, RouterBuilderDiscoverExt,
    },
    Result,
};

#[tokio::main]
async fn main() {
    let router = Router::builder()
        .discover()
        .assets(AssetBundle::load().unwrap())
        .build();

    topcoat::start(router).await.unwrap();
}

/// True when running under `topcoat dev`, which serves its live-reload client
/// script from a second origin.
fn dev_origin() -> Option<String> {
    std::env::var("TOPCOAT_DEV_URL").ok()
}

/// Sets security-hardening headers on every response.
///
/// Two headers are relaxed only under `topcoat dev`, because the live-reload
/// script and its websocket live on the dev server's origin, not ours:
/// `Content-Security-Policy` gains that origin, and
/// `Cross-Origin-Embedder-Policy` is dropped (require-corp would block the
/// cross-origin script). Production responses are unaffected.
#[layer("/")]
async fn security_headers(cx: &mut CxBuilder, body: Body, next: Next<'_>) -> Result<Response> {
    let mut response = next.run(cx, body).await?;
    let headers = response.headers_mut();

    let mut set = |name: &'static str, value: String| {
        if let Ok(value) = HeaderValue::from_str(&value) {
            headers.insert(HeaderName::from_static(name), value);
        }
    };

    set("x-frame-options", "DENY".to_owned());
    set("x-content-type-options", "nosniff".to_owned());
    set("referrer-policy", "no-referrer".to_owned());
    set(
        "permissions-policy",
        "geolocation=(), microphone=(), camera=()".to_owned(),
    );
    set("cross-origin-opener-policy", "same-origin".to_owned());
    set("cross-origin-resource-policy", "same-origin".to_owned());

    match dev_origin() {
        None => {
            set("cross-origin-embedder-policy", "require-corp".to_owned());
            set("content-security-policy", csp("", ""));
        }
        Some(origin) => {
            let ws = origin
                .replacen("https://", "wss://", 1)
                .replacen("http://", "ws://", 1);
            set(
                "content-security-policy",
                csp(&format!(" {origin}"), &format!(" {origin} {ws}")),
            );
        }
    }

    Ok(response)
}

/// The CSP, with optional extra `script-src` / `connect-src` origins for dev.
///
/// Server-rendered pages ship no inline script or style at all.
fn csp(extra_script_src: &str, extra_connect_src: &str) -> String {
    format!(
        "default-src 'self'; \
         script-src 'self'{extra_script_src}; \
         style-src 'self'; \
         img-src 'self' data:; \
         connect-src 'self'{extra_connect_src}; \
         object-src 'none'; \
         base-uri 'self'; \
         form-action 'self'; \
         frame-ancestors 'none'"
    )
}
