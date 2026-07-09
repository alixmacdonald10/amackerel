# amackerel

A simple developer blog built with [Leptos](https://github.com/leptos-rs/leptos)
(SSR + hydration) on [Axum](https://github.com/tokio-rs/axum). Write posts as
markdown files, drop them in `posts/`, and they render as pages.

## Features

- **Blog-first home page** (`/`) — short bio banner, then all posts newest-first.
- **Markdown posts** — each `posts/*.md` file becomes a page at `/posts/<slug>`.
- **About page** (`/about`) — separate "who I am" page, linked in the nav.
- Rendered server-side with `pulldown-cmark`; frontmatter drives title/date/description.

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
posts/          markdown blog posts (read at runtime by the server)
src/app.rs      routes, nav, and page components (edit bio + About here)
src/blog.rs     post types, markdown rendering, list_posts/get_post server fns
style/main.scss site styling
public/         static assets (favicon, etc.)
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
3. `rustup target add wasm32-unknown-unknown`
4. `npm install -g sass`
5. `npm install` in the `end2end` directory before running tests

## Building for release

```bash
cargo leptos build --release
```

Produces the server binary in `target/release` and the site package in `target/site`.

> **Note:** the server reads `posts/` at runtime relative to its working directory.
> When deploying, the `posts/` folder must sit next to where you run the binary.

## Testing

```bash
cargo leptos end-to-end
```

Uses Playwright; tests live in `end2end/tests`.

## Deploying without the toolchain

After `cargo leptos build --release`, the minimum files needed are:

1. The server binary in `target/release`
2. The `target/site` directory and its contents
3. The `posts/` directory (read at runtime)

Set these environment variables as needed:

```sh
export LEPTOS_OUTPUT_NAME="amackerel"
export LEPTOS_SITE_ROOT="site"
export LEPTOS_SITE_PKG_DIR="pkg"
export LEPTOS_SITE_ADDR="127.0.0.1:3000"
export LEPTOS_RELOAD_PORT="3001"
```

Then run the server binary.

## License

See [LICENSE](LICENSE).
