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

#[derive(Deserialize)]
pub struct OutcomeListParams {
    pub r#type: Option<String>,
    pub task_id: Option<i64>,
    pub person_id: Option<i64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize, Serialize)]
pub struct CreateOutcomeBody {
    pub r#type: String,
    pub summary: String,
    pub details: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateOutcomeBody {
    pub r#type: Option<String>,
    pub summary: Option<String>,
    pub details: Option<String>,
}

fn validate_type(s: &str) -> Result<(), ApiError> {
    let allowed = [
        "CONFIRMED",
        "FALSE_LEAD",
        "INCONCLUSIVE",
        "NEW_LEAD",
        "NO_EVIDENCE",
    ];
    if !allowed.contains(&s) {
        return Err(ApiError::bad_request(
            "INVALID_RESEARCH_OUTCOME_TYPE",
            format!("type must be one of {}", allowed.join(",")),
        ));
    }
    Ok(())
}

fn to_json(row: neogenealogy_storage::models::ResearchOutcomeRow) -> serde_json::Value {
    serde_json::json!({
        "id": row.id,
        "tree_id": row.tree_id,
        "task_id": row.task_id,
        "type": row.r#type,
        "summary": row.summary,
        "details": row.details,
        "created_at": row.created_at,
        "updated_at": row.updated_at
    })
}

pub async fn list_outcomes(
    State(state): State<AppState>,
    Path(tree_id): Path<i64>,
    Query(params): Query<OutcomeListParams>,
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
    if let Some(ref t) = params.r#type {
        validate_type(&t.to_uppercase())?;
    }
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    if !(0..=100).contains(&limit) {
        return Err(ApiError::bad_request("INVALID_LIMIT", "limit 0..100"));
    }
    if offset < 0 {
        return Err(ApiError::bad_request("INVALID_OFFSET", "offset >=0"));
    }
    let ttype = params.r#type.as_deref().map(|s| s.to_uppercase());
    let (rows, total) = if params.person_id.is_some() {
        state
            .storage
            .list_research_outcomes_with_person(
                tree_id,
                ttype.as_deref(),
                params.person_id,
                limit,
                offset,
            )
            .await?
    } else {
        state
            .storage
            .list_research_outcomes(tree_id, ttype.as_deref(), params.task_id, limit, offset)
            .await?
    };
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

pub async fn get_outcome(
    State(state): State<AppState>,
    Path((tree_id, outcome_id)): Path<(i64, i64)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if tree_id <= 0 || outcome_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    let row = state
        .storage
        .get_research_outcome(outcome_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "RESEARCH_OUTCOME_NOT_FOUND",
                format!("Outcome {outcome_id} not found"),
            )
        })?;
    if row.tree_id != tree_id {
        return Err(ApiError::not_found(
            "RESEARCH_OUTCOME_NOT_FOUND",
            format!("Outcome {outcome_id} not in tree {tree_id}"),
        ));
    }
    Ok(Json(to_json(row)))
}

pub async fn create_outcome(
    State(state): State<AppState>,
    Path((tree_id, task_id)): Path<(i64, i64)>,
    Json(body): Json<CreateOutcomeBody>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    if tree_id <= 0 || task_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    validate_type(&body.r#type.to_uppercase())?;
    if body.summary.trim().is_empty() {
        return Err(ApiError::bad_request(
            "INVALID_SUMMARY",
            "summary must not be empty",
        ));
    }
    let row = state
        .storage
        .create_research_outcome(
            tree_id,
            task_id,
            &body.r#type.to_uppercase(),
            &body.summary,
            body.details.as_deref(),
        )
        .await
        .map_err(|e| match e {
            neogenealogy_storage::StorageError::NotFound(msg) => {
                if msg.contains("task") {
                    ApiError::not_found("RESEARCH_TASK_NOT_FOUND", msg)
                } else {
                    ApiError::not_found("NOT_FOUND", msg)
                }
            }
            neogenealogy_storage::StorageError::Import(msg) if msg.contains("already exists") => {
                ApiError {
                    status: StatusCode::CONFLICT,
                    code: "RESEARCH_OUTCOME_ALREADY_EXISTS",
                    message: msg,
                }
            }
            other => ApiError::internal(other.to_string()),
        })?;
    Ok((StatusCode::CREATED, Json(to_json(row))))
}

pub async fn update_outcome(
    State(state): State<AppState>,
    Path((tree_id, outcome_id)): Path<(i64, i64)>,
    Json(body): Json<UpdateOutcomeBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if tree_id <= 0 || outcome_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    if let Some(ref t) = body.r#type {
        validate_type(&t.to_uppercase())?;
    }
    if let Some(ref s) = body.summary {
        if s.trim().is_empty() {
            return Err(ApiError::bad_request(
                "INVALID_SUMMARY",
                "summary must not be empty",
            ));
        }
    }
    let existing = state
        .storage
        .get_research_outcome(outcome_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "RESEARCH_OUTCOME_NOT_FOUND",
                format!("Outcome {outcome_id} not found"),
            )
        })?;
    if existing.tree_id != tree_id {
        return Err(ApiError::not_found(
            "RESEARCH_OUTCOME_NOT_FOUND",
            format!("Outcome {outcome_id} not in tree {tree_id}"),
        ));
    }
    let row = state
        .storage
        .update_research_outcome(
            outcome_id,
            body.r#type.as_deref().map(|s| s.to_uppercase()).as_deref(),
            body.summary.as_deref(),
            body.details.as_deref(),
        )
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(to_json(row)))
}

pub async fn delete_outcome(
    State(state): State<AppState>,
    Path((tree_id, outcome_id)): Path<(i64, i64)>,
) -> Result<StatusCode, ApiError> {
    if tree_id <= 0 || outcome_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    let existing = state
        .storage
        .get_research_outcome(outcome_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "RESEARCH_OUTCOME_NOT_FOUND",
                format!("Outcome {outcome_id} not found"),
            )
        })?;
    if existing.tree_id != tree_id {
        return Err(ApiError::not_found(
            "RESEARCH_OUTCOME_NOT_FOUND",
            format!("Outcome {outcome_id} not in tree {tree_id}"),
        ));
    }
    state.storage.delete_research_outcome(outcome_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
