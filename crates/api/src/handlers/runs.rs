use axum::{
    extract::{Path, State},
    Json,
};

use crate::{error::ApiError, state::AppState};

pub async fn list_runs(
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
    let runs = state.storage.get_analysis_runs(tree_id).await?;
    let items: Vec<serde_json::Value> = runs
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "tree_id": r.tree_id,
                "started_at": r.started_at,
                "completed_at": r.completed_at,
                "engine_version": r.engine_version,
                "status": r.status,
                "error_message": r.error_message
            })
        })
        .collect();
    Ok(Json(serde_json::json!({ "items": items })))
}

pub async fn get_run(
    State(state): State<AppState>,
    Path((tree_id, run_id)): Path<(i64, i64)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if tree_id <= 0 || run_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    let runs = state.storage.get_analysis_runs(tree_id).await?;
    let run = runs.into_iter().find(|r| r.id == run_id).ok_or_else(|| {
        ApiError::not_found(
            "RUN_NOT_FOUND",
            format!("Run {run_id} not found for tree {tree_id}"),
        )
    })?;
    Ok(Json(serde_json::json!({
        "id": run.id,
        "tree_id": run.tree_id,
        "started_at": run.started_at,
        "completed_at": run.completed_at,
        "engine_version": run.engine_version,
        "status": run.status
    })))
}
