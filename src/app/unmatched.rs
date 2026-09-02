use topcoat::{
    router::{error::not_found, page},
    Result,
};

/// Catches every unmatched path so the branded 404 renders inside the layout.
///
/// The router's own not-found response bypasses layouts entirely, so a page has
/// to raise the error for `root_layout` to catch it.
#[page("/{*path}")]
pub async fn unmatched() -> Result {
    Err(not_found().into())
}
