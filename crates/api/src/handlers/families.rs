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
pub struct FamilyListParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_families(
    State(state): State<AppState>,
    Path(tree_id): Path<i64>,
    Query(params): Query<FamilyListParams>,
) -> Result<Json<Paginated<serde_json::Value>>, ApiError> {
    if tree_id <= 0 {
        return Err(ApiError::bad_request(
            "INVALID_TREE_ID",
            "tree_id must be >0",
        ));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    if !(0..=100).contains(&limit) {
        return Err(ApiError::bad_request("INVALID_LIMIT", "limit 0..100"));
    }
    if offset < 0 {
        return Err(ApiError::bad_request("INVALID_OFFSET", "offset >=0"));
    }
    let families = state
        .storage
        .list_families(tree_id, Some(limit), Some(offset))
        .await?;
    let total: i64 = state
        .storage
        .count_families(tree_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let family_ids: Vec<i64> = families.iter().map(|f| f.id).collect();
    let members = state
        .storage
        .get_family_members(&family_ids)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    use std::collections::HashMap;
    let mut members_map: HashMap<i64, Vec<&neogenealogy_storage::models::FamilyMemberRow>> =
        HashMap::new();
    for m in &members {
        members_map.entry(m.family_id).or_default().push(m);
    }
    let items: Vec<serde_json::Value> = families.into_iter().map(|f| {
        let mems = members_map.get(&f.id).cloned().unwrap_or_default();
        let husbands: Vec<i64> = mems.iter().filter(|m| m.role=="husband").map(|m| m.person_id).collect();
        let wives: Vec<i64> = mems.iter().filter(|m| m.role=="wife").map(|m| m.person_id).collect();
        let children: Vec<i64> = mems.iter().filter(|m| m.role=="child").map(|m| m.person_id).collect();
        serde_json::json!({
            "id": f.id,
            "tree_id": f.tree_id,
            "gedcom_id": f.gedcom_id,
            "raw_tags": f.raw_tags.and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok()),
            "members": {
                "husband": husbands,
                "wife": wives,
                "children": children
            }
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

pub async fn get_family(
    State(state): State<AppState>,
    Path((tree_id, family_id)): Path<(i64, i64)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if tree_id <= 0 || family_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    let fam = state.storage.get_family(family_id).await?.ok_or_else(|| {
        ApiError::not_found(
            "FAMILY_NOT_FOUND",
            format!("Family {family_id} was not found"),
        )
    })?;
    if fam.tree_id != tree_id {
        return Err(ApiError::not_found(
            "FAMILY_NOT_FOUND",
            format!("Family {family_id} not in tree {tree_id}"),
        ));
    }
    let members = state
        .storage
        .get_family_members_for_family(family_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let husbands: Vec<i64> = members
        .iter()
        .filter(|m| m.role == "husband")
        .map(|m| m.person_id)
        .collect();
    let wives: Vec<i64> = members
        .iter()
        .filter(|m| m.role == "wife")
        .map(|m| m.person_id)
        .collect();
    let children: Vec<i64> = members
        .iter()
        .filter(|m| m.role == "child")
        .map(|m| m.person_id)
        .collect();
    Ok(Json(serde_json::json!({
        "id": fam.id,
        "tree_id": fam.tree_id,
        "gedcom_id": fam.gedcom_id,
        "raw_tags": fam.raw_tags.and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok()),
        "members": { "husband": husbands, "wife": wives, "children": children }
    })))
}
