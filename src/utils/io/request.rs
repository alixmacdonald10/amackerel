use std::str::FromStr;

use anyhow::Result;
use graphql_client::GraphQLQuery;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use secrecy::SecretString;
use serde::Deserialize;

use crate::{
    projects::RepositoryMeta,
    utils::io::{
        github::{compile_github_headers, GITHUB_API_URL},
        request::repositories_query::ResponseData,
    },
};

#[allow(clippy::upper_case_acronyms)]
type URI = String;

// The paths are relative to the directory where your `Cargo.toml` is located.
// Both json and the GraphQL schema language are supported as sources for the schema
#[derive(GraphQLQuery, Deserialize)]
#[graphql(
    schema_path = "queries/github_schema.json",
    query_path = "queries/repositories/query.graphql",
    response_derives = "Debug"
)]
pub struct RepositoriesQuery;

pub async fn perform_repositories_query(
    variables: repositories_query::Variables,
    client: &reqwest::Client,
    token: &SecretString,
) -> Result<Vec<RepositoryMeta>> {
    let request_body = RepositoriesQuery::build_query(variables);

    let headers = compile_github_headers(token)?;
    let res = client
        .post(format!("{GITHUB_API_URL}/graphql"))
        .headers(headers.to_owned())
        .json(&request_body)
        .send()
        .await?;

    // TODO: tidy this up
    match res.error_for_status() {
        Ok(res) => {
            let res = res.json::<graphql_client::Response<ResponseData>>().await?;

            // TODO: handle graphQL errors
            let data = res.data;
            if let Some(data) = data {
                Ok(data.into())
            } else {
                Err(anyhow::anyhow!("its buggered"))
            }
        }
        Err(err) => Err(anyhow::anyhow!(
            "An error occured. status code: {:?} message: {}",
            err.status().unwrap_or_default(),
            err
        )),
    }
}

pub fn set_header(headers: &mut HeaderMap, name: &str, value: &str) -> anyhow::Result<()> {
    let value = HeaderValue::from_str(value)?;
    let name = HeaderName::from_str(name)?;
    headers.insert(name, value);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_header_inserts_the_name_and_value() {
        let mut headers = HeaderMap::new();

        set_header(&mut headers, "content-type", "text/html").expect("header should be set");

        assert_eq!(headers["content-type"], "text/html");
        assert_eq!(headers.len(), 1);
    }

    #[test]
    fn set_header_normalises_a_mixed_case_name() {
        let mut headers = HeaderMap::new();

        set_header(&mut headers, "Content-Type", "text/html").expect("header should be set");

        assert_eq!(headers["content-type"], "text/html");
    }

    #[test]
    fn set_header_overwrites_an_existing_value() {
        let mut headers = HeaderMap::new();

        set_header(&mut headers, "content-type", "text/plain").expect("header should be set");
        set_header(&mut headers, "content-type", "text/html").expect("header should be set");

        // `insert`, not `append`: repeated calls from the middleware stay idempotent.
        assert_eq!(headers["content-type"], "text/html");
        assert_eq!(headers.len(), 1);
    }

    #[test]
    fn set_header_leaves_other_headers_untouched() {
        let mut headers = HeaderMap::new();
        set_header(&mut headers, "cross-origin-opener-policy", "same-origin")
            .expect("header should be set");

        set_header(&mut headers, "cross-origin-embedder-policy", "require-corp")
            .expect("header should be set");

        assert_eq!(headers["cross-origin-opener-policy"], "same-origin");
        assert_eq!(headers["cross-origin-embedder-policy"], "require-corp");
        assert_eq!(headers.len(), 2);
    }

    #[test]
    fn set_header_errors_on_a_value_with_invalid_bytes() {
        let mut headers = HeaderMap::new();

        assert!(set_header(&mut headers, "content-type", "bad\nvalue").is_err());
    }

    #[test]
    fn set_header_errors_on_an_invalid_name() {
        let mut headers = HeaderMap::new();

        assert!(set_header(&mut headers, "bad name", "text/html").is_err());
        assert!(set_header(&mut headers, "", "text/html").is_err());
    }

    #[test]
    fn set_header_does_not_mutate_the_map_on_error() {
        let mut headers = HeaderMap::new();
        set_header(&mut headers, "content-type", "text/html").expect("header should be set");

        assert!(set_header(&mut headers, "content-type", "bad\nvalue").is_err());

        assert_eq!(headers["content-type"], "text/html");
        assert_eq!(headers.len(), 1);
    }
}
