use serde::Serialize;
use topcoat::{
    context::Cx,
    router::{content::Json, route},
    Result,
};

#[derive(Debug, Clone, Serialize)]
struct Health {
    status: String,
}

#[route(GET "/healthz")]
pub async fn healthz(_cx: &Cx) -> Result<Json<Health>> {
    Ok(Health {
        status: "ok".to_string(),
    }
    .into())
}
