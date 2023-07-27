use axum::{
    headers::{Error, Header},
    http::HeaderName,
};
use schemars::JsonSchema;
use serde::{de::DeserializeOwned, Serialize};

static SOURCE_HEADER: HeaderName = HeaderName::from_static("x-hasura-dataconnector-sourcename");
static NAME_HEADER: HeaderName = HeaderName::from_static("x-hasura-dataconnector-config");

#[derive(Debug)]
pub struct SourceName(pub String);
#[derive(Debug)]
pub struct SourceConfig(pub Config);

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for SourceName {
    type Rejection = StatusCode;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        if let Some(source_header) = parts.headers.get(&NAME_HEADER) {
            let source_name = source_header
                .to_str()
                .map_err(|err| StatusCode::BAD_REQUEST)?;
            Ok(Self(source_name.to_owned()))
        } else {
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

#[async_trait]
impl<S: Send + Sync> FromRequestParts<S> for SourceConfig {
    type Rejection = StatusCode;
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        if let Some(config_header) = parts.headers.get(&CONFIG_HEADER) {
            let config: Config = serde_json::from_slice(config_header.as_bytes())
                .map_err(|err| StatusCode::BAD_REQUEST)?;
            Ok(Self(config))
        } else {
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

impl Header for SourceName {
    fn name() -> &'static HeaderName {
        &SOURCE_HEADER
    }
    fn decode<'i, I>(values: &mut I) -> Result<Self, Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i axum::http::HeaderValue>,
    {
        let value = values.next().ok_or_else(Error::invalid)?;

        let value = value
            .to_str()
            .map_err(|_| axum::headers::Error::invalid())?;

        Ok(Self(value.to_owned()))
    }

    fn encode<E: Extend<axum::http::HeaderValue>>(&self, values: &mut E) {
        let value = axum::http::HeaderValue::from_str(&self.0).unwrap();
        values.extend(std::iter::once(value));
    }
}

impl<T: DeserializeOwned + Serialize + JsonSchema> Header for SourceConfig<T> {
    fn name() -> &'static HeaderName {
        &CONFIG_HEADER
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, Error>
    where
        Self: Sized,
        I: Iterator<Item = &'i axum::http::HeaderValue>,
    {
        let value = values.next().ok_or_else(Error::invalid)?;

        let value = value
            .to_str()
            .map_err(|_| axum::headers::Error::invalid())?;

        let config = serde_json::from_str(value).map_err(|_| Error::invalid())?;

        Ok(Self(config))
    }

    fn encode<E: Extend<axum::http::HeaderValue>>(&self, values: &mut E) {
        let value = serde_json::to_string(&self.0).unwrap();
        let value = axum::http::HeaderValue::from_str(&value).unwrap();
        values.extend(std::iter::once(value));
    }
}
