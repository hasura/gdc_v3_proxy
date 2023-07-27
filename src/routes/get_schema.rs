use axum::{http::StatusCode, Json, TypedHeader};

use axum_extra::extract::WithRejection;
use ndc_client::models;
use serde::{Deserialize, Serialize};

use crate::{
    api::schema_response::{ColumnInfo, SchemaResponse, TableInfo},
    config::{SourceConfig, SourceName},
    error::ServerError,
};

#[axum_macros::debug_handler]
pub async fn get_schema(
    SourceName(source_name): SourceName,
    SourceConfig(config): SourceConfig,
) -> Result<Json<SchemaResponse>, ServerError> {
    let url = format!("{}/schema", config.base_url);
    let client = reqwest::Client::new();
    let request = client.get(&url).send().await?.text().await?;
    let response = serde_json::from_str(&request)?;

    let response = map_response(response);

    Ok(Json(response))
}

fn map_response(response: models::SchemaResponse) -> SchemaResponse {
    SchemaResponse {
        tables: response
            .tables
            .iter()
            .map(|table| {
                let table_type = response
                    .object_types
                    .get(&table.table_type)
                    .expect("Tables should have corresponding object type");
                TableInfo {
                    name: vec![table.name.to_owned()],
                    description: table.description.to_owned(),
                    insertable: table
                        .insertable_columns
                        .as_ref()
                        .map(|insertable_columns| !insertable_columns.is_empty()),
                    updatable: table
                        .updatable_columns
                        .as_ref()
                        .map(|updatable_columns| !updatable_columns.is_empty()),
                    deletable: Some(table.deletable),
                    primary_key: None,
                    foreign_keys: None,
                    table_type: None,
                    columns: table_type
                        .fields
                        .iter()
                        .map(|(field_name, field_info)| ColumnInfo {
                            name: field_name.to_owned(),
                            column_type: match &field_info.r#type {
                                models::Type::Named { name } => name.to_owned(),
                                models::Type::Nullable { underlying_type } => {
                                    match &**underlying_type {
                                        models::Type::Named { name } => name.to_owned(),
                                        _ => panic!("type not supported"),
                                    }
                                }
                                _ => panic!("type not supported"),
                            },
                            nullable: matches!(field_info.r#type, models::Type::Nullable { .. }),
                            description: field_info.description.to_owned(),
                            insertable: table
                                .insertable_columns
                                .as_ref()
                                .map(|insertable_columns| insertable_columns.contains(field_name)),
                            updatable: table
                                .updatable_columns
                                .as_ref()
                                .map(|updatable_columns| updatable_columns.contains(field_name)),
                        })
                        .collect(),
                }
            })
            .collect(),
    }
}
