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
pub struct OppParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub priority: Option<String>,
    pub min_score: Option<i64>,
    pub sort: Option<String>,
}

fn validate_priority(s: &str) -> Result<(), ApiError> {
    let allowed = ["low", "info", "medium", "warning", "high", "critical"];
    if !allowed.contains(&s.to_lowercase().as_str()) {
        return Err(ApiError::bad_request(
            "INVALID_PRIORITY",
            format!("priority must be one of {}", allowed.join(",")),
        ));
    }
    Ok(())
}
fn validate_sort(s: &str) -> Result<(), ApiError> {
    let allowed = ["score", "priority", "confidence"];
    if !allowed.contains(&s.to_lowercase().as_str()) {
        return Err(ApiError::bad_request(
            "INVALID_SORT",
            format!("sort must be one of {}", allowed.join(",")),
        ));
    }
    Ok(())
}

pub async fn list_opportunities(
    State(state): State<AppState>,
    Path(tree_id): Path<i64>,
    Query(params): Query<OppParams>,
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

    if let Some(ref p) = params.priority {
        validate_priority(p)?;
    }
    if let Some(ref s) = params.sort {
        validate_sort(s)?;
    }
    if let Some(ms) = params.min_score {
        if !(0..=100).contains(&ms) {
            return Err(ApiError::bad_request(
                "INVALID_MIN_SCORE",
                "min_score 0..100",
            ));
        }
    }
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    if !(0..=100).contains(&limit) {
        return Err(ApiError::bad_request("INVALID_LIMIT", "limit 0..100"));
    }
    if offset < 0 {
        return Err(ApiError::bad_request("INVALID_OFFSET", "offset >=0"));
    }

    let (rows, total) = state
        .storage
        .list_opportunities_filtered(
            tree_id,
            params.priority.as_deref(),
            params.min_score,
            params.sort.as_deref(),
            limit,
            offset,
        )
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    let items: Vec<serde_json::Value> = rows.into_iter().map(|r| {
        serde_json::json!({
            "id": r.id,
            "tree_id": r.tree_id,
            "person_id": r.person_id,
            "priority": r.priority,
            "score": r.score,
            "confidence": r.confidence,
            "researchability": r.researchability,
            "why": r.why,
            "what": r.what.and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok()),
            "potential_sources": r.potential_sources.and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok()),
            "breakdown": r.breakdown.and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok()),
            "missing_information": r.missing_information.and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok()),
            "reasons": r.reasons.and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok()),
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

pub async fn top_opportunities(
    State(state): State<AppState>,
    Path(tree_id): Path<i64>,
    Query(params): Query<OppParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if tree_id <= 0 {
        return Err(ApiError::bad_request(
            "INVALID_TREE_ID",
            "tree_id must be >0",
        ));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    let limit = params.limit.unwrap_or(10).min(100);
    let rows = state
        .storage
        .get_top_research_opportunities(tree_id, params.priority.as_deref(), limit)
        .await?;
    let items: Vec<serde_json::Value> = rows.into_iter().map(|r| {
        serde_json::json!({
            "id": r.id,
            "tree_id": r.tree_id,
            "person_id": r.person_id,
            "priority": r.priority,
            "score": r.score,
            "confidence": r.confidence,
            "researchability": r.researchability,
            "why": r.why,
            "breakdown": r.breakdown.and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok()),
        })
    }).collect();
    Ok(Json(serde_json::json!({ "items": items })))
}
