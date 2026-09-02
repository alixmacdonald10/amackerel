//! Compiles `style/tailwind.css` with the standalone Tailwind CLI and writes it
//! to `$OUT_DIR/tailwind.css`, where `tailwind::stylesheet!()` picks it up as a
//! content-hashed asset. No npm, no dart-sass, no PostCSS.
//!
//! topcoat downloads and caches the CLI itself, but only ever fetches the glibc
//! Linux build, which cannot run on Alpine. Setting `TAILWIND_CLI` to a
//! preinstalled binary (see the Dockerfile) overrides the download.
//!
//! No `cargo:rerun-if-*` directives on purpose: printing even one replaces
//! Cargo's default "rerun when any package file changed", which is exactly what
//! picks up new utility classes in `view!` markup.
fn main() {
    let mut config = topcoat::tailwind::BuildConfig::new().input("styles.css");

    if std::env::var_os("TAILWIND_CLI").is_some() {
        config = config.executable_env("TAILWIND_CLI");
    }

    config.render().unwrap();

    topcoat::icon::iconify::BuildConfig::new()
        .icon_set("hugeicons")
        .stage()
        .unwrap();
}
