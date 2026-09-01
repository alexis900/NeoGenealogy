use axum::{
    extract::{Path, State},
    Json,
};

use crate::{error::ApiError, state::AppState};

pub async fn get_coverage(
    State(state): State<AppState>,
    Path(tree_id): Path<i64>,
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
    let cov = state
        .storage
        .get_source_coverage(tree_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "COVERAGE_NOT_FOUND",
                format!("Coverage for tree {tree_id} not found"),
            )
        })?;
    Ok(Json(serde_json::json!({
        "tree_id": cov.tree_id,
        "analysis_run_id": cov.analysis_run_id,
        "birth": cov.birth,
        "marriage": cov.marriage,
        "death": cov.death,
        "other_events": cov.other_events,
        "overall": cov.overall
    })))
}
