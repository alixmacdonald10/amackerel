#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use amackerel::app::*;
    use axum::{middleware, Router};
    use leptos::logging::log;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        // Add hardening headers to every response.
        .layer(middleware::from_fn(security_headers))
        .with_state(leptos_options);

    // run our app with hyper
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

/// Sets security-hardening headers on every response.
#[cfg(feature = "ssr")]
async fn security_headers(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    use axum::http::header::{HeaderName, HeaderValue};

    let mut response = next.run(request).await;
    let headers = response.headers_mut();

    let set = |headers: &mut axum::http::HeaderMap, name: &'static str, value: &'static str| {
        headers.insert(
            HeaderName::from_static(name),
            HeaderValue::from_static(value),
        );
    };

    set(headers, "x-frame-options", "DENY");
    set(headers, "x-content-type-options", "nosniff");
    set(headers, "referrer-policy", "no-referrer");
    set(
        headers,
        "content-security-policy",
        "default-src 'self'; \
         script-src 'self' 'wasm-unsafe-eval' 'unsafe-inline'; \
         style-src 'self' 'unsafe-inline'; \
         img-src 'self' data:; \
         connect-src 'self'; \
         object-src 'none'; \
         base-uri 'self'; \
         form-action 'self'; \
         frame-ancestors 'none'",
    );
    set(
        headers,
        "permissions-policy",
        "geolocation=(), microphone=(), camera=()",
    );
    set(headers, "cross-origin-embedder-policy", "require-corp");
    set(headers, "cross-origin-opener-policy", "same-origin");
    set(headers, "cross-origin-resource-policy", "same-origin");

    response
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
