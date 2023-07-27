use std::{str::FromStr, vec};

use axum::{http::StatusCode, Json};
use axum_extra::extract::WithRejection;
use indexmap::IndexMap;
use ndc_client::models;

use crate::{
    api::{raw_request::RawRequest, raw_response::RawResponse},
    config::{SourceConfig, SourceName},
    error::ServerError,
};

#[axum_macros::debug_handler]
pub async fn post_raw(
    SourceName(_source_name): SourceName,
    SourceConfig(config): SourceConfig,
    WithRejection(Json(request), _): WithRejection<Json<RawRequest>, ServerError>,
) -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
    // ) -> Result<Json<RawResponse>, ServerError> {
    // let url = format!("{}/raw", config.base_url);

    // let client = reqwest::Client::new();

    // let request = map_request(request);
    // let body = serde_json::to_string(&request)?;

    // let request = client.post(&url).body(body).send().await?.text().await?;

    // let response = serde_json::from_str(&request)?;
    // let response = map_response(response);

    // Ok(Json(response))
}

// fn map_request(request: RawRequest) -> models::RawRequest {
//     todo!()
// }

// fn map_response(response: models::RawResponse) -> RawResponse {
//     todo!()
// }
