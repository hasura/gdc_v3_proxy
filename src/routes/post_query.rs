use std::collections::HashMap;

use axum::{extract::State, http::response, Json};
use axum_extra::extract::WithRejection;
use indexmap::IndexMap;
use ndc_client::models;

use crate::{
    api::{
        query_request::{
            Aggregate, ComparisonColumn, ComparisonValue, ExistsInTable, Expression, Field,
            OrderBy, OrderByElement, OrderByRelation, OrderByTarget, OrderDirection, Query,
            QueryRequest, Relationship, RelationshipType, TableRelationships,
        },
        query_response::{QueryResponse, RowFieldValue},
    },
    config::{SourceConfig, SourceName},
    error::ServerError,
    ServerState,
};

#[axum_macros::debug_handler(state = ServerState)]
pub async fn post_query(
    SourceName(source_name): SourceName,
    SourceConfig(config): SourceConfig,
    State(state): State<ServerState>,
    WithRejection(Json(request), _): WithRejection<Json<QueryRequest>, ServerError>,
) -> Result<Json<QueryResponse>, ServerError> {
    let url = format!("{}/query", state.base_url);

    let client = reqwest::Client::new();

    let is_foreach = request.foreach.is_some();

    let request = map_request(request);

    let request = client
        .post(&url)
        .json(&request)
        .send()
        .await?
        .text()
        .await?;

    let response = serde_json::from_str(&request)?;
    let response = map_response(response, is_foreach);

    Ok(Json(response))
}

fn table_name(table: Vec<String>) -> String {
    // we expect always exactly one string in the table name string vector
    table.into_iter().next().unwrap()
}

pub fn map_request(request: QueryRequest) -> models::QueryRequest {
    let QueryRequest {
        foreach,
        table,
        table_relationships,
        query,
    } = request;

    let variables = foreach.map(|foreach| {
        foreach
            .into_iter()
            .map(|map| HashMap::from_iter(map.into_iter().map(|(key, value)| (key, value.value))))
            .collect()
    });

    models::QueryRequest {
        table: table_name(table.clone()),
        arguments: HashMap::new(),
        variables,
        query: map_query(query, table_name(table), &table_relationships),
        table_relationships: HashMap::from_iter(table_relationships.into_iter().flat_map(
            |relationship| {
                relationship.relationships.into_iter().map(
                    move |(relationship_name, relationship_info)| {
                        let Relationship {
                            column_mapping,
                            relationship_type,
                            target_table,
                        } = relationship_info;
                        (
                            format!(
                                "{}.{}",
                                table_name(relationship.source_table.clone()),
                                relationship_name
                            ),
                            models::Relationship {
                                column_mapping: HashMap::from_iter(column_mapping.into_iter()),
                                relationship_type: match relationship_type {
                                    RelationshipType::Object => models::RelationshipType::Object,
                                    RelationshipType::Array => models::RelationshipType::Array,
                                },
                                source_table_or_type: table_name(relationship.source_table.clone()),
                                target_table: table_name(target_table),
                                arguments: HashMap::new(),
                            },
                        )
                    },
                )
            },
        )),
    }
}

fn map_query(
    query: Query,
    table: String,
    table_relationships: &Vec<TableRelationships>,
) -> models::Query {
    let Query {
        aggregates,
        aggregates_limit,
        fields,
        limit,
        offset,
        order_by,
        selection,
    } = query;

    let order_by = order_by.map(|order_by| {
        let OrderBy {
            elements,
            relations,
        } = order_by;
        models::OrderBy {
            elements: elements
                .into_iter()
                .map(|element| {
                    let OrderByElement {
                        order_direction,
                        target,
                        target_path,
                    } = element;

                    models::OrderByElement {
                        order_direction: match order_direction {
                            OrderDirection::Asc => models::OrderDirection::Asc,
                            OrderDirection::Desc => models::OrderDirection::Desc,
                        },
                        target: match target {
                            OrderByTarget::StarCountAggregate => {
                                models::OrderByTarget::StarCountAggregate {
                                    path: map_order_by_path(
                                        target_path,
                                        relations.to_owned(),
                                        table.to_owned(),
                                        table_relationships,
                                    ),
                                }
                            }
                            OrderByTarget::SingleColumnAggregate {
                                column,
                                function,
                                result_type,
                            } => models::OrderByTarget::SingleColumnAggregate {
                                column,
                                function,
                                path: map_order_by_path(
                                    target_path,
                                    relations.to_owned(),
                                    table.to_owned(),
                                    table_relationships,
                                ),
                            },
                            OrderByTarget::Column { column } => models::OrderByTarget::Column {
                                name: column,
                                path: map_order_by_path_without_predicate(
                                    target_path,
                                    relations.to_owned(),
                                    table.to_owned(),
                                    table_relationships,
                                ),
                            },
                        },
                    }
                })
                .collect(),
        }
    });
    let aggregates = aggregates.map(|aggregates| {
        HashMap::from_iter(aggregates.into_iter().map(|(key, aggregate)| {
            (
                key,
                match aggregate {
                    Aggregate::ColumnCount { column, distinct } => {
                        models::Aggregate::ColumnCount { column, distinct }
                    }
                    Aggregate::SingleColumn {
                        column,
                        function,
                        result_type,
                    } => models::Aggregate::SingleColumn { column, function },
                    Aggregate::StarCount => models::Aggregate::StarCount {},
                },
            )
        }))
    });
    let fields = fields.map(|fields| {
        HashMap::from_iter(fields.into_iter().map(|(key, field)| {
            (
                key,
                match field {
                    Field::Column {
                        column,
                        column_type,
                    } => models::Field::Column {
                        column,
                        arguments: HashMap::new(),
                    },
                    Field::Relationship {
                        query,
                        relationship,
                    } => models::Field::Relationship {
                        query: Box::new(map_query(
                            query,
                            get_target_table(&table, &relationship, table_relationships),
                            table_relationships,
                        )),
                        relationship: format!("{}.{}", table, relationship),
                        arguments: HashMap::new(),
                    },
                },
            )
        }))
    });
    models::Query {
        aggregates,
        fields,
        limit,
        offset,
        order_by,
        predicate: selection
            .map(|selection| map_expression(selection, &table, table_relationships)),
    }
}

fn map_order_by_path(
    path: Vec<String>,
    relations: IndexMap<String, OrderByRelation>,
    table: String,
    table_relationships: &Vec<TableRelationships>,
) -> Vec<models::PathElementWithPredicate> {
    let mut mapped_path: Vec<models::PathElementWithPredicate> = vec![];

    let mut relations = relations;
    let mut source_table = table;
    for segment in path {
        let relation = relations
            .get(&segment)
            .expect("order by path segments should reference valid relationships");

        let target_table = get_target_table(&source_table, &segment, table_relationships);

        mapped_path.push(models::PathElementWithPredicate {
            relationship: format!("{}.{}", source_table, segment),
            arguments: HashMap::new(),
            predicate: if let Some(selection) = &relation.selection {
                Box::new(map_expression(
                    selection.to_owned(),
                    &target_table,
                    table_relationships,
                ))
            } else {
                // hack: predicate is not optional, so default to empty "And" expression, which evaluates to true.
                Box::new(models::Expression::And {
                    expressions: vec![],
                })
            },
        });

        source_table = target_table;
        relations = relation.subrelations.to_owned();
    }

    mapped_path
}
fn map_order_by_path_without_predicate(
    path: Vec<String>,
    relations: IndexMap<String, OrderByRelation>,
    table: String,
    table_relationships: &Vec<TableRelationships>,
) -> Vec<models::PathElement> {
    let mut mapped_path: Vec<models::PathElement> = vec![];

    let mut relations = relations;
    let mut source_table = table;
    for segment in path {
        let relation = relations
            .get(&segment)
            .expect("order by path segments should reference valid relationships");

        let target_table = get_target_table(&source_table, &segment, table_relationships);

        mapped_path.push(models::PathElement {
            relationship: format!("{}.{}", source_table, segment),
            arguments: HashMap::new(),
        });

        source_table = target_table;
        relations = relation.subrelations.to_owned();
    }

    mapped_path
}

fn get_target_table(
    source_table: &str,
    relationship: &str,
    table_relationships: &Vec<TableRelationships>,
) -> String {
    let source_table = table_relationships
        .iter()
        .find(|table_relationships| {
            table_relationships
                .source_table
                .first()
                .expect("tables names have at least one string")
                == source_table
        })
        .expect("all relationships should exist in table_relationships");

    let relationship = source_table
        .relationships
        .get(relationship)
        .expect("relationships should exist in table relationships");

    table_name(relationship.target_table.to_owned())
}

fn map_expression(
    expression: Expression,
    table: &str,
    table_relationships: &Vec<TableRelationships>,
) -> models::Expression {
    match expression {
        Expression::And { expressions } => models::Expression::And {
            expressions: expressions
                .into_iter()
                .map(|expression| map_expression(expression, table, table_relationships))
                .collect(),
        },
        Expression::Or { expressions } => models::Expression::Or {
            expressions: expressions
                .into_iter()
                .map(|expression| map_expression(expression, table, table_relationships))
                .collect(),
        },
        Expression::Not { expression } => models::Expression::Not {
            expression: Box::new(map_expression(*expression, table, table_relationships)),
        },
        Expression::UnaryComparisonOperator { column, operator } => {
            models::Expression::UnaryComparisonOperator {
                column: Box::new(map_comparison_column(
                    column,
                    table.to_string(),
                    table_relationships,
                )),
                operator: match operator.as_str() {
                    "is_null" => Box::new(models::UnaryComparisonOperator::IsNull),
                    _ => panic!("Unsupported unary comparison operator"),
                },
            }
        }
        Expression::BinaryComparisonOperator {
            column,
            operator,
            value,
        } => models::Expression::BinaryComparisonOperator {
            column: Box::new(map_comparison_column(
                column,
                table.to_string(),
                table_relationships,
            )),
            operator: match operator.as_str() {
                "equal" => Box::new(models::BinaryComparisonOperator::Equal),
                _ => Box::new(models::BinaryComparisonOperator::Other { name: operator }),
            },
            value: Box::new(match value {
                ComparisonValue::ScalarValueComparison { value, value_type } => {
                    models::ComparisonValue::Scalar { value }
                }
                ComparisonValue::AnotherColumnComparison { column } => {
                    models::ComparisonValue::Column {
                        column: Box::new(map_comparison_column(
                            column,
                            table.to_string(),
                            table_relationships,
                        )),
                    }
                }
            }),
        },
        Expression::BinaryArrayComparisonOperator {
            column,
            operator,
            value_type,
            values,
        } => models::Expression::BinaryArrayComparisonOperator {
            column: Box::new(map_comparison_column(
                column,
                table.to_string(),
                table_relationships,
            )),
            operator: match operator.as_str() {
                "in" => Box::new(models::BinaryArrayComparisonOperator::In),
                _ => panic!("Unsupported binary array comparison operator"),
            },
            values: values
                .into_iter()
                .map(|value| models::ComparisonValue::Scalar { value })
                .collect(),
        },
        Expression::Exists {
            in_table,
            selection,
        } => models::Expression::Exists {
            in_table: match in_table {
                ExistsInTable::UnrelatedTable { table } => {
                    Box::new(models::ExistsInTable::Unrelated {
                        table: table_name(table),
                        arguments: HashMap::new(),
                    })
                }
                ExistsInTable::RelatedTable { relationship } => {
                    Box::new(models::ExistsInTable::Related {
                        relationship: format!("{}.{}", table, relationship),
                        arguments: HashMap::new(),
                    })
                }
            },
            predicate: Box::new(map_expression(*selection, table, table_relationships)),
        },
    }
}

fn map_comparison_column(
    column: ComparisonColumn,
    table: String,
    table_relationships: &Vec<TableRelationships>,
) -> models::ComparisonTarget {
    match column.path {
        Some(path) if !path.is_empty() => {
            let mut mapped_path = vec![];

            let mut source_table = table;
            for path_segment in path {
                let target_table =
                    get_target_table(&source_table, &path_segment, table_relationships);
                mapped_path.push(models::PathElement {
                    relationship: format!("{}.{}", source_table, path_segment),
                    arguments: HashMap::new(),
                });
                source_table = target_table;
            }

            models::ComparisonTarget::Column {
                name: column.name,
                path: mapped_path,
            }
        }
        _ => models::ComparisonTarget::RootTableColumn { name: column.name },
    }
}

fn map_response(response: models::QueryResponse, is_foreach: bool) -> QueryResponse {
    if is_foreach {
        QueryResponse {
            aggregates: None,
            rows: Some(
                response
                    .0
                    .into_iter()
                    .map(|row| {
                        let value = row_set_as_json(row);

                        IndexMap::from_iter(
                            vec![(
                                "query".to_string(),
                                Some(RowFieldValue::ColumnFieldValue(value)),
                            )]
                            .into_iter(),
                        )
                    })
                    .collect(),
            ),
        }
    } else {
        let (aggregates, rows) = if let Some(row) = response.0.into_iter().next() {
            let models::RowSet { rows, aggregates } = row;
            let aggregates = aggregates.map(|aggregates| {
                let mut aggregates_map = IndexMap::new();

                for (key, value) in aggregates {
                    aggregates_map.insert(key, value);
                }

                aggregates_map
            });

            let rows = rows.map(|rows| {
                rows.into_iter()
                    .map(|row| {
                        let mut row_map = IndexMap::new();

                        for (key, value) in row {
                            let value = match value {
                                models::RowFieldValue::Relationship { rows } => {
                                    row_set_as_json(rows)
                                }
                                models::RowFieldValue::Column { value } => value,
                            };
                            row_map.insert(key, Some(RowFieldValue::ColumnFieldValue(value)));
                        }

                        row_map
                    })
                    .collect()
            });

            (aggregates, rows)
        } else {
            (None, None)
        };
        QueryResponse { aggregates, rows }
    }
}

fn row_set_as_json(row: models::RowSet) -> serde_json::Value {
    let mut row_object = serde_json::Map::new();

    if let Some(aggregates) = row.aggregates {
        let mut aggregates_object = serde_json::Map::new();

        for (key, value) in aggregates {
            aggregates_object.insert(key, value);
        }

        row_object.insert(
            "aggregates".to_string(),
            serde_json::Value::Object(aggregates_object),
        );
    }
    if let Some(rows) = row.rows {
        let rows_array = rows
            .into_iter()
            .map(|row| {
                let mut row_object = serde_json::Map::new();

                for (key, value) in row {
                    row_object.insert(
                        key,
                        match value {
                            models::RowFieldValue::Relationship { rows } => row_set_as_json(rows),
                            models::RowFieldValue::Column { value } => value,
                        },
                    );
                }

                serde_json::Value::Object(row_object)
            })
            .collect();

        row_object.insert("rows".to_string(), serde_json::Value::Array(rows_array));
    }

    serde_json::Value::Object(row_object)
}
