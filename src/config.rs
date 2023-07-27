use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Config {
    pub base_url: String,
}
use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, HeaderName, StatusCode},
};

use crate::{api::error_response::ErrorResponseType, error::ServerError};

static CONFIG_HEADER: HeaderName = HeaderName::from_static("x-hasura-dataconnector-config");
static SOURCE_HEADER: HeaderName = HeaderName::from_static("x-hasura-dataconnector-sourcename");

#[derive(Debug)]
pub struct SourceName(pub String);
#[derive(Debug)]
pub struct SourceConfig(pub Config);

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for SourceName {
    type Rejection = ServerError;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        if let Some(source_header) = parts.headers.get(&SOURCE_HEADER) {
            let source_name = source_header
                .to_str()
                .map_err(|err| ServerError::UncaughtError {
                    details: None,
                    message: "Unable to parse source name header".to_string(),
                    error_type: ErrorResponseType::UncaughtError,
                })?;
            Ok(Self(source_name.to_owned()))
        } else {
            Err(ServerError::UncaughtError {
                details: None,
                message: "Source name header missing".to_string(),
                error_type: ErrorResponseType::UncaughtError,
            })
        }
    }
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for SourceConfig {
    type Rejection = ServerError;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        if let Some(config_header) = parts.headers.get(&CONFIG_HEADER) {
            let config: Config =
                serde_json::from_slice(config_header.as_bytes()).map_err(|err| {
                    ServerError::UncaughtError {
                        details: None,
                        message: "Unable to parse config header".to_string(),
                        error_type: ErrorResponseType::UncaughtError,
                    }
                })?;
            Ok(Self(config))
        } else {
            Err(ServerError::UncaughtError {
                details: None,
                message: "Config header missing".to_string(),
                error_type: ErrorResponseType::UncaughtError,
            })
        }
    }
}
