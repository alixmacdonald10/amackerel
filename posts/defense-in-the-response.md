---
title: "Defense in the Response"
date: "2026-07-04"
description: "The perimeter is shut — now harden what the app actually sends back. Security headers, a CSP that survives WASM hydration, and full cross-origin isolation."
---

<!--
DRAFT SCAFFOLD — section headers + notes only. Fill each section with prose.
Series: "Zero-Exposure" (2 of 3). Prev: The Locked Front Door. Next: Proving It Before It Ships.
Source: src/main.rs security_headers middleware + .zap/rules.tsv.
-->

# Defense in the Response

<!--
HOOK: article 1 shut the network perimeter. But every HTTP response is still an
attack surface — headers tell the browser what's allowed. Here's every header this
blog sets and why.
-->

## One tower layer, every response

<!--
- middleware::from_fn(security_headers) as a tower layer on the Axum router.
- Runs on every response; small `set` closure with HeaderValue::from_static.
- CODE: the middleware wiring + the set closure from src/main.rs.
-->

## The easy wins

<!--
- x-frame-options: DENY, x-content-type-options: nosniff, referrer-policy: no-referrer.
- permissions-policy: geolocation=(), microphone=(), camera=() — deny powerful features.
- One line each on the threat each closes (clickjacking, MIME sniffing, referrer leak).
-->

## The CSP, and the WASM problem

<!--
- Full CSP string: default-src 'self'; script-src 'self' 'wasm-unsafe-eval' 'unsafe-inline'; etc.
- WHY 'wasm-unsafe-eval': instantiating the Leptos/wasm-bindgen module counts as eval under CSP.
- WHY 'unsafe-inline' in script-src: the Leptos hydration bootstrap script boots the client.
- WHY 'unsafe-inline' in style-src: inline styles emitted during SSR.
- The honest tension: 'unsafe-inline' weakens CSP; justified here (static blog, no reflected user input).
- Forward ref: what a nonce-based CSP would take to remove it.
-->

```
default-src 'self';
script-src 'self' 'wasm-unsafe-eval' 'unsafe-inline';
style-src 'self' 'unsafe-inline';
img-src 'self' data:;
connect-src 'self';
object-src 'none';
base-uri 'self';
form-action 'self';
frame-ancestors 'none'
```

## Cross-origin isolation: the COEP/COOP/CORP trio

<!--
- cross-origin-embedder-policy: require-corp
- cross-origin-opener-policy: same-origin
- cross-origin-resource-policy: same-origin
- What isolation buys (Spectre-class mitigations, no cross-origin window handles).
- Note the cost: it constrains what third-party resources you can embed.
-->

## Telling the scanner what's intentional

<!--
- .zap/rules.tsv: IGNORE 10055 (CSP unsafe-inline) with the justification, IGNORE 10049 (cacheable static).
- Point: a suppressed finding should carry a written reason, not just silence.
- Ties into article 3 (the ZAP gate that enforces this on every release).
-->

## Takeaway

<!--
- Headers are cheap, high-leverage defense-in-depth.
- Be honest about the CSP compromises the framework forces, and document them.
- Tease article 3: headers are set — but who checks they stay set before each deploy?
-->
