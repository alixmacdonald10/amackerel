pub mod io;

/// True when running under `topcoat dev`, which serves its live-reload client
/// script from a second origin.
pub fn dev_origin() -> Option<String> {
    std::env::var("TOPCOAT_DEV_URL").ok()
}
