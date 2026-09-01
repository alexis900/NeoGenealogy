use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

use crate::{
    error::ApiError,
    pagination::{Paginated, PaginationMeta},
    state::AppState,
};

#[derive(Deserialize)]
pub struct PersonListParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_persons(
    State(state): State<AppState>,
    Path(tree_id): Path<i64>,
    Query(params): Query<PersonListParams>,
) -> Result<Json<Paginated<serde_json::Value>>, ApiError> {
    if tree_id <= 0 {
        return Err(ApiError::bad_request(
            "INVALID_TREE_ID",
            "tree_id must be >0",
        ));
    }
    // validate tree exists
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;

    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    if !(0..=100).contains(&limit) {
        return Err(ApiError::bad_request(
            "INVALID_LIMIT",
            "limit must be between 0 and 100",
        ));
    }
    if offset < 0 {
        return Err(ApiError::bad_request(
            "INVALID_OFFSET",
            "offset must be >=0",
        ));
    }
    let persons = state
        .storage
        .list_persons(tree_id, Some(limit), Some(offset))
        .await?;
    let total: i64 = state
        .storage
        .count_persons(tree_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let items: Vec<serde_json::Value> = persons.into_iter().map(|p| {
        serde_json::json!({
            "id": p.id,
            "tree_id": p.tree_id,
            "gedcom_id": p.gedcom_id,
            "given_name": p.given_name,
            "surname": p.surname,
            "display_name": p.display_name,
            "sex": p.sex,
            "raw_name": p.raw_name,
            "birth_date_original": p.birth_date_original,
            "birth_date_precision": p.birth_date_precision,
            "birth_date_year": p.birth_date_year,
            "birth_place": p.birth_place,
            "death_date_original": p.death_date_original,
            "death_place": p.death_place,
            "occupation": p.occupation,
            "raw_tags": p.raw_tags.and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
        })
    }).collect();

    Ok(Json(Paginated {
        items,
        pagination: PaginationMeta {
            limit,
            offset,
            total,
        },
    }))
}

pub async fn get_person(
    State(state): State<AppState>,
    Path((tree_id, person_id)): Path<(i64, i64)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if tree_id <= 0 || person_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    let person = state.storage.get_person(person_id).await?.ok_or_else(|| {
        ApiError::not_found(
            "PERSON_NOT_FOUND",
            format!("Person {person_id} was not found"),
        )
    })?;
    if person.tree_id != tree_id {
        return Err(ApiError::not_found(
            "PERSON_NOT_FOUND",
            format!("Person {person_id} not in tree {tree_id}"),
        ));
    }
    Ok(Json(serde_json::json!({
        "id": person.id,
        "tree_id": person.tree_id,
        "gedcom_id": person.gedcom_id,
        "given_name": person.given_name,
        "surname": person.surname,
        "display_name": person.display_name,
        "sex": person.sex,
        "raw_name": person.raw_name,
        "birth_date_original": person.birth_date_original,
        "birth_date_precision": person.birth_date_precision,
        "birth_date_year": person.birth_date_year,
        "birth_place": person.birth_place,
        "death_date_original": person.death_date_original,
        "death_place": person.death_place,
        "occupation": person.occupation,
        "raw_tags": person.raw_tags.and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok())
    })))
}
