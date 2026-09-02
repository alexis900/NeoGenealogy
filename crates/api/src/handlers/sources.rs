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
pub struct SourceListParams {
    pub r#type: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize, Serialize)]
pub struct CreateSourceBody {
    pub title: String,
    pub author: Option<String>,
    pub publication: Option<String>,
    pub date: Option<String>,
    pub r#type: String,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateSourceBody {
    pub title: Option<String>,
    pub author: Option<String>,
    pub publication: Option<String>,
    pub date: Option<String>,
    pub r#type: Option<String>,
}

fn validate_type(s: &str) -> Result<(), ApiError> {
    let allowed = [
        "BOOK",
        "REGISTER",
        "CENSUS",
        "CIVIL_RECORD",
        "PARISH_RECORD",
        "NEWSPAPER",
        "WEBSITE",
        "OTHER",
    ];
    if !allowed.contains(&s) {
        return Err(ApiError::bad_request(
            "INVALID_SOURCE_TYPE",
            format!("type must be one of {}", allowed.join(",")),
        ));
    }
    Ok(())
}

fn to_json(row: neogenealogy_storage::models::ResearchSourceRow) -> serde_json::Value {
    serde_json::json!({
        "id": row.id,
        "tree_id": row.tree_id,
        "title": row.title,
        "author": row.author,
        "publication": row.publication,
        "date": row.date,
        "type": row.r#type,
        "created_at": row.created_at,
        "updated_at": row.updated_at
    })
}

pub async fn list_sources(
    State(state): State<AppState>,
    Path(tree_id): Path<i64>,
    Query(params): Query<SourceListParams>,
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
    let (rows, total) = state
        .storage
        .list_research_sources(tree_id, ttype.as_deref(), limit, offset)
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

pub async fn get_source(
    State(state): State<AppState>,
    Path((tree_id, source_id)): Path<(i64, i64)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if tree_id <= 0 || source_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    let row = state
        .storage
        .get_research_source(source_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found("SOURCE_NOT_FOUND", format!("Source {source_id} not found"))
        })?;
    if row.tree_id != tree_id {
        return Err(ApiError::not_found(
            "SOURCE_NOT_FOUND",
            format!("Source {source_id} not in tree {tree_id}"),
        ));
    }
    Ok(Json(to_json(row)))
}

pub async fn create_source(
    State(state): State<AppState>,
    Path(tree_id): Path<i64>,
    Json(body): Json<CreateSourceBody>,
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
    validate_type(&body.r#type.to_uppercase())?;
    if body.title.trim().is_empty() {
        return Err(ApiError::bad_request(
            "INVALID_TITLE",
            "title must not be empty",
        ));
    }
    let row = state
        .storage
        .create_research_source(
            tree_id,
            &body.title,
            body.author.as_deref(),
            body.publication.as_deref(),
            body.date.as_deref(),
            &body.r#type.to_uppercase(),
        )
        .await
        .map_err(|e| match e {
            neogenealogy_storage::StorageError::Import(msg)
                if msg.contains("invalid source type") =>
            {
                ApiError::bad_request("INVALID_SOURCE_TYPE", msg)
            }
            neogenealogy_storage::StorageError::Import(msg) => {
                ApiError::bad_request("INVALID_TITLE", msg)
            }
            other => ApiError::internal(other.to_string()),
        })?;
    Ok((StatusCode::CREATED, Json(to_json(row))))
}

pub async fn update_source(
    State(state): State<AppState>,
    Path((tree_id, source_id)): Path<(i64, i64)>,
    Json(body): Json<UpdateSourceBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if tree_id <= 0 || source_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    if let Some(ref t) = body.r#type {
        validate_type(&t.to_uppercase())?;
    }
    if let Some(ref title) = body.title {
        if title.trim().is_empty() {
            return Err(ApiError::bad_request(
                "INVALID_TITLE",
                "title must not be empty",
            ));
        }
    }
    let existing = state
        .storage
        .get_research_source(source_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found("SOURCE_NOT_FOUND", format!("Source {source_id} not found"))
        })?;
    if existing.tree_id != tree_id {
        return Err(ApiError::not_found(
            "SOURCE_NOT_FOUND",
            format!("Source {source_id} not in tree {tree_id}"),
        ));
    }
    let row = state
        .storage
        .update_research_source(
            source_id,
            body.title.as_deref(),
            body.author.as_deref(),
            body.publication.as_deref(),
            body.date.as_deref(),
            body.r#type.as_deref().map(|s| s.to_uppercase()).as_deref(),
        )
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(to_json(row)))
}

pub async fn delete_source(
    State(state): State<AppState>,
    Path((tree_id, source_id)): Path<(i64, i64)>,
) -> Result<StatusCode, ApiError> {
    if tree_id <= 0 || source_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    let existing = state
        .storage
        .get_research_source(source_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found("SOURCE_NOT_FOUND", format!("Source {source_id} not found"))
        })?;
    if existing.tree_id != tree_id {
        return Err(ApiError::not_found(
            "SOURCE_NOT_FOUND",
            format!("Source {source_id} not in tree {tree_id}"),
        ));
    }
    state.storage.delete_research_source(source_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
