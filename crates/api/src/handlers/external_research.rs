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
pub struct CreateQueryBody {
    pub provider: String,
    pub query: String,
    pub tree_id: Option<i64>,
}

#[derive(Deserialize)]
pub struct QueryListParams {
    pub tree_id: Option<i64>,
    pub task_id: Option<i64>,
    pub provider: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub page: Option<i64>,
}

#[derive(Deserialize)]
pub struct PaginationOnly {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub page: Option<i64>,
}

#[derive(Serialize)]
pub struct QueryResponse {
    pub id: i64,
    pub tree_id: i64,
    pub task_id: i64,
    pub provider: String,
    pub query: String,
    pub status: String,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub latest_execution: Option<serde_json::Value>,
}

fn map_query_row(
    row: neogenealogy_storage::models::ResearchQueryRow,
    latest: Option<neogenealogy_storage::models::ResearchQueryExecutionRow>,
    result_count: i64,
) -> serde_json::Value {
    let latest_json = latest.map(|e| {
        let rc = result_count;
        serde_json::json!({
            "id": e.id,
            "status": e.status,
            "started_at": e.started_at,
            "completed_at": e.completed_at,
            "error_code": e.error_code,
            "error_message": e.error_message,
            "provider_request_id": e.provider_request_id,
            "result_count": rc
        })
    });
    serde_json::json!({
        "id": row.id,
        "tree_id": row.tree_id,
        "task_id": row.task_id,
        "provider": row.provider,
        "query": row.query,
        "status": row.status,
        "created_at": row.created_at,
        "started_at": row.started_at,
        "completed_at": row.completed_at,
        "error_code": row.error_code,
        "error_message": row.error_message,
        "latest_execution": latest_json
    })
}

fn map_execution_row(
    row: neogenealogy_storage::models::ResearchQueryExecutionRow,
    result_count: Option<i64>,
) -> serde_json::Value {
    serde_json::json!({
        "id": row.id,
        "query_id": row.query_id,
        "status": row.status,
        "started_at": row.started_at,
        "completed_at": row.completed_at,
        "error_code": row.error_code,
        "error_message": row.error_message,
        "provider_request_id": row.provider_request_id,
        "provider_metadata": row.provider_metadata.and_then(|s| serde_json::from_str::<serde_json::Value>(&s).ok()),
        "created_at": row.created_at,
        "result_count": result_count
    })
}

fn map_result_row(row: neogenealogy_storage::models::ResearchResultRow) -> serde_json::Value {
    let metadata: serde_json::Value = row
        .metadata
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or(serde_json::json!({}));
    serde_json::json!({
        "id": row.id,
        "execution_id": row.execution_id,
        "query_id": row.query_id,
        "provider": row.provider,
        "external_id": row.external_id,
        "title": row.title,
        "description": row.description,
        "url": row.url,
        "record_type": row.record_type,
        "date": row.date,
        "place": row.place,
        "metadata": metadata,
        "position": row.position,
        "created_at": row.created_at
    })
}

fn validate_provider(name: &str) -> Result<(), ApiError> {
    let norm = name.to_lowercase();
    let reg = neogenealogy_storage::external_research::ResearchProviderRegistry::new();
    if reg.get(&norm).is_none() {
        return Err(ApiError::bad_request(
            "INVALID_PROVIDER",
            format!("unknown provider {name}"),
        ));
    }
    Ok(())
}

fn familysearch_status() -> serde_json::Value {
    let cfg = neogenealogy_storage::familysearch::FamilySearchConfig::from_env();
    let configured = cfg.is_configured() && cfg.enabled();
    let status = if !cfg.enabled() {
        "disabled"
    } else if configured {
        "configured"
    } else {
        "not_configured"
    };
    serde_json::json!({
        "name": "familysearch",
        "display_name": "FamilySearch",
        "configured": configured,
        "enabled": cfg.enabled(),
        "status": status,
        "requires_auth": !configured
    })
}

fn providers_json() -> serde_json::Value {
    let fs = familysearch_status();
    let mock = serde_json::json!({
        "name": "mock",
        "display_name": "Mock",
        "configured": true,
        "enabled": true,
        "status": "configured",
        "requires_auth": false
    });
    serde_json::json!({ "providers": [mock, fs] })
}

pub async fn list_providers_generic(
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    Ok(Json(providers_json()))
}

pub async fn list_providers_tree(
    State(state): State<AppState>,
    Path(tree_id): Path<i64>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if tree_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "tree_id must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} not found"))
    })?;
    Ok(Json(providers_json()))
}

// POST /api/v1/research-tasks/:task_id/research-queries
pub async fn create_query_for_task(
    State(state): State<AppState>,
    Path(task_id): Path<i64>,
    Json(body): Json<CreateQueryBody>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    if task_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "task_id must be >0"));
    }
    if body.query.trim().is_empty() {
        return Err(ApiError::bad_request(
            "INVALID_QUERY",
            "query must not be empty",
        ));
    }
    if body.provider.trim().is_empty() {
        return Err(ApiError::bad_request(
            "INVALID_PROVIDER",
            "provider required",
        ));
    }
    validate_provider(&body.provider)?;
    // get task to find tree_id
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
    let tree_id = body.tree_id.unwrap_or(task.tree_id);
    if tree_id != task.tree_id {
        return Err(ApiError::bad_request("TREE_MISMATCH", "tree_id mismatch"));
    }
    let row = state
        .storage
        .create_research_query(tree_id, task_id, &body.provider, &body.query)
        .await
        .map_err(|e| match e {
            neogenealogy_storage::StorageError::NotFound(msg) => {
                ApiError::not_found("NOT_FOUND", msg)
            }
            other => ApiError::internal(other.to_string()),
        })?;
    let latest = state
        .storage
        .get_latest_execution_for_query(row.id)
        .await
        .unwrap_or(None);
    let rc = 0;
    Ok((StatusCode::CREATED, Json(map_query_row(row, latest, rc))))
}

// POST /api/v1/trees/:tree_id/research-tasks/:task_id/research-queries
pub async fn create_query_for_task_tree(
    State(state): State<AppState>,
    Path((tree_id, task_id)): Path<(i64, i64)>,
    Json(body): Json<CreateQueryBody>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    if tree_id <= 0 || task_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} not found"))
    })?;
    if body.query.trim().is_empty() {
        return Err(ApiError::bad_request(
            "INVALID_QUERY",
            "query must not be empty",
        ));
    }
    if body.provider.trim().is_empty() {
        return Err(ApiError::bad_request(
            "INVALID_PROVIDER",
            "provider required",
        ));
    }
    validate_provider(&body.provider)?;
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
    let row = state
        .storage
        .create_research_query(tree_id, task_id, &body.provider, &body.query)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let latest = state
        .storage
        .get_latest_execution_for_query(row.id)
        .await
        .unwrap_or(None);
    Ok((StatusCode::CREATED, Json(map_query_row(row, latest, 0))))
}

// GET /api/v1/research-queries
pub async fn list_queries_generic(
    State(state): State<AppState>,
    Query(params): Query<QueryListParams>,
) -> Result<Json<Paginated<serde_json::Value>>, ApiError> {
    let tree_id = params
        .tree_id
        .ok_or_else(|| ApiError::bad_request("TREE_REQUIRED", "tree_id required"))?;
    if tree_id <= 0 {
        return Err(ApiError::bad_request(
            "INVALID_TREE_ID",
            "tree_id must be >0",
        ));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} not found"))
    })?;
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
    let (queries, total) = state
        .storage
        .list_research_queries(
            tree_id,
            params.task_id,
            params.provider.as_deref(),
            params.status.as_deref(),
            limit,
            offset,
        )
        .await?;
    // batch latest executions and counts
    let qids: Vec<i64> = queries.iter().map(|q| q.id).collect();
    let latest_map = state
        .storage
        .get_latest_executions_for_queries(&qids)
        .await
        .unwrap_or_default();
    let exec_ids: Vec<i64> = latest_map.values().map(|e| e.id).collect();
    let counts = state
        .storage
        .count_results_for_executions(&exec_ids)
        .await
        .unwrap_or_default();
    let items: Vec<serde_json::Value> = queries
        .into_iter()
        .map(|q| {
            let latest = latest_map.get(&q.id).cloned();
            let rc = latest
                .as_ref()
                .and_then(|e| counts.get(&e.id).copied())
                .unwrap_or(0);
            map_query_row(q, latest, rc)
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

// GET /api/v1/trees/:tree_id/research-queries  and /trees/:tree_id/research-tasks/:task_id/research-queries
pub async fn list_queries_for_tree(
    State(state): State<AppState>,
    Path(tree_id): Path<i64>,
    Query(params): Query<QueryListParams>,
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
    let (queries, total) = state
        .storage
        .list_research_queries(
            tree_id,
            params.task_id,
            params.provider.as_deref(),
            params.status.as_deref(),
            limit,
            offset,
        )
        .await?;
    let qids: Vec<i64> = queries.iter().map(|q| q.id).collect();
    let latest_map = state
        .storage
        .get_latest_executions_for_queries(&qids)
        .await
        .unwrap_or_default();
    let exec_ids: Vec<i64> = latest_map.values().map(|e| e.id).collect();
    let counts = state
        .storage
        .count_results_for_executions(&exec_ids)
        .await
        .unwrap_or_default();
    let items: Vec<serde_json::Value> = queries
        .into_iter()
        .map(|q| {
            let latest = latest_map.get(&q.id).cloned();
            let rc = latest
                .as_ref()
                .and_then(|e| counts.get(&e.id).copied())
                .unwrap_or(0);
            map_query_row(q, latest, rc)
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

// GET /api/v1/trees/:tree_id/research-tasks/:task_id/research-queries (convenience)
pub async fn list_queries_for_task_tree(
    State(state): State<AppState>,
    Path((tree_id, task_id)): Path<(i64, i64)>,
    Query(params): Query<PaginationOnly>,
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
    let (queries, total) = state
        .storage
        .list_research_queries(tree_id, Some(task_id), None, None, limit, offset)
        .await?;
    let qids: Vec<i64> = queries.iter().map(|q| q.id).collect();
    let latest_map = state
        .storage
        .get_latest_executions_for_queries(&qids)
        .await
        .unwrap_or_default();
    let exec_ids: Vec<i64> = latest_map.values().map(|e| e.id).collect();
    let counts = state
        .storage
        .count_results_for_executions(&exec_ids)
        .await
        .unwrap_or_default();
    let items: Vec<serde_json::Value> = queries
        .into_iter()
        .map(|q| {
            let latest = latest_map.get(&q.id).cloned();
            let rc = latest
                .as_ref()
                .and_then(|e| counts.get(&e.id).copied())
                .unwrap_or(0);
            map_query_row(q, latest, rc)
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

// GET /api/v1/research-queries/:query_id
pub async fn get_query_generic(
    State(state): State<AppState>,
    Path(query_id): Path<i64>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if query_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "query_id must be >0"));
    }
    let row = state
        .storage
        .get_research_query(query_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "RESEARCH_QUERY_NOT_FOUND",
                format!("Query {query_id} not found"),
            )
        })?;
    let latest = state
        .storage
        .get_latest_execution_for_query(query_id)
        .await
        .unwrap_or(None);
    let rc = if let Some(exec) = &latest {
        let m = state
            .storage
            .count_results_for_executions(&[exec.id])
            .await
            .unwrap_or_default();
        *m.get(&exec.id).unwrap_or(&0)
    } else {
        0
    };
    Ok(Json(map_query_row(row, latest, rc)))
}

// GET /api/v1/trees/:tree_id/research-queries/:query_id
pub async fn get_query_tree(
    State(state): State<AppState>,
    Path((tree_id, query_id)): Path<(i64, i64)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if tree_id <= 0 || query_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} not found"))
    })?;
    let row = state
        .storage
        .get_research_query(query_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "RESEARCH_QUERY_NOT_FOUND",
                format!("Query {query_id} not found"),
            )
        })?;
    if row.tree_id != tree_id {
        return Err(ApiError::not_found(
            "RESEARCH_QUERY_NOT_FOUND",
            format!("Query {query_id} not in tree {tree_id}"),
        ));
    }
    let latest = state
        .storage
        .get_latest_execution_for_query(query_id)
        .await
        .unwrap_or(None);
    let rc = if let Some(exec) = &latest {
        let m = state
            .storage
            .count_results_for_executions(&[exec.id])
            .await
            .unwrap_or_default();
        *m.get(&exec.id).unwrap_or(&0)
    } else {
        0
    };
    Ok(Json(map_query_row(row, latest, rc)))
}

// DELETE /api/v1/research-queries/:query_id
pub async fn delete_query_generic(
    State(state): State<AppState>,
    Path(query_id): Path<i64>,
) -> Result<StatusCode, ApiError> {
    if query_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "query_id must be >0"));
    }
    let row = state
        .storage
        .get_research_query(query_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "RESEARCH_QUERY_NOT_FOUND",
                format!("Query {query_id} not found"),
            )
        })?;
    let _ = row;
    state.storage.delete_research_query(query_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_query_tree(
    State(state): State<AppState>,
    Path((tree_id, query_id)): Path<(i64, i64)>,
) -> Result<StatusCode, ApiError> {
    if tree_id <= 0 || query_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} not found"))
    })?;
    let row = state
        .storage
        .get_research_query(query_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "RESEARCH_QUERY_NOT_FOUND",
                format!("Query {query_id} not found"),
            )
        })?;
    if row.tree_id != tree_id {
        return Err(ApiError::not_found(
            "RESEARCH_QUERY_NOT_FOUND",
            format!("Query {query_id} not in tree {tree_id}"),
        ));
    }
    state.storage.delete_research_query(query_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// POST /api/v1/research-queries/:query_id/run
pub async fn run_query_generic(
    State(state): State<AppState>,
    Path(query_id): Path<i64>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if query_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "query_id must be >0"));
    }
    let query = state
        .storage
        .get_research_query(query_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "RESEARCH_QUERY_NOT_FOUND",
                format!("Query {query_id} not found"),
            )
        })?;
    execute_query(state, query).await
}

pub async fn run_query_tree(
    State(state): State<AppState>,
    Path((tree_id, query_id)): Path<(i64, i64)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if tree_id <= 0 || query_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} not found"))
    })?;
    let query = state
        .storage
        .get_research_query(query_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "RESEARCH_QUERY_NOT_FOUND",
                format!("Query {query_id} not found"),
            )
        })?;
    if query.tree_id != tree_id {
        return Err(ApiError::not_found(
            "RESEARCH_QUERY_NOT_FOUND",
            format!("Query {query_id} not in tree {tree_id}"),
        ));
    }
    execute_query(state, query).await
}

async fn execute_query(
    state: AppState,
    query: neogenealogy_storage::models::ResearchQueryRow,
) -> Result<Json<serde_json::Value>, ApiError> {
    let query_id = query.id;
    let provider_name = query.provider.clone();
    let query_text = query.query.clone();
    // Validate provider exists
    let registry = neogenealogy_storage::external_research::ResearchProviderRegistry::new();
    let provider = registry.get(&provider_name).ok_or_else(|| {
        ApiError::bad_request(
            "INVALID_PROVIDER",
            format!("unknown provider {}", provider_name),
        )
    })?;
    // Create execution row
    let exec = state
        .storage
        .create_execution(query_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    let start = std::time::Instant::now();
    let res = provider.search(&query_text).await;
    let duration_ms = start.elapsed().as_millis() as i64;
    match res {
        Ok(resp) => {
            // Persist results
            for (idx, cand) in resp.results.iter().enumerate() {
                // validate url already in storage layer; but also skip invalid?
                if let Some(url) = &cand.url {
                    if !neogenealogy_storage::external_research::is_valid_external_url(url) {
                        // skip or return error? spec says validate url, so skip invalid and continue? For now skip invalid url and create without url
                        let mut cand2 = cand.clone();
                        cand2.url = None;
                        state
                            .storage
                            .create_research_result(
                                exec.id,
                                query_id,
                                &provider_name,
                                &cand2,
                                idx as i64,
                            )
                            .await
                            .map_err(|e| ApiError::internal(e.to_string()))?;
                        continue;
                    }
                }
                state
                    .storage
                    .create_research_result(exec.id, query_id, &provider_name, cand, idx as i64)
                    .await
                    .map_err(|e| ApiError::internal(e.to_string()))?;
            }
            let prov_meta =
                serde_json::to_string(&resp.provider_metadata).unwrap_or("{}".to_string());
            let exec_done = state
                .storage
                .update_execution_status(
                    exec.id,
                    "COMPLETED",
                    None,
                    None,
                    resp.provider_request_id.as_deref(),
                    Some(&prov_meta),
                )
                .await
                .map_err(|e| ApiError::internal(e.to_string()))?;
            // tracing log without secrets
            tracing::info!(provider=%provider_name, query_id=query_id, execution_id=exec.id, status="COMPLETED", duration_ms=duration_ms, result_count=resp.results.len(), "external research execution completed");
            let rc = resp.results.len() as i64;
            Ok(Json(map_execution_row(exec_done, Some(rc))))
        }
        Err(err) => {
            let code = err.code.as_str().to_string();
            let msg = err.message.clone();
            let exec_failed = state
                .storage
                .update_execution_status(exec.id, "FAILED", Some(&code), Some(&msg), None, None)
                .await
                .map_err(|e| ApiError::internal(e.to_string()))?;
            tracing::error!(provider=%provider_name, query_id=query_id, execution_id=exec.id, status="FAILED", duration_ms=duration_ms, error_code=%code, "external research execution failed");
            // Map to normalized API error but also return execution status
            // If error is INVALID_QUERY, return 400 otherwise provider failure remains 200 with FAILED status
            // To differentiate, we return execution JSON with FAILED, but for client error we may also surface via ApiError?
            // Spec: provider failure ≠ successful. API should differentiate provider failure from success.
            // We return execution object with FAILED, not error code, to allow client to see history.
            // For INVALID_QUERY we return 400? But execution already created as FAILED. Choose to return execution JSON regardless.
            // However if INVALID_QUERY, spec expects bad request? We'll return execution json with 200 for consistency, unless provider not found etc already handled.
            let rc: Option<i64> = Some(0);
            // If code is INVALID_QUERY, we still return execution but also could map to 422? For now return execution.
            Ok(Json(map_execution_row(exec_failed, rc)))
        }
    }
}

// GET /api/v1/research-queries/:query_id/executions
pub async fn list_executions_generic(
    State(state): State<AppState>,
    Path(query_id): Path<i64>,
    Query(params): Query<PaginationOnly>,
) -> Result<Json<Paginated<serde_json::Value>>, ApiError> {
    if query_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "query_id must be >0"));
    }
    let query = state
        .storage
        .get_research_query(query_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "RESEARCH_QUERY_NOT_FOUND",
                format!("Query {query_id} not found"),
            )
        })?;
    let _ = query;
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
    let (rows, total) = state
        .storage
        .list_executions_for_query(query_id, limit, offset)
        .await?;
    let exec_ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
    let counts = state
        .storage
        .count_results_for_executions(&exec_ids)
        .await
        .unwrap_or_default();
    let items: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| {
            let rc = counts.get(&r.id).copied();
            map_execution_row(r, rc)
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

pub async fn list_executions_tree(
    State(state): State<AppState>,
    Path((tree_id, query_id)): Path<(i64, i64)>,
    Query(params): Query<PaginationOnly>,
) -> Result<Json<Paginated<serde_json::Value>>, ApiError> {
    if tree_id <= 0 || query_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} not found"))
    })?;
    let query = state
        .storage
        .get_research_query(query_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "RESEARCH_QUERY_NOT_FOUND",
                format!("Query {query_id} not found"),
            )
        })?;
    if query.tree_id != tree_id {
        return Err(ApiError::not_found(
            "RESEARCH_QUERY_NOT_FOUND",
            format!("Query {query_id} not in tree {tree_id}"),
        ));
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
    let (rows, total) = state
        .storage
        .list_executions_for_query(query_id, limit, offset)
        .await?;
    let exec_ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
    let counts = state
        .storage
        .count_results_for_executions(&exec_ids)
        .await
        .unwrap_or_default();
    let items: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|r| {
            let rc = counts.get(&r.id).copied();
            map_execution_row(r, rc)
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

// GET /api/v1/research-queries/:query_id/results
pub async fn list_results_generic(
    State(state): State<AppState>,
    Path(query_id): Path<i64>,
    Query(params): Query<PaginationOnly>,
) -> Result<Json<Paginated<serde_json::Value>>, ApiError> {
    if query_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "query_id must be >0"));
    }
    let query = state
        .storage
        .get_research_query(query_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "RESEARCH_QUERY_NOT_FOUND",
                format!("Query {query_id} not found"),
            )
        })?;
    let _ = query;
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
    let (rows, total) = state
        .storage
        .list_latest_results_for_query(query_id, limit, offset)
        .await?;
    let items: Vec<serde_json::Value> = rows.into_iter().map(map_result_row).collect();
    Ok(Json(Paginated {
        items,
        pagination: PaginationMeta {
            limit,
            offset,
            total,
        },
    }))
}

pub async fn list_results_tree(
    State(state): State<AppState>,
    Path((tree_id, query_id)): Path<(i64, i64)>,
    Query(params): Query<PaginationOnly>,
) -> Result<Json<Paginated<serde_json::Value>>, ApiError> {
    if tree_id <= 0 || query_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} not found"))
    })?;
    let query = state
        .storage
        .get_research_query(query_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "RESEARCH_QUERY_NOT_FOUND",
                format!("Query {query_id} not found"),
            )
        })?;
    if query.tree_id != tree_id {
        return Err(ApiError::not_found(
            "RESEARCH_QUERY_NOT_FOUND",
            format!("Query {query_id} not in tree {tree_id}"),
        ));
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
    let (rows, total) = state
        .storage
        .list_latest_results_for_query(query_id, limit, offset)
        .await?;
    let items: Vec<serde_json::Value> = rows.into_iter().map(map_result_row).collect();
    Ok(Json(Paginated {
        items,
        pagination: PaginationMeta {
            limit,
            offset,
            total,
        },
    }))
}

// GET /api/v1/research-results/:result_id
pub async fn get_result_generic(
    State(state): State<AppState>,
    Path(result_id): Path<i64>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if result_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "result_id must be >0"));
    }
    let row = state
        .storage
        .get_research_result(result_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "RESEARCH_RESULT_NOT_FOUND",
                format!("Result {result_id} not found"),
            )
        })?;
    Ok(Json(map_result_row(row)))
}

pub async fn get_result_tree(
    State(state): State<AppState>,
    Path((tree_id, result_id)): Path<(i64, i64)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if tree_id <= 0 || result_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} not found"))
    })?;
    let row = state
        .storage
        .get_research_result(result_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "RESEARCH_RESULT_NOT_FOUND",
                format!("Result {result_id} not found"),
            )
        })?;
    // tree_id isolation: result's query must belong to tree
    let query = state
        .storage
        .get_research_query(row.query_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "RESEARCH_RESULT_NOT_FOUND",
                format!("Result {result_id} not found"),
            )
        })?;
    if query.tree_id != tree_id {
        return Err(ApiError::not_found(
            "RESEARCH_RESULT_NOT_FOUND",
            format!("Result {result_id} not in tree {tree_id}"),
        ));
    }
    Ok(Json(map_result_row(row)))
}
