use std::{str::FromStr, vec};

use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::WithRejection;
use indexmap::IndexMap;
use ndc_client::models;

use crate::{
    api::{raw_request::RawRequest, raw_response::RawResponse},
    config::{ProxyTarget, SourceConfig, SourceName},
    error::ServerError,
};

#[axum_macros::debug_handler]
pub async fn post_raw(
    ProxyTarget(base_url): ProxyTarget,
    SourceName(_source_name): SourceName,
    SourceConfig(config): SourceConfig,
    WithRejection(Json(request), _): WithRejection<Json<RawRequest>, ServerError>,
) -> StatusCode {
    StatusCode::NOT_IMPLEMENTED
}
