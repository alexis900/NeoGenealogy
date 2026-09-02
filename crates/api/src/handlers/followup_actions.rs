use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::{
    error::ApiError,
    pagination::{Paginated, PaginationMeta},
    state::AppState,
};

fn to_json(row: neogenealogy_storage::models::ResearchFollowupActionRow) -> serde_json::Value {
    serde_json::json!({
        "id": row.id,
        "tree_id": row.tree_id,
        "task_id": row.task_id,
        "outcome_id": row.outcome_id,
        "followup_code": row.followup_code,
        "status": row.status,
        "notes": row.notes,
        "created_at": row.created_at,
        "updated_at": row.updated_at,
        "completed_at": row.completed_at
    })
}

fn validate_status(s: &str) -> Result<(), ApiError> {
    let allowed = ["OPEN", "COMPLETED", "SKIPPED"];
    if !allowed.contains(&s) {
        return Err(ApiError::bad_request(
            "INVALID_FOLLOWUP_ACTION_STATUS",
            format!("status must be one of {}", allowed.join(",")),
        ));
    }
    Ok(())
}
fn validate_followup_code(s: &str) -> Result<(), ApiError> {
    let allowed = [
        "ADD_SUPPORTING_EVIDENCE",
        "ADD_CITATION",
        "REVIEW_CONTRADICTION",
        "ADD_SECOND_SUPPORTING_EVIDENCE",
        "REVIEW_SOURCE_COVERAGE",
    ];
    if !allowed.contains(&s) {
        return Err(ApiError::bad_request(
            "INVALID_FOLLOWUP_CODE",
            format!("followup_code must be one of {}", allowed.join(",")),
        ));
    }
    Ok(())
}

#[derive(Deserialize)]
pub struct ListParams {
    pub task_id: Option<i64>,
    pub outcome_id: Option<i64>,
    pub status: Option<String>,
    pub followup_code: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize, Serialize)]
pub struct CreateBody {
    pub followup_code: String,
    pub notes: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateBody {
    pub status: Option<String>,
    pub notes: Option<String>,
}

pub async fn list_actions(
    State(state): State<AppState>,
    Path(tree_id): Path<i64>,
    Query(params): Query<ListParams>,
) -> Result<Json<Paginated<serde_json::Value>>, ApiError> {
    if tree_id <= 0 {
        return Err(ApiError::bad_request(
            "INVALID_TREE_ID",
            "tree_id must be >0",
        ));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} not found"))
    })?;
    if let Some(ref s) = params.status {
        validate_status(&s.to_uppercase())?;
    }
    if let Some(ref c) = params.followup_code {
        validate_followup_code(&c.to_uppercase())?;
    }
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    if !(0..=100).contains(&limit) {
        return Err(ApiError::bad_request("INVALID_LIMIT", "limit 0..100"));
    }
    if offset < 0 {
        return Err(ApiError::bad_request("INVALID_OFFSET", "offset >=0"));
    }
    let status = params.status.as_deref().map(|s| s.to_uppercase());
    let code = params.followup_code.as_deref().map(|s| s.to_uppercase());
    let (rows, total) = state
        .storage
        .list_followup_actions(
            tree_id,
            params.task_id,
            params.outcome_id,
            status.as_deref(),
            code.as_deref(),
            limit,
            offset,
        )
        .await?;
    let items = rows.into_iter().map(to_json).collect();
    Ok(Json(Paginated {
        items,
        pagination: PaginationMeta {
            limit,
            offset,
            total,
        },
    }))
}

pub async fn get_action(
    State(state): State<AppState>,
    Path((tree_id, action_id)): Path<(i64, i64)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if tree_id <= 0 || action_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} not found"))
    })?;
    let row = state
        .storage
        .get_followup_action(action_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "FOLLOWUP_ACTION_NOT_FOUND",
                format!("Followup action {action_id} not found"),
            )
        })?;
    if row.tree_id != tree_id {
        return Err(ApiError::not_found(
            "FOLLOWUP_ACTION_NOT_FOUND",
            format!("Action {action_id} not in tree {tree_id}"),
        ));
    }
    Ok(Json(to_json(row)))
}

pub async fn create_action(
    State(state): State<AppState>,
    Path((tree_id, outcome_id)): Path<(i64, i64)>,
    Json(body): Json<CreateBody>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    if tree_id <= 0 || outcome_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} not found"))
    })?;
    let code = body.followup_code.to_uppercase();
    validate_followup_code(&code)?;
    let outcome = state
        .storage
        .get_research_outcome(outcome_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "RESEARCH_OUTCOME_NOT_FOUND",
                format!("Outcome {outcome_id} not found"),
            )
        })?;
    if outcome.tree_id != tree_id {
        return Err(ApiError::not_found(
            "RESEARCH_OUTCOME_NOT_FOUND",
            format!("Outcome {outcome_id} not in tree {tree_id}"),
        ));
    }
    // validate followup is active
    let active = state
        .storage
        .get_outcome_followups(outcome_id)
        .await
        .unwrap_or_default();
    if !active.iter().any(|f| f.code == code) {
        return Err(ApiError {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code: "FOLLOWUP_NOT_ACTIVE",
            message: "The requested research follow-up is not currently active.".into(),
        });
    }
    let row = state
        .storage
        .create_followup_action(
            tree_id,
            outcome.task_id,
            outcome_id,
            &code,
            body.notes.as_deref(),
        )
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok((StatusCode::CREATED, Json(to_json(row))))
}

pub async fn update_action(
    State(state): State<AppState>,
    Path((tree_id, action_id)): Path<(i64, i64)>,
    Json(body): Json<UpdateBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if tree_id <= 0 || action_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} not found"))
    })?;
    let existing = state
        .storage
        .get_followup_action(action_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "FOLLOWUP_ACTION_NOT_FOUND",
                format!("Action {action_id} not found"),
            )
        })?;
    if existing.tree_id != tree_id {
        return Err(ApiError::not_found(
            "FOLLOWUP_ACTION_NOT_FOUND",
            format!("Action {action_id} not in tree {tree_id}"),
        ));
    }
    if let Some(ref s) = body.status {
        validate_status(&s.to_uppercase())?;
    }
    let status = body.status.as_deref().map(|s| s.to_uppercase());
    // notes: we need to distinguish between not provided vs explicit null
    // body.notes is Option<String>: None = not provided, Some(val) = provided (could be empty?)
    // For our storage, notes param is Option<Option<&str>>: outer None = keep, inner Some -> set value, inner None -> set null
    // But UpdateBody.notes being Option<String> can't represent explicit null vs absent. We treat Some("") as set to empty? We'll treat provided notes even if None? Actually we need to allow setting to null.
    // We interpret: if body.notes is Some, set to that value (even empty string => set); if we want to set null, client would send null which deserializes to None? But then we can't distinguish.
    // Simpler: if body.notes.is_some(), set to Some(value), otherwise keep existing.
    // To allow clearing, client can send notes:"" and we handle? But spec doesn't mention clearing. We'll just handle as optional set.
    let notes_opt: Option<Option<&str>> = if body.notes.is_some() {
        Some(body.notes.as_deref())
    } else {
        None
    };
    let row = state
        .storage
        .update_followup_action(action_id, status.as_deref(), notes_opt)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(to_json(row)))
}

pub async fn delete_action(
    State(state): State<AppState>,
    Path((tree_id, action_id)): Path<(i64, i64)>,
) -> Result<StatusCode, ApiError> {
    if tree_id <= 0 || action_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} not found"))
    })?;
    let existing = state
        .storage
        .get_followup_action(action_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "FOLLOWUP_ACTION_NOT_FOUND",
                format!("Action {action_id} not found"),
            )
        })?;
    if existing.tree_id != tree_id {
        return Err(ApiError::not_found(
            "FOLLOWUP_ACTION_NOT_FOUND",
            format!("Action {action_id} not in tree {tree_id}"),
        ));
    }
    state.storage.delete_followup_action(action_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_task_actions(
    State(state): State<AppState>,
    Path((tree_id, task_id)): Path<(i64, i64)>,
    Query(params): Query<ListParams>,
) -> Result<Json<Paginated<serde_json::Value>>, ApiError> {
    if tree_id <= 0 || task_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} not found"))
    })?;
    let task = state
        .storage
        .get_research_task(task_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "RESEARCH_TASK_NOT_FOUND",
                format!("Task {task_id} not found"),
            )
        })?;
    if task.tree_id != tree_id {
        return Err(ApiError::not_found(
            "RESEARCH_TASK_NOT_FOUND",
            format!("Task {task_id} not in tree {tree_id}"),
        ));
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
        .list_task_followup_actions(task_id, limit, offset)
        .await?;
    let items = rows.into_iter().map(to_json).collect();
    Ok(Json(Paginated {
        items,
        pagination: PaginationMeta {
            limit,
            offset,
            total,
        },
    }))
}

pub async fn list_outcome_actions(
    State(state): State<AppState>,
    Path((tree_id, outcome_id)): Path<(i64, i64)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if tree_id <= 0 || outcome_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} not found"))
    })?;
    let outcome = state
        .storage
        .get_research_outcome(outcome_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "RESEARCH_OUTCOME_NOT_FOUND",
                format!("Outcome {outcome_id} not found"),
            )
        })?;
    if outcome.tree_id != tree_id {
        return Err(ApiError::not_found(
            "RESEARCH_OUTCOME_NOT_FOUND",
            format!("Outcome {outcome_id} not in tree {tree_id}"),
        ));
    }
    let rows = state
        .storage
        .list_outcome_followup_actions(outcome_id)
        .await?;
    let items: Vec<serde_json::Value> = rows.into_iter().map(to_json).collect();
    Ok(Json(serde_json::json!({"items": items})))
}
