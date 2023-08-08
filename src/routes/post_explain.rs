use axum::{extract::State, Json};
use axum_extra::extract::WithRejection;
use ndc_client::models;

use crate::{
    api::{explain_response::ExplainResponse, query_request::QueryRequest},
    config::{ProxyTarget, SourceConfig, SourceName},
    error::ServerError,
};

use super::post_query::map_request;

#[axum_macros::debug_handler]
pub async fn post_explain(
    ProxyTarget(base_url): ProxyTarget,
    SourceName(source_name): SourceName,
    SourceConfig(config): SourceConfig,
    WithRejection(Json(request), _): WithRejection<Json<QueryRequest>, ServerError>,
) -> Result<Json<ExplainResponse>, ServerError> {
    let url = format!("{}/explain", base_url);

    let client = reqwest::Client::new();

    let request = map_request(request);
    let body = serde_json::to_string(&request)?;

    let request = client.post(&url).body(body).send().await?.text().await?;

    let response = serde_json::from_str(&request)?;
    let response = map_response(response);

    Ok(Json(response))
}

fn map_response(response: models::ExplainResponse) -> ExplainResponse {
    todo!()
}
