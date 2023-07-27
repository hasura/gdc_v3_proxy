use axum::{extract::State, Json};
use indexmap::IndexMap;
use ndc_client::models;
use schemars::schema_for;

use crate::{
    error::ServerError,
    ServerState,
    {
        api::capabilities_response::{
            Capabilities, CapabilitiesResponse, ColumnNullability, ComparisonCapabilities,
            ConfigSchemaResponse, DataSchemaCapabilities, GraphQlType, QueryCapabilities,
            ScalarTypeCapabilities, SubqueryComparisonCapabilities,
        },
        config::Config,
    },
};

#[axum_macros::debug_handler]
pub async fn get_capabilities(
    State(state): State<ServerState>,
) -> Result<Json<CapabilitiesResponse>, ServerError> {
    let url = format!("{}/capabilities", state.base_url);
    let client = reqwest::Client::new();
    let request = client.get(&url).send().await?.text().await?;
    let capabilities_response = serde_json::from_str(&request)?;

    let url = format!("{}/schema", state.base_url);
    let client = reqwest::Client::new();
    let request = client.get(&url).send().await?.text().await?;
    let schema_response = serde_json::from_str(&request)?;

    let response = map_capabilities(capabilities_response, schema_response);

    Ok(Json(response))
}

fn map_capabilities(
    capabilities: models::CapabilitiesResponse,
    schema: models::SchemaResponse,
) -> CapabilitiesResponse {
    CapabilitiesResponse {
        display_name: Some("Hasura GDC v2 proxy for v3".to_string()),
        release_name: Some(capabilities.versions),
        config_schemas: ConfigSchemaResponse {
            config_schema: schema_for!(Config),
            other_schemas: IndexMap::new(),
        },
        capabilities: Capabilities {
            comparisons: Some(ComparisonCapabilities {
                subquery: Some(SubqueryComparisonCapabilities {
                    supports_relations: capabilities
                        .capabilities
                        .query
                        .as_ref()
                        .map(|query| query.relation_comparisons.is_some()),
                }),
            }),
            data_schema: Some(DataSchemaCapabilities {
                column_nullability: Some(ColumnNullability::NullableAndNonNullable),
                supports_foreign_keys: Some(false),
                supports_primary_keys: Some(true),
            }),
            datasets: None,
            explain: None,
            metrics: None,
            mutations: None,
            queries: capabilities
                .capabilities
                .query
                .map(|query| QueryCapabilities {
                    foreach: query.foreach,
                }),
            raw: None,
            relationships: capabilities.capabilities.relationships,
            scalar_types: IndexMap::from_iter(schema.scalar_types.into_iter().map(
                |(key, scalar_type)| {
                    (
                        key,
                        ScalarTypeCapabilities {
                            aggregate_functions: Some(IndexMap::from_iter(
                                scalar_type.aggregate_functions.into_iter().map(
                                    |(key, aggregate_function)| {
                                        (
                                            key,
                                            match aggregate_function.result_type {
                                                models::Type::Named { name } => name,
                                                _ => panic!(
                                                    "Unsupported aggregate function return type"
                                                ),
                                            },
                                        )
                                    },
                                ),
                            )),
                            comparison_operators: Some(IndexMap::from_iter(
                                scalar_type.comparison_operators.into_iter().map(
                                    |(key, comparison_operator)| {
                                        (
                                            key,
                                            match comparison_operator.argument_type {
                                                models::Type::Named { name } => name,
                                                _ => panic!(
                                                    "Unsupported comparison operator argument type"
                                                ),
                                            },
                                        )
                                    },
                                ),
                            )),
                            update_column_operators: Some(IndexMap::new()),
                            graphql_type: GraphQlType::String,
                        },
                    )
                },
            )),
            subscriptions: None,
        },
    }
}
