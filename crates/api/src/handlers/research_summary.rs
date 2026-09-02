use crate::{error::ApiError, state::AppState};
use axum::{
    extract::{Path, State},
    Json,
};

pub async fn get_research_summary(
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
    let summary = state.storage.research_summary(tree_id).await?;
    Ok(Json(summary))
}
