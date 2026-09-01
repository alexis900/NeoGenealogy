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
pub struct FindingsParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub severity: Option<String>,
    #[serde(rename = "type")]
    pub finding_type: Option<String>,
    pub person_id: Option<i64>,
}

fn validate_severity(s: &str) -> Result<(), ApiError> {
    let allowed = ["low", "info", "medium", "warning", "high", "critical"];
    if !allowed.contains(&s.to_lowercase().as_str()) {
        return Err(ApiError::bad_request(
            "INVALID_SEVERITY",
            format!("severity must be one of {}", allowed.join(",")),
        ));
    }
    Ok(())
}

pub async fn list_findings(
    State(state): State<AppState>,
    Path(tree_id): Path<i64>,
    Query(params): Query<FindingsParams>,
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

    if let Some(ref sev) = params.severity {
        validate_severity(sev)?;
    }

    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    if !(0..=100).contains(&limit) {
        return Err(ApiError::bad_request("INVALID_LIMIT", "limit 0..100"));
    }
    if offset < 0 {
        return Err(ApiError::bad_request("INVALID_OFFSET", "offset >=0"));
    }
    if let Some(pid) = params.person_id {
        if pid <= 0 {
            return Err(ApiError::bad_request("INVALID_PERSON_ID", "person_id >0"));
        }
    }

    let (rows, total) = state
        .storage
        .list_findings_filtered(
            tree_id,
            params.severity.as_deref(),
            params.finding_type.as_deref(),
            params.person_id,
            limit,
            offset,
        )
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    let items: Vec<serde_json::Value> = rows.into_iter().map(|r| {
        serde_json::json!({
            "id": r.id,
            "tree_id": r.tree_id,
            "analysis_run_id": r.analysis_run_id,
            "person_id": r.person_id,
            "related_person_id": r.related_person_id,
            "finding_type": r.finding_type,
            "severity": r.severity,
            "confidence": r.confidence,
            "message": r.message,
            "evidence": r.evidence.and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok()),
            "created_at": r.created_at
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
