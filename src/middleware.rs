use reqwest::header::HeaderMap;
use topcoat::{
    context::Cx,
    router::{Body, LayerFuture, Next},
};

use crate::utils::io::request::set_header;

const DEFAULT_HEADERS: &[(&str, &str)] = &[
    ("x-frame-options", "DENY"),
    ("x-content-type-options", "nosniff"),
    ("referrer-policy", "no-referrer"),
    (
        "permissions-policy",
        "geolocation=(), microphone=(), camera=()",
    ),
    ("cross-origin-opener-policy", "same-origin"),
    ("cross-origin-resource-policy", "same-origin"),
    (
        "cache-control",
        "no-cache, no-store, must-revalidate, private",
    ),
];

enum ResponseHeaders {
    Dev { origin: String },
    Prod,
}

impl ResponseHeaders {
    fn compile(self, headers: &mut HeaderMap) -> anyhow::Result<()> {
        for (name, value) in DEFAULT_HEADERS {
            set_header(headers, name, value)?;
        }

        match self {
            ResponseHeaders::Dev { origin } => {
                let ws = origin
                    .replacen("https://", "wss://", 1)
                    .replacen("http://", "ws://", 1);
                set_header(
                    headers,
                    "content-security-policy",
                    Self::csp(Some(&origin.to_string()), Some(&format!("{origin} {ws}"))).as_str(),
                )?;
            }
            ResponseHeaders::Prod => {
                set_header(headers, "cross-origin-embedder-policy", "require-corp")?;
                set_header(
                    headers,
                    "content-security-policy",
                    Self::csp(None, None).as_str(),
                )?;
            }
        };
        Ok(())
    }

    /// The content security policy (CSP), with optional extra `script-src` / `connect-src` origins for dev.
    fn csp(extra_script_src: Option<&str>, extra_connect_src: Option<&str>) -> String {
        // TODO: fix the scrappy extra space on None inputs
        let script_src = format!(
            "script-src 'self' {};",
            extra_script_src.unwrap_or_default()
        );

        let connect_src = format!(
            "connect-src 'self' {};",
            extra_connect_src.unwrap_or_default()
        );

        format!(
            "{}{}{}{}{}{}{}{}{}{}",
            "default-src 'self';",
            script_src,
            "style-src 'self';",
            // Fontsource serves the Geist woff2 files; the stylesheet that
            // references them is still same-origin.
            "font-src 'self' https://cdn.jsdelivr.net;",
            "img-src 'self' data:;",
            connect_src,
            "object-src 'none';",
            "base-uri 'self';",
            "form-action 'self';",
            "frame-ancestors 'none'",
        )
    }
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

        match crate::utils::dev_origin() {
            None => ResponseHeaders::Prod,
            Some(origin) => ResponseHeaders::Dev { origin },
        }
        .compile(headers)?;

        Ok(response)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compiled(variant: ResponseHeaders) -> HeaderMap {
        let mut headers = HeaderMap::new();
        variant
            .compile(&mut headers)
            .expect("headers should compile");
        headers
    }

    fn dev(origin: &str) -> HeaderMap {
        compiled(ResponseHeaders::Dev {
            origin: origin.to_string(),
        })
    }

    fn assert_carries_default_headers(headers: &HeaderMap) {
        for (name, value) in DEFAULT_HEADERS {
            assert_eq!(&headers[*name], value, "missing or wrong header `{name}`");
        }
    }

    #[test]
    fn both_variants_carry_every_default_header() {
        assert_carries_default_headers(&compiled(ResponseHeaders::Prod));
        assert_carries_default_headers(&dev("https://example.com"));
    }

    #[test]
    fn both_variants_set_a_content_security_policy() {
        assert!(compiled(ResponseHeaders::Prod).contains_key("content-security-policy"));
        assert!(dev("https://example.com").contains_key("content-security-policy"));
    }

    #[test]
    fn prod_sets_cross_origin_embedder_policy_but_dev_does_not() {
        assert_eq!(
            compiled(ResponseHeaders::Prod)["cross-origin-embedder-policy"],
            "require-corp"
        );
        // require-corp would block the live-reload script served from the other origin.
        assert!(!dev("https://example.com").contains_key("cross-origin-embedder-policy"));
    }

    #[test]
    fn prod_policy_allows_no_extra_origins() {
        let headers = compiled(ResponseHeaders::Prod);
        let csp = headers["content-security-policy"]
            .to_str()
            .expect("policy should be ascii");

        assert!(csp.contains("script-src 'self' ;"));
        assert!(csp.contains("connect-src 'self' ;"));
    }

    #[test]
    fn dev_policy_allows_the_origin_for_scripts_and_its_websocket_for_connections() {
        let headers = dev("https://example.com");
        let csp = headers["content-security-policy"]
            .to_str()
            .expect("policy should be ascii");

        assert!(csp.contains("script-src 'self' https://example.com;"));
        assert!(csp.contains("connect-src 'self' https://example.com wss://example.com;"));
    }

    #[test]
    fn dev_derives_an_insecure_websocket_scheme_from_an_http_origin() {
        let headers = dev("http://localhost:3000");
        let csp = headers["content-security-policy"]
            .to_str()
            .expect("policy should be ascii");

        assert!(csp.contains("connect-src 'self' http://localhost:3000 ws://localhost:3000;"));
    }

    #[test]
    fn dev_rewrites_only_the_scheme_not_a_later_occurrence() {
        let headers = dev("https://example.com/https://nested");
        let csp = headers["content-security-policy"]
            .to_str()
            .expect("policy should be ascii");

        // `replacen(.., 1)`: the path keeps its literal `https://`.
        assert!(csp.contains("wss://example.com/https://nested"));
    }

    #[test]
    fn dev_errors_on_an_origin_with_invalid_header_bytes() {
        let mut headers = HeaderMap::new();

        let result = ResponseHeaders::Dev {
            origin: "https://exa\nmple.com".to_string(),
        }
        .compile(&mut headers);

        assert!(result.is_err());
    }

    #[test]
    fn compile_overwrites_headers_already_on_the_response() {
        let mut headers = HeaderMap::new();
        headers.insert("x-frame-options", "SAMEORIGIN".parse().unwrap());
        headers.insert("content-security-policy", "default-src *".parse().unwrap());

        ResponseHeaders::Prod
            .compile(&mut headers)
            .expect("headers should compile");

        assert_eq!(headers["x-frame-options"], "DENY");
        assert_eq!(headers.get_all("x-frame-options").iter().count(), 1);
        assert!(!headers["content-security-policy"]
            .to_str()
            .unwrap()
            .contains("default-src *"));
    }

    #[test]
    fn csp_carries_every_locked_down_directive() {
        let csp = ResponseHeaders::csp(None, None);

        for directive in [
            "default-src 'self';",
            "style-src 'self';",
            "font-src 'self' https://cdn.jsdelivr.net;",
            "img-src 'self' data:;",
            "object-src 'none';",
            "base-uri 'self';",
            "form-action 'self';",
            "frame-ancestors 'none'",
        ] {
            assert!(csp.contains(directive), "policy is missing `{directive}`");
        }
    }

    #[test]
    fn csp_starts_with_default_src_and_has_no_trailing_semicolon() {
        let csp = ResponseHeaders::csp(None, None);

        assert!(csp.starts_with("default-src 'self';"));
        assert!(csp.ends_with("frame-ancestors 'none'"));
    }

    #[test]
    fn csp_interpolates_the_extra_origins_independently() {
        let csp = ResponseHeaders::csp(Some("https://scripts.test"), Some("wss://sockets.test"));

        assert!(csp.contains("script-src 'self' https://scripts.test;"));
        assert!(csp.contains("connect-src 'self' wss://sockets.test;"));
    }
}
