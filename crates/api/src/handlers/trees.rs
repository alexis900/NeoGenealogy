use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    error::ApiError,
    pagination::{Paginated, PaginationMeta},
    state::AppState,
};

#[derive(Deserialize)]
pub struct TreeListParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Serialize)]
pub struct TreeSummary {
    pub id: i64,
    pub name: String,
    pub source_filename: Option<String>,
    pub gedcom_version: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub persons: i64,
    pub families: i64,
    pub findings: i64,
    pub research_opportunities: i64,
}

pub async fn list_trees(
    State(state): State<AppState>,
    Query(params): Query<TreeListParams>,
) -> Result<Json<Paginated<TreeSummary>>, ApiError> {
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    if !(0..=100).contains(&limit) {
        return Err(ApiError::bad_request("INVALID_LIMIT", "limit 0..100"));
    }
    if offset < 0 {
        return Err(ApiError::bad_request("INVALID_OFFSET", "offset >=0"));
    }
    let trees = state.storage.list_trees(Some(limit), Some(offset)).await?;
    let total: i64 = state
        .storage
        .count_trees()
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let mut items = Vec::new();
    for t in trees {
        // avoid N+1 by using count per tree but we already need per tree; using single query per tree is okay for small N; batch would be better
        let (persons, families, _, _, findings, opps) = state.storage.count(t.id).await?;
        items.push(TreeSummary {
            id: t.id,
            name: t.name,
            source_filename: t.source_filename,
            gedcom_version: t.gedcom_version,
            created_at: t.created_at,
            updated_at: t.updated_at,
            persons,
            families,
            findings,
            research_opportunities: opps,
        });
    }
    Ok(Json(Paginated {
        items,
        pagination: PaginationMeta {
            limit,
            offset,
            total,
        },
    }))
}

pub async fn get_tree(
    State(state): State<AppState>,
    Path(tree_id): Path<i64>,
) -> Result<Json<TreeSummary>, ApiError> {
    if tree_id <= 0 {
        return Err(ApiError::bad_request(
            "INVALID_TREE_ID",
            "tree_id must be >0",
        ));
    }
    let tree = state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    let (persons, families, _, _, findings, opps) = state.storage.count(tree.id).await?;
    Ok(Json(TreeSummary {
        id: tree.id,
        name: tree.name,
        source_filename: tree.source_filename,
        gedcom_version: tree.gedcom_version,
        created_at: tree.created_at,
        updated_at: tree.updated_at,
        persons,
        families,
        findings,
        research_opportunities: opps,
    }))
}
