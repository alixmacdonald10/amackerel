# amackerel

A simple developer blog built with [Leptos](https://github.com/leptos-rs/leptos)
(SSR + hydration) on [Axum](https://github.com/tokio-rs/axum). Write posts as
markdown files, drop them in `posts/`, and they render as pages.

## Features

- **Blog-first home page** (`/`) — short bio, then all posts newest-first.
- **Markdown posts** — each `posts/*.md` file becomes a page at `/posts/<slug>`.
- **About page** (`/about`) — separate "who I am" page, linked in the nav.
- Rendered server-side with `pulldown-cmark`; frontmatter drives title/date/description.
- **Security-hardened** responses (CSP, X-Frame-Options, nosniff, COEP, etc.).

## Writing a post

Create a file in `posts/`, e.g. `posts/my-project.md`:

```md
---
title: "My Project"
date: "2026-07-02"
description: "Short summary shown in the post list."
---

# My Project

Standard markdown below — headings, lists, code blocks, tables, quotes.
```

- The **filename** (minus `.md`) is the URL slug — this lives at `/posts/my-project`.
- Posts are sorted **newest-first** by the `date` field.
- `title`, `date`, `description` are optional; a missing `title` falls back to the slug.

## Project layout

```text
posts/               markdown blog posts (read at runtime by the server)
src/app.rs           routes, nav, and page components (edit bio + About here)
src/blog.rs          post types, markdown rendering, list_posts/get_post server fns
src/main.rs          Axum server + security-headers middleware
style/main.scss      site styling
public/              static assets (favicon, etc.)
end2end/             Playwright end-to-end tests
Dockerfile           two-stage Alpine build (nightly builder → tiny runtime)
.github/workflows/   CI/CD pipeline
```

## Running

```bash
cargo leptos watch
```

Open http://127.0.0.1:3000

## Prerequisites

`cargo-leptos` uses nightly Rust and dart-sass. If something is missing:

1. `cargo install cargo-leptos --locked`
2. `rustup toolchain install nightly --allow-downgrade`
3. `rustup target add wasm32-unknown-unknown` (also declared in `rust-toolchain.toml`)
4. `npm install -g sass`
5. `npm install` in the `end2end` directory before running tests

## Testing

```bash
# Unit tests
cargo test --features ssr --no-default-features

# End-to-end (builds app, serves on :3000, runs Playwright)
cargo leptos end-to-end
```

Playwright specs live in `end2end/tests`.

## Docker

Two-stage build: Alpine + Rust nightly builder → bare Alpine runtime (~20 MB).

```bash
docker build -t amackerel .
docker run -p 8080:8080 amackerel
```

Open http://localhost:8080

> **Note:** the server reads `posts/` at runtime relative to its working
> directory. The image copies `posts/` in; if you mount your own, mount it at
> `/app/posts`.

## Security headers

`src/main.rs` layers hardening headers onto every response: `Content-Security-Policy`,
`X-Frame-Options: DENY`, `X-Content-Type-Options: nosniff`, `Referrer-Policy`,
`Permissions-Policy`, and the cross-origin isolation trio (COEP/COOP/CORP).

The CSP allows `'wasm-unsafe-eval'` and `'unsafe-inline'` scripts — both required
for Leptos hydration. Don't remove them without switching to a nonce-based CSP,
or hydration breaks.

## CI/CD

`.github/workflows/ci.yml` runs on pushes to `main` and on `X.Y.Z` semver tags.

```mermaid
flowchart TD
    push([Push to main / X.Y.Z tag])
    push --> audit[audit<br/>cargo audit]

    audit --> unit[unit-tests<br/>cargo test]
    audit --> e2e[e2e-tests<br/>Playwright]

    subgraph release [Release path — tag pushes only]
        direction TB
        version[version<br/>require semver tag]
        zap[zap-scan<br/>build image + OWASP ZAP]
        publish[publish<br/>push to GHCR]
        version --> zap
        unit --> zap
        e2e --> zap
        zap -->|scan passes| publish
    end

    version -.->|no valid tag → fail| stop([Pipeline fails])
    zap -.->|alerts → fail| stop
    publish --> ghcr([ghcr.io/&lt;owner&gt;/amackerel<br/>:version + :latest])

    classDef gate fill:#fde68a,stroke:#b45309,color:#111;
    classDef ship fill:#bbf7d0,stroke:#15803d,color:#111;
    class version,zap gate;
    class publish,ghcr ship;
```

| Stage | What it does |
|-------|--------------|
| **audit** | `cargo audit` — fails on any advisory |
| **unit-tests** | `cargo test` (runs in parallel with e2e) |
| **e2e-tests** | Playwright suite (runs in parallel with unit) |
| **version** | requires a valid semver tag on the commit (release only) |
| **zap-scan** | builds the image, runs it, OWASP ZAP baseline scan (release only) |
| **publish** | pushes the *scanned* image to GHCR (release only) |

- **Push to `main`** → audit + tests only. No image is published.
- **Push a `X.Y.Z` tag** (e.g. `0.1.1`, no `v` prefix) → full pipeline; on success
  the exact scanned image is published to `ghcr.io/<owner>/amackerel:<version>`
  and `:latest`.

The image ZAP scans is saved and re-loaded by the publish job, so you publish
precisely the bytes that were tested (not a rebuild).

### Cutting a release

```bash
git tag 0.1.1
git push origin 0.1.1
```

Pull the published image:

```bash
docker pull ghcr.io/alixmacdonald10/amackerel:latest
```

## Deploying without the toolchain

After `cargo leptos build --release`, the minimum files needed are:

1. The server binary in `target/release`
2. The `target/site` directory and its contents
3. The `posts/` directory (read at runtime)

Set these environment variables as needed:

```sh
export LEPTOS_ENV="PROD"
export LEPTOS_OUTPUT_NAME="amackerel"
export LEPTOS_SITE_ROOT="site"
export LEPTOS_SITE_PKG_DIR="pkg"
export LEPTOS_SITE_ADDR="0.0.0.0:3000"
export LEPTOS_RELOAD_PORT="3001"
```

Then run the server binary.

## License

See [LICENSE](LICENSE).
