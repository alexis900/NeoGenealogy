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
pub struct CitationListParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize, Serialize)]
pub struct CreateCitationBody {
    pub locator: Option<String>,
    pub text: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateCitationBody {
    pub locator: Option<String>,
    pub text: Option<String>,
}

fn to_json(row: neogenealogy_storage::models::ResearchCitationRow) -> serde_json::Value {
    serde_json::json!({
        "id": row.id,
        "source_id": row.source_id,
        "locator": row.locator,
        "text": row.text,
        "created_at": row.created_at,
        "updated_at": row.updated_at
    })
}

pub async fn list_citations(
    State(state): State<AppState>,
    Path((tree_id, source_id)): Path<(i64, i64)>,
    Query(params): Query<CitationListParams>,
) -> Result<Json<Paginated<serde_json::Value>>, ApiError> {
    if tree_id <= 0 || source_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    let source = state
        .storage
        .get_research_source(source_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found("SOURCE_NOT_FOUND", format!("Source {source_id} not found"))
        })?;
    if source.tree_id != tree_id {
        return Err(ApiError::not_found(
            "SOURCE_NOT_FOUND",
            format!("Source {source_id} not in tree {tree_id}"),
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
        .list_research_citations(source_id, limit, offset)
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

pub async fn get_citation(
    State(state): State<AppState>,
    Path((tree_id, citation_id)): Path<(i64, i64)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if tree_id <= 0 || citation_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    let row = state
        .storage
        .get_research_citation(citation_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "CITATION_NOT_FOUND",
                format!("Citation {citation_id} not found"),
            )
        })?;
    // need to verify via source tree
    let source = state
        .storage
        .get_research_source(row.source_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "SOURCE_NOT_FOUND",
                format!("Source {} not found", row.source_id),
            )
        })?;
    if source.tree_id != tree_id {
        return Err(ApiError::not_found(
            "CITATION_NOT_FOUND",
            format!("Citation {citation_id} not in tree {tree_id}"),
        ));
    }
    Ok(Json(to_json(row)))
}

pub async fn create_citation(
    State(state): State<AppState>,
    Path((tree_id, source_id)): Path<(i64, i64)>,
    Json(body): Json<CreateCitationBody>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    if tree_id <= 0 || source_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    let source = state
        .storage
        .get_research_source(source_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found("SOURCE_NOT_FOUND", format!("Source {source_id} not found"))
        })?;
    if source.tree_id != tree_id {
        return Err(ApiError::not_found(
            "SOURCE_NOT_FOUND",
            format!("Source {source_id} not in tree {tree_id}"),
        ));
    }
    let row = state
        .storage
        .create_research_citation(source_id, body.locator.as_deref(), body.text.as_deref())
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok((StatusCode::CREATED, Json(to_json(row))))
}

pub async fn update_citation(
    State(state): State<AppState>,
    Path((tree_id, citation_id)): Path<(i64, i64)>,
    Json(body): Json<UpdateCitationBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if tree_id <= 0 || citation_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    let existing = state
        .storage
        .get_research_citation(citation_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "CITATION_NOT_FOUND",
                format!("Citation {citation_id} not found"),
            )
        })?;
    let source = state
        .storage
        .get_research_source(existing.source_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "SOURCE_NOT_FOUND",
                format!("Source {} not found", existing.source_id),
            )
        })?;
    if source.tree_id != tree_id {
        return Err(ApiError::not_found(
            "CITATION_NOT_FOUND",
            format!("Citation {citation_id} not in tree {tree_id}"),
        ));
    }
    let row = state
        .storage
        .update_research_citation(citation_id, body.locator.as_deref(), body.text.as_deref())
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;
    Ok(Json(to_json(row)))
}

pub async fn delete_citation(
    State(state): State<AppState>,
    Path((tree_id, citation_id)): Path<(i64, i64)>,
) -> Result<StatusCode, ApiError> {
    if tree_id <= 0 || citation_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    let existing = state
        .storage
        .get_research_citation(citation_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "CITATION_NOT_FOUND",
                format!("Citation {citation_id} not found"),
            )
        })?;
    let source = state
        .storage
        .get_research_source(existing.source_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "SOURCE_NOT_FOUND",
                format!("Source {} not found", existing.source_id),
            )
        })?;
    if source.tree_id != tree_id {
        return Err(ApiError::not_found(
            "CITATION_NOT_FOUND",
            format!("Citation {citation_id} not in tree {tree_id}"),
        ));
    }
    state.storage.delete_research_citation(citation_id).await?;
    Ok(StatusCode::NO_CONTENT)
}
