use reqwest::header::{HeaderName, HeaderValue};
use topcoat::{
    context::Cx,
    router::{Body, LayerFuture, Next},
};

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
pub fn security_headers<'a>(cx: &'a Cx, body: Body, next: Next<'a>) -> LayerFuture<'a> {
    Box::pin(async move {
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
                set("content-security-policy", csp(None, None));
            }
            Some(origin) => {
                let ws = origin
                    .replacen("https://", "wss://", 1)
                    .replacen("http://", "ws://", 1);
                set(
                    "content-security-policy",
                    csp(Some(&format!("{origin}")), Some(&format!("{origin} {ws}"))),
                );
            }
        }

        Ok(response)
    })
}

/// The CSP, with optional extra `script-src` / `connect-src` origins for dev.
///
/// Server-rendered pages ship no inline script or style at all.
fn csp(extra_script_src: Option<&str>, extra_connect_src: Option<&str>) -> String {
    let script_src = format!(
        "script-src 'self' {}",
        match extra_script_src {
            Some(src) => src,
            None => "",
        }
    );

    let connect_src = format!(
        "connect-src 'self' {}",
        match extra_connect_src {
            Some(src) => src,
            None => "",
        }
    );

    format!(
        "{}{}{}{}{}{}{}{}{}",
        "default-src 'self';",
        script_src,
        "style-src 'self';",
        "img-src 'self' data:;",
        connect_src,
        "object-src 'none';",
        "base-uri 'self';",
        "form-action 'self';",
        "frame-ancestors 'none'",
    )
}
