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
    pub has_outcome: Option<bool>,
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

#[allow(dead_code)]
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
        "outcome": null,
        "has_outcome": false
    })
}

#[allow(dead_code)]
fn to_json_with_has_outcome(
    row: neogenealogy_storage::models::ResearchTaskRow,
    has_outcome: bool,
    opportunity: Option<serde_json::Value>,
) -> serde_json::Value {
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
        "outcome": null,
        "has_outcome": has_outcome,
        "opportunity": opportunity,
        "session_id": row.session_id,
        "session": null
    })
}

fn to_json_with_has_outcome_and_session(
    row: neogenealogy_storage::models::ResearchTaskRow,
    has_outcome: bool,
    opportunity: Option<serde_json::Value>,
    session: Option<serde_json::Value>,
) -> serde_json::Value {
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
        "outcome": null,
        "has_outcome": has_outcome,
        "opportunity": opportunity,
        "session_id": row.session_id,
        "session": session
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
    let session = if let Some(sid) = row.session_id {
        state
            .storage
            .get_research_session(sid)
            .await
            .ok()
            .flatten()
            .map(|s| serde_json::json!({"id": s.id, "title": s.title, "status": s.status}))
    } else {
        None
    };
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
        "outcome": outcome,
        "session_id": row.session_id,
        "session": session
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
        .list_research_tasks_filtered(
            tree_id,
            status.as_deref(),
            params.person_id,
            params.opportunity_id,
            params.has_outcome,
            limit,
            offset,
        )
        .await?;
    // Batch fetch has_outcome, opportunity and session info to avoid N+1
    let task_ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
    let has_map = state.storage.get_tasks_has_outcome_map(&task_ids).await?;
    let session_map = state
        .storage
        .get_tasks_session_map(&task_ids)
        .await
        .unwrap_or_default();
    let opp_ids: Vec<i64> = rows.iter().filter_map(|r| r.opportunity_id).collect();
    let mut opp_map: std::collections::HashMap<i64, serde_json::Value> =
        std::collections::HashMap::new();
    if !opp_ids.is_empty() {
        let placeholders = opp_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "SELECT * FROM research_opportunities WHERE id IN ({placeholders}) AND tree_id=?"
        );
        let mut q = sqlx::query_as::<_, neogenealogy_storage::models::ResearchOpportunityRow>(&sql);
        for oid in &opp_ids {
            q = q.bind(oid);
        }
        q = q.bind(tree_id);
        if let Ok(opps) = q.fetch_all(&state.storage.pool).await {
            for o in opps {
                opp_map.insert(
                    o.id,
                    serde_json::json!({
                        "id": o.id,
                        "score": o.score,
                        "priority": o.priority,
                        "why": o.why
                    }),
                );
            }
        }
    }
    let items: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|row| {
            let has = has_map.get(&row.id).copied().unwrap_or(false);
            let opp = row
                .opportunity_id
                .and_then(|oid| opp_map.get(&oid).cloned());
            let sess = session_map
                .get(&row.id)
                .map(|s| serde_json::json!({"id": s.id, "title": s.title, "status": s.status}));
            to_json_with_has_outcome_and_session(row, has, opp, sess)
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
