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
pub struct TaskListParams {
    pub status: Option<String>,
    pub person_id: Option<i64>,
    pub opportunity_id: Option<i64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize, Serialize)]
pub struct CreateTaskBody {
    pub title: String,
    pub description: Option<String>,
    pub person_id: Option<i64>,
    pub opportunity_id: Option<i64>,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateTaskBody {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub resolution: Option<String>,
}

fn validate_status(s: &str) -> Result<(), ApiError> {
    let allowed = [
        "OPEN",
        "IN_PROGRESS",
        "RESOLVED",
        "REJECTED",
        "INCONCLUSIVE",
    ];
    if !allowed.contains(&s) {
        return Err(ApiError::bad_request(
            "INVALID_RESEARCH_TASK_STATUS",
            format!("status must be one of {}", allowed.join(",")),
        ));
    }
    Ok(())
}

fn to_json(row: neogenealogy_storage::models::ResearchTaskRow) -> serde_json::Value {
    serde_json::json!({
        "id": row.id,
        "tree_id": row.tree_id,
        "opportunity_id": row.opportunity_id,
        "person_id": row.person_id,
        "title": row.title,
        "description": row.description,
        "status": row.status,
        "created_at": row.created_at,
        "updated_at": row.updated_at,
        "started_at": row.started_at,
        "completed_at": row.completed_at,
        "resolution": row.resolution,
        "outcome": null
    })
}

async fn to_json_with_outcome(
    state: &crate::state::AppState,
    row: neogenealogy_storage::models::ResearchTaskRow,
) -> serde_json::Value {
    let outcome = state
        .storage
        .get_research_outcome_by_task(row.id)
        .await
        .ok()
        .flatten()
        .map(|o| {
            serde_json::json!({
                "id": o.id,
                "tree_id": o.tree_id,
                "task_id": o.task_id,
                "type": o.r#type,
                "summary": o.summary,
                "details": o.details,
                "created_at": o.created_at,
                "updated_at": o.updated_at
            })
        });
    serde_json::json!({
        "id": row.id,
        "tree_id": row.tree_id,
        "opportunity_id": row.opportunity_id,
        "person_id": row.person_id,
        "title": row.title,
        "description": row.description,
        "status": row.status,
        "created_at": row.created_at,
        "updated_at": row.updated_at,
        "started_at": row.started_at,
        "completed_at": row.completed_at,
        "resolution": row.resolution,
        "outcome": outcome
    })
}

pub async fn list_tasks(
    State(state): State<AppState>,
    Path(tree_id): Path<i64>,
    Query(params): Query<TaskListParams>,
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
    if let Some(ref s) = params.status {
        validate_status(&s.to_uppercase())?;
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
    let (rows, total) = state
        .storage
        .list_research_tasks(
            tree_id,
            status.as_deref(),
            params.person_id,
            params.opportunity_id,
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

pub async fn get_task(
    State(state): State<AppState>,
    Path((tree_id, task_id)): Path<(i64, i64)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if tree_id <= 0 || task_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    let row = state
        .storage
        .get_research_task(task_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "RESEARCH_TASK_NOT_FOUND",
                format!("Task {task_id} not found"),
            )
        })?;
    if row.tree_id != tree_id {
        return Err(ApiError::not_found(
            "RESEARCH_TASK_NOT_FOUND",
            format!("Task {task_id} not in tree {tree_id}"),
        ));
    }
    Ok(Json(to_json_with_outcome(&state, row).await))
}

pub async fn create_task(
    State(state): State<AppState>,
    Path(tree_id): Path<i64>,
    Json(body): Json<CreateTaskBody>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    if tree_id <= 0 {
        return Err(ApiError::bad_request(
            "INVALID_TREE_ID",
            "tree_id must be >0",
        ));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    if body.title.trim().is_empty() {
        return Err(ApiError::bad_request(
            "INVALID_TITLE",
            "title must not be empty",
        ));
    }
    // Validate person/opportunity belong to tree is done in storage
    let row = state
        .storage
        .create_research_task(
            tree_id,
            body.opportunity_id,
            body.person_id,
            &body.title,
            body.description.as_deref(),
        )
        .await
        .map_err(|e| match e {
            neogenealogy_storage::StorageError::NotFound(msg) => {
                if msg.contains("opportunity") {
                    ApiError::not_found("RESEARCH_OPPORTUNITY_NOT_FOUND", msg)
                } else if msg.contains("person") {
                    ApiError::not_found("PERSON_NOT_FOUND", msg)
                } else {
                    ApiError::not_found("NOT_FOUND", msg)
                }
            }
            other => ApiError::internal(other.to_string()),
        })?;
    Ok((
        StatusCode::CREATED,
        Json(to_json_with_outcome(&state, row).await),
    ))
}

pub async fn create_task_from_opportunity(
    State(state): State<AppState>,
    Path((tree_id, opportunity_id)): Path<(i64, i64)>,
    Json(body): Json<serde_json::Value>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    if tree_id <= 0 || opportunity_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    // Fetch opportunity to get person and title
    let opp: Option<neogenealogy_storage::models::ResearchOpportunityRow> =
        sqlx::query_as("SELECT * FROM research_opportunities WHERE id=?1 AND tree_id=?2")
            .bind(opportunity_id)
            .bind(tree_id)
            .fetch_optional(&state.storage.pool)
            .await
            .map_err(|e| ApiError::internal(e.to_string()))?;
    let opp = opp.ok_or_else(|| {
        ApiError::not_found(
            "RESEARCH_OPPORTUNITY_NOT_FOUND",
            format!("Opportunity {opportunity_id} not found"),
        )
    })?;
    let title = body
        .get("title")
        .and_then(|v| v.as_str())
        .unwrap_or(&format!("Research: opportunity {}", opportunity_id))
        .to_string();
    let description = body
        .get("description")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let person_id = opp.person_id;
    let row = state
        .storage
        .create_research_task(
            tree_id,
            Some(opportunity_id),
            Some(person_id),
            &title,
            description.as_deref(),
        )
        .await
        .map_err(|e| match e {
            neogenealogy_storage::StorageError::NotFound(msg) => {
                ApiError::not_found("NOT_FOUND", msg)
            }
            other => ApiError::internal(other.to_string()),
        })?;
    Ok((
        StatusCode::CREATED,
        Json(to_json_with_outcome(&state, row).await),
    ))
}

pub async fn update_task(
    State(state): State<AppState>,
    Path((tree_id, task_id)): Path<(i64, i64)>,
    Json(body): Json<UpdateTaskBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if tree_id <= 0 || task_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    let existing = state
        .storage
        .get_research_task(task_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "RESEARCH_TASK_NOT_FOUND",
                format!("Task {task_id} not found"),
            )
        })?;
    if existing.tree_id != tree_id {
        return Err(ApiError::not_found(
            "RESEARCH_TASK_NOT_FOUND",
            format!("Task {task_id} not in tree {tree_id}"),
        ));
    }
    if let Some(ref s) = body.status {
        validate_status(&s.to_uppercase())?;
    }
    let status = body.status.as_deref().map(|s| s.to_uppercase());
    let row = state
        .storage
        .update_research_task(
            task_id,
            body.title.as_deref(),
            body.description.as_deref(),
            status.as_deref(),
            body.resolution.as_deref(),
        )
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(to_json_with_outcome(&state, row).await))
}

pub async fn delete_task(
    State(state): State<AppState>,
    Path((tree_id, task_id)): Path<(i64, i64)>,
) -> Result<StatusCode, ApiError> {
    if tree_id <= 0 || task_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    let existing = state
        .storage
        .get_research_task(task_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "RESEARCH_TASK_NOT_FOUND",
                format!("Task {task_id} not found"),
            )
        })?;
    if existing.tree_id != tree_id {
        return Err(ApiError::not_found(
            "RESEARCH_TASK_NOT_FOUND",
            format!("Task {task_id} not in tree {tree_id}"),
        ));
    }
    state.storage.delete_research_task(task_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
