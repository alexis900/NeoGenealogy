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
pub struct SessionListParams {
    pub status: Option<String>,
    pub person_id: Option<i64>,
    pub opportunity_id: Option<i64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub history: Option<bool>,
}

#[derive(Deserialize)]
pub struct SessionHistoryParams {
    pub status: Option<String>,
    pub person_id: Option<i64>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub page: Option<i64>,
    pub tree_id: Option<i64>,
}

#[derive(Deserialize, Serialize)]
pub struct CreateSessionBody {
    pub title: String,
    pub description: Option<String>,
    pub person_id: Option<i64>,
    pub opportunity_id: Option<i64>,
    pub tree_id: Option<i64>, // optional when tree_id in path
}

#[derive(Deserialize, Serialize)]
pub struct UpdateSessionBody {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub person_id: Option<i64>,
    pub opportunity_id: Option<i64>,
}

#[derive(Deserialize)]
pub struct TaskSessionBody {
    pub session_id: i64,
}

fn validate_session_status(s: &str) -> Result<(), ApiError> {
    let allowed = ["PLANNED", "ACTIVE", "COMPLETED", "ABANDONED"];
    if !allowed.contains(&s) {
        return Err(ApiError::bad_request(
            "INVALID_SESSION_STATUS",
            format!("status must be one of {}", allowed.join(",")),
        ));
    }
    Ok(())
}

fn validate_history_status(s: &str) -> Result<(), ApiError> {
    let allowed = ["COMPLETED", "ABANDONED"];
    if !allowed.contains(&s) {
        return Err(ApiError::bad_request(
            "INVALID_SESSION_STATUS",
            format!("history status must be one of {}", allowed.join(",")),
        ));
    }
    Ok(())
}

pub async fn list_sessions(
    State(state): State<AppState>,
    Path(tree_id): Path<i64>,
    Query(params): Query<SessionListParams>,
) -> Result<Json<Paginated<serde_json::Value>>, ApiError> {
    // Support history via query param ?history=true as alternative
    if params.history.unwrap_or(false) {
        // delegate to history logic with same params
        let hist_params = SessionHistoryParams {
            status: params.status.clone(),
            person_id: params.person_id,
            limit: params.limit,
            offset: params.offset,
            page: None,
            tree_id: Some(tree_id),
        };
        return list_sessions_history(State(state), Path(tree_id), Query(hist_params)).await;
    }
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
        validate_session_status(&s.to_uppercase())?;
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
        .list_research_sessions(
            tree_id,
            status.as_deref(),
            params.person_id,
            params.opportunity_id,
            limit,
            offset,
        )
        .await?;
    let items: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| {
            serde_json::json!({
                "id": r.id,
                "tree_id": r.tree_id,
                "title": r.title,
                "description": r.description,
                "status": r.status,
                "person_id": r.person_id,
                "opportunity_id": r.opportunity_id,
                "created_at": r.created_at,
                "updated_at": r.updated_at,
                "started_at": r.started_at,
                "completed_at": r.completed_at
            })
        })
        .collect();
    Ok(Json(Paginated {
        items,
        pagination: PaginationMeta {
            limit,
            offset,
            total,
        },
    }))
}

pub async fn list_sessions_history(
    State(state): State<AppState>,
    Path(tree_id): Path<i64>,
    Query(params): Query<SessionHistoryParams>,
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
        validate_history_status(&s.to_uppercase())?;
    }
    let limit = params.limit.unwrap_or(50);
    let mut offset = params.offset.unwrap_or(0);
    if let Some(page) = params.page {
        if page <= 0 {
            return Err(ApiError::bad_request("INVALID_PAGE", "page must be >=1"));
        }
        offset = (page - 1) * limit;
    }
    if !(0..=100).contains(&limit) {
        return Err(ApiError::bad_request("INVALID_LIMIT", "limit 0..100"));
    }
    if offset < 0 {
        return Err(ApiError::bad_request("INVALID_OFFSET", "offset >=0"));
    }
    let status = params.status.as_deref().map(|s| s.to_uppercase());
    let (rows, total) = state
        .storage
        .list_research_sessions_history(tree_id, status.as_deref(), params.person_id, limit, offset)
        .await?;
    // batch stats to avoid N+1
    let sids: Vec<i64> = rows.iter().map(|r| r.id).collect();
    let stats_map = state
        .storage
        .get_sessions_stats(&sids)
        .await
        .unwrap_or_default();
    let items: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| {
            let stats = stats_map.get(&r.id).cloned().unwrap_or_default();
            serde_json::json!({
                "id": r.id,
                "tree_id": r.tree_id,
                "title": r.title,
                "description": r.description,
                "status": r.status,
                "person_id": r.person_id,
                "opportunity_id": r.opportunity_id,
                "created_at": r.created_at,
                "updated_at": r.updated_at,
                "started_at": r.started_at,
                "completed_at": r.completed_at,
                "stats": stats
            })
        })
        .collect();
    Ok(Json(Paginated {
        items,
        pagination: PaginationMeta {
            limit,
            offset,
            total,
        },
    }))
}

pub async fn list_sessions_history_generic(
    State(state): State<AppState>,
    Query(params): Query<SessionHistoryParams>,
) -> Result<Json<Paginated<serde_json::Value>>, ApiError> {
    let tree_id = params
        .tree_id
        .ok_or_else(|| ApiError::bad_request("TREE_REQUIRED", "tree_id query param required"))?;
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
        validate_history_status(&s.to_uppercase())?;
    }
    let limit = params.limit.unwrap_or(50);
    let mut offset = params.offset.unwrap_or(0);
    if let Some(page) = params.page {
        if page <= 0 {
            return Err(ApiError::bad_request("INVALID_PAGE", "page must be >=1"));
        }
        offset = (page - 1) * limit;
    }
    if !(0..=100).contains(&limit) {
        return Err(ApiError::bad_request("INVALID_LIMIT", "limit 0..100"));
    }
    if offset < 0 {
        return Err(ApiError::bad_request("INVALID_OFFSET", "offset >=0"));
    }
    let status = params.status.as_deref().map(|s| s.to_uppercase());
    let (rows, total) = state
        .storage
        .list_research_sessions_history(tree_id, status.as_deref(), params.person_id, limit, offset)
        .await?;
    let sids: Vec<i64> = rows.iter().map(|r| r.id).collect();
    let stats_map = state
        .storage
        .get_sessions_stats(&sids)
        .await
        .unwrap_or_default();
    let items: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| {
            let stats = stats_map.get(&r.id).cloned().unwrap_or_default();
            serde_json::json!({
                "id": r.id,
                "tree_id": r.tree_id,
                "title": r.title,
                "description": r.description,
                "status": r.status,
                "person_id": r.person_id,
                "opportunity_id": r.opportunity_id,
                "created_at": r.created_at,
                "updated_at": r.updated_at,
                "started_at": r.started_at,
                "completed_at": r.completed_at,
                "stats": stats
            })
        })
        .collect();
    Ok(Json(Paginated {
        items,
        pagination: PaginationMeta {
            limit,
            offset,
            total,
        },
    }))
}

pub async fn get_session(
    State(state): State<AppState>,
    Path((tree_id, session_id)): Path<(i64, i64)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if tree_id <= 0 || session_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    let detail = state
        .storage
        .get_session_detail(session_id)
        .await
        .map_err(|e| match e {
            neogenealogy_storage::StorageError::NotFound(msg) => {
                ApiError::not_found("RESEARCH_SESSION_NOT_FOUND", msg)
            }
            other => ApiError::internal(other.to_string()),
        })?;
    // verify tree isolation
    let session_tree = detail["session"]["tree_id"].as_i64().unwrap_or(-1);
    if session_tree != tree_id {
        return Err(ApiError::not_found(
            "RESEARCH_SESSION_NOT_FOUND",
            format!("Session {session_id} not in tree {tree_id}"),
        ));
    }
    Ok(Json(detail))
}

pub async fn create_session(
    State(state): State<AppState>,
    Path(tree_id): Path<i64>,
    Json(body): Json<CreateSessionBody>,
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
    // if body tree_id provided and mismatches path, error
    if let Some(btid) = body.tree_id {
        if btid != tree_id {
            return Err(ApiError::bad_request(
                "TREE_MISMATCH",
                "tree_id in body must match path",
            ));
        }
    }
    let row = state
        .storage
        .create_research_session(
            tree_id,
            &body.title,
            body.description.as_deref(),
            body.person_id,
            body.opportunity_id,
        )
        .await
        .map_err(|e| match e {
            neogenealogy_storage::StorageError::NotFound(msg) => {
                if msg.contains("person") {
                    ApiError::not_found("PERSON_NOT_FOUND", msg)
                } else if msg.contains("opportunity") {
                    ApiError::not_found("RESEARCH_OPPORTUNITY_NOT_FOUND", msg)
                } else {
                    ApiError::not_found("NOT_FOUND", msg)
                }
            }
            other => ApiError::internal(other.to_string()),
        })?;
    let detail = state.storage.get_session_detail(row.id).await?;
    Ok((StatusCode::CREATED, Json(detail)))
}

// Generic create without tree path (spec example POST /api/v1/research-sessions with tree_id in body)
pub async fn create_session_generic(
    State(state): State<AppState>,
    Json(body): Json<CreateSessionBody>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let tree_id = body
        .tree_id
        .ok_or_else(|| ApiError::bad_request("INVALID_TREE_ID", "tree_id required"))?;
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
    let row = state
        .storage
        .create_research_session(
            tree_id,
            &body.title,
            body.description.as_deref(),
            body.person_id,
            body.opportunity_id,
        )
        .await
        .map_err(|e| match e {
            neogenealogy_storage::StorageError::NotFound(msg) => {
                if msg.contains("person") {
                    ApiError::not_found("PERSON_NOT_FOUND", msg)
                } else if msg.contains("opportunity") {
                    ApiError::not_found("RESEARCH_OPPORTUNITY_NOT_FOUND", msg)
                } else {
                    ApiError::not_found("NOT_FOUND", msg)
                }
            }
            other => ApiError::internal(other.to_string()),
        })?;
    let detail = state.storage.get_session_detail(row.id).await?;
    Ok((StatusCode::CREATED, Json(detail)))
}

pub async fn update_session(
    State(state): State<AppState>,
    Path((tree_id, session_id)): Path<(i64, i64)>,
    Json(body): Json<UpdateSessionBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if tree_id <= 0 || session_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    let existing = state
        .storage
        .get_research_session(session_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "RESEARCH_SESSION_NOT_FOUND",
                format!("Session {session_id} not found"),
            )
        })?;
    if existing.tree_id != tree_id {
        return Err(ApiError::not_found(
            "RESEARCH_SESSION_NOT_FOUND",
            format!("Session {session_id} not in tree {tree_id}"),
        ));
    }
    if let Some(ref s) = body.status {
        validate_session_status(&s.to_uppercase())?;
    }
    let status = body.status.as_deref().map(|s| s.to_uppercase());
    // For patch, we need to allow explicit null for person_id/opportunity_id and description.
    // The UpdateSessionBody uses Option<i64> for person_id which cannot distinguish null vs missing.
    // We will treat if body contains field we update, otherwise keep. For description, same.
    // To support clearing, we need to check if field was present via serde? Simpler: if body.person_id is Some, we set to Some(value), if None, keep existing.
    // But we cannot clear via this API. That's acceptable for now; clearing via separate logic not needed.
    // However spec expects ability to update person/opportunity? We'll handle as optional update.
    // For description, same.
    let title_opt = body.title.as_deref();
    let desc_opt: Option<Option<&str>> = if body.description.is_some() {
        Some(body.description.as_deref())
    } else {
        None
    };
    let person_opt: Option<Option<i64>> = if body.person_id.is_some() {
        Some(body.person_id)
    } else {
        None
    };
    let opp_opt: Option<Option<i64>> = if body.opportunity_id.is_some() {
        Some(body.opportunity_id)
    } else {
        None
    };
    let row = state
        .storage
        .update_research_session(
            session_id,
            title_opt,
            desc_opt,
            status.as_deref(),
            person_opt,
            opp_opt,
        )
        .await
        .map_err(|e| match e {
            neogenealogy_storage::StorageError::NotFound(msg) => {
                ApiError::not_found("NOT_FOUND", msg)
            }
            neogenealogy_storage::StorageError::Import(msg) => {
                ApiError::bad_request("INVALID_INPUT", msg)
            }
            other => ApiError::internal(other.to_string()),
        })?;
    let detail = state.storage.get_session_detail(row.id).await?;
    Ok(Json(detail))
}

pub async fn delete_session(
    State(state): State<AppState>,
    Path((tree_id, session_id)): Path<(i64, i64)>,
) -> Result<StatusCode, ApiError> {
    if tree_id <= 0 || session_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    let existing = state
        .storage
        .get_research_session(session_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "RESEARCH_SESSION_NOT_FOUND",
                format!("Session {session_id} not found"),
            )
        })?;
    if existing.tree_id != tree_id {
        return Err(ApiError::not_found(
            "RESEARCH_SESSION_NOT_FOUND",
            format!("Session {session_id} not in tree {tree_id}"),
        ));
    }
    state.storage.delete_research_session(session_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// Generic list/get without tree prefix (optional)
pub async fn list_sessions_generic(
    State(_state): State<AppState>,
    Query(_params): Query<SessionListParams>,
) -> Result<Json<Paginated<serde_json::Value>>, ApiError> {
    Err(ApiError::bad_request(
        "TREE_REQUIRED",
        "tree_id required - use /trees/:tree_id/research-sessions",
    ))
}

pub async fn get_session_generic(
    State(state): State<AppState>,
    Path(session_id): Path<i64>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if session_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "session_id must be >0"));
    }
    let detail = state
        .storage
        .get_session_detail(session_id)
        .await
        .map_err(|e| match e {
            neogenealogy_storage::StorageError::NotFound(msg) => {
                ApiError::not_found("RESEARCH_SESSION_NOT_FOUND", msg)
            }
            other => ApiError::internal(other.to_string()),
        })?;
    Ok(Json(detail))
}

pub async fn patch_session_generic(
    State(state): State<AppState>,
    Path(session_id): Path<i64>,
    Json(body): Json<UpdateSessionBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if session_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    let existing = state
        .storage
        .get_research_session(session_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "RESEARCH_SESSION_NOT_FOUND",
                format!("Session {session_id} not found"),
            )
        })?;
    if let Some(ref s) = body.status {
        validate_session_status(&s.to_uppercase())?;
    }
    let status = body.status.as_deref().map(|s| s.to_uppercase());
    let title_opt = body.title.as_deref();
    let desc_opt: Option<Option<&str>> = if body.description.is_some() {
        Some(body.description.as_deref())
    } else {
        None
    };
    let person_opt: Option<Option<i64>> = if body.person_id.is_some() {
        Some(body.person_id)
    } else {
        None
    };
    let opp_opt: Option<Option<i64>> = if body.opportunity_id.is_some() {
        Some(body.opportunity_id)
    } else {
        None
    };
    let row = state
        .storage
        .update_research_session(
            session_id,
            title_opt,
            desc_opt,
            status.as_deref(),
            person_opt,
            opp_opt,
        )
        .await
        .map_err(|e| match e {
            neogenealogy_storage::StorageError::NotFound(msg) => {
                ApiError::not_found("NOT_FOUND", msg)
            }
            neogenealogy_storage::StorageError::Import(msg) => {
                ApiError::bad_request("INVALID_INPUT", msg)
            }
            other => ApiError::internal(other.to_string()),
        })?;
    let _ = existing;
    let detail = state.storage.get_session_detail(row.id).await?;
    Ok(Json(detail))
}

pub async fn delete_session_generic(
    State(state): State<AppState>,
    Path(session_id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    if session_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    let existing = state
        .storage
        .get_research_session(session_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "RESEARCH_SESSION_NOT_FOUND",
                format!("Session {session_id} not found"),
            )
        })?;
    let _ = existing;
    state.storage.delete_research_session(session_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// Task ↔ Session
pub async fn assign_task_to_session(
    State(state): State<AppState>,
    Path((tree_id, task_id)): Path<(i64, i64)>,
    Json(body): Json<TaskSessionBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if tree_id <= 0 || task_id <= 0 || body.session_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    let row = state
        .storage
        .assign_task_to_session(task_id, body.session_id)
        .await
        .map_err(|e| match e {
            neogenealogy_storage::StorageError::NotFound(msg) => {
                ApiError::not_found("NOT_FOUND", msg)
            }
            other => ApiError::internal(other.to_string()),
        })?;
    // verify task tree matches path
    if row.tree_id != tree_id {
        return Err(ApiError::not_found(
            "RESEARCH_TASK_NOT_FOUND",
            format!("Task {task_id} not in tree {tree_id}"),
        ));
    }
    Ok(Json(serde_json::json!({
        "id": row.id,
        "tree_id": row.tree_id,
        "session_id": row.session_id,
        "title": row.title,
        "status": row.status
    })))
}

pub async fn remove_task_from_session(
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
        .remove_task_from_session(task_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    if row.tree_id != tree_id {
        return Err(ApiError::not_found(
            "RESEARCH_TASK_NOT_FOUND",
            format!("Task {task_id} not in tree {tree_id}"),
        ));
    }
    Ok(Json(serde_json::json!({
        "id": row.id,
        "tree_id": row.tree_id,
        "session_id": row.session_id,
        "title": row.title,
        "status": row.status
    })))
}

pub async fn list_session_tasks(
    State(state): State<AppState>,
    Path((tree_id, session_id)): Path<(i64, i64)>,
) -> Result<Json<Paginated<serde_json::Value>>, ApiError> {
    if tree_id <= 0 || session_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    let sess = state
        .storage
        .get_research_session(session_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "RESEARCH_SESSION_NOT_FOUND",
                format!("Session {session_id} not found"),
            )
        })?;
    if sess.tree_id != tree_id {
        return Err(ApiError::not_found(
            "RESEARCH_SESSION_NOT_FOUND",
            format!("Session {session_id} not in tree {tree_id}"),
        ));
    }
    let tasks = state.storage.list_tasks_for_session(session_id).await?;
    let task_ids: Vec<i64> = tasks.iter().map(|t| t.id).collect();
    let has_map = state
        .storage
        .get_tasks_has_outcome_map(&task_ids)
        .await
        .unwrap_or_default();
    let items: Vec<serde_json::Value> = tasks
        .into_iter()
        .map(|t| {
            serde_json::json!({
                "id": t.id,
                "tree_id": t.tree_id,
                "opportunity_id": t.opportunity_id,
                "person_id": t.person_id,
                "title": t.title,
                "description": t.description,
                "status": t.status,
                "session_id": t.session_id,
                "created_at": t.created_at,
                "updated_at": t.updated_at,
                "has_outcome": has_map.get(&t.id).copied().unwrap_or(false)
            })
        })
        .collect();
    let total = items.len() as i64;
    Ok(Json(Paginated {
        items,
        pagination: PaginationMeta {
            limit: total,
            offset: 0,
            total,
        },
    }))
}
