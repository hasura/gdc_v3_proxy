use axum::{http::StatusCode, TypedHeader};
use axum_extra::extract::WithRejection;

use crate::{
    config::{SourceConfig, SourceName},
    error::ServerError,
};

#[axum_macros::debug_handler]
pub async fn get_health(
    source_name: Option<SourceName>,
    config: Option<SourceConfig>,
) -> StatusCode {
    // todo: if source_name and config provided, check if that specific source is healthy
    StatusCode::NO_CONTENT
}
