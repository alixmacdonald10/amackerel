use leptos::prelude::*;
use serde::{Deserialize, Serialize};

/// Metadata for a single post, parsed from its markdown frontmatter.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostMeta {
    pub slug: String,
    pub title: String,
    pub date: String,
    pub description: String,
}

/// A fully rendered post: metadata plus the markdown body compiled to HTML.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Post {
    pub meta: PostMeta,
    pub html: String,
}

#[cfg(feature = "ssr")]
mod ssr_impl {
    use super::{Post, PostMeta};
    use std::fs;
    use std::path::Path;

    /// Splits `--- ... ---` frontmatter off the top of a file and parses
    /// the `key: value` lines. Missing keys fall back to sensible defaults.
    pub fn parse_frontmatter(raw: &str, slug: &str) -> (PostMeta, String) {
        let mut title = slug.to_string();
        let mut date = String::new();
        let mut description = String::new();

        let body = if let Some(rest) = raw.strip_prefix("---") {
            if let Some(end) = rest.find("\n---") {
                let frontmatter = &rest[..end];
                for line in frontmatter.lines() {
                    if let Some((k, v)) = line.split_once(':') {
                        let v = v.trim().trim_matches('"').to_string();
                        match k.trim() {
                            "title" => title = v,
                            "date" => date = v,
                            "description" => description = v,
                            _ => {}
                        }
                    }
                }
                // Skip past the closing `---` line.
                rest[end + 4..].trim_start_matches(['\r', '\n']).to_string()
            } else {
                raw.to_string()
            }
        } else {
            raw.to_string()
        };

        (
            PostMeta {
                slug: slug.to_string(),
                title,
                date,
                description,
            },
            body,
        )
    }

    pub fn render_markdown(body: &str) -> String {
        use pulldown_cmark::{html, Options, Parser};
        let mut opts = Options::empty();
        opts.insert(Options::ENABLE_STRIKETHROUGH);
        opts.insert(Options::ENABLE_TABLES);
        opts.insert(Options::ENABLE_FOOTNOTES);
        let parser = Parser::new_ext(body, opts);
        let mut out = String::new();
        html::push_html(&mut out, parser);
        out
    }

    /// Reads every `posts/*.md` file, sorted newest-first by date.
    pub fn read_all() -> Vec<Post> {
        let mut posts = Vec::new();
        if let Ok(entries) = fs::read_dir(Path::new("posts")) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) != Some("md") {
                    continue;
                }
                let slug = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .to_string();
                if let Ok(raw) = fs::read_to_string(&path) {
                    let (meta, body) = parse_frontmatter(&raw, &slug);
                    posts.push(Post {
                        meta,
                        html: render_markdown(&body),
                    });
                }
            }
        }
        posts.sort_by(|a, b| b.meta.date.cmp(&a.meta.date));
        posts
    }
}

#[server(ListPosts, "/api")]
pub async fn list_posts() -> Result<Vec<PostMeta>, ServerFnError> {
    Ok(ssr_impl::read_all().into_iter().map(|p| p.meta).collect())
}

#[server(GetPost, "/api")]
pub async fn get_post(slug: String) -> Result<Option<Post>, ServerFnError> {
    Ok(ssr_impl::read_all()
        .into_iter()
        .find(|p| p.meta.slug == slug))
}
