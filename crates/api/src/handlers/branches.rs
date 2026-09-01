use axum::{
    extract::{Path, State},
    Json,
};

use crate::{error::ApiError, state::AppState};

pub async fn list_branches(
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
    let branches = state.storage.get_branches(tree_id).await?;
    let items: Vec<serde_json::Value> = branches
        .into_iter()
        .map(|b| {
            serde_json::json!({
                "id": b.id,
                "tree_id": b.tree_id,
                "analysis_run_id": b.analysis_run_id,
                "name": b.name,
                "branch": b.name,
                "branch_score": b.score,
                "score": b.score,
                "opportunity_count": b.opportunity_count,
                "high_priority_count": b.high_priority_count,
                "deepest_generation": b.deepest_generation,
                "source_coverage": b.source_coverage
            })
        })
        .collect();
    Ok(Json(serde_json::json!({ "items": items })))
}
