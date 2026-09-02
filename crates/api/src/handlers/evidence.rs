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
pub struct EvidenceListParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Deserialize, Serialize)]
pub struct CreateEvidenceBody {
    pub source_id: i64,
    pub citation_id: Option<i64>,
    pub statement: String,
    pub notes: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct UpdateEvidenceBody {
    pub statement: Option<String>,
    pub notes: Option<String>,
    pub citation_id: Option<i64>,
}

#[derive(Deserialize, Serialize)]
pub struct AttachBody {
    pub relationship: String,
}

fn to_json(row: neogenealogy_storage::models::EvidenceRow) -> serde_json::Value {
    serde_json::json!({
        "id": row.id,
        "tree_id": row.tree_id,
        "source_id": row.source_id,
        "citation_id": row.citation_id,
        "statement": row.statement,
        "notes": row.notes,
        "created_at": row.created_at,
        "updated_at": row.updated_at
    })
}

pub async fn list_evidence(
    State(state): State<AppState>,
    Path(tree_id): Path<i64>,
    Query(params): Query<EvidenceListParams>,
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
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    if !(0..=100).contains(&limit) {
        return Err(ApiError::bad_request("INVALID_LIMIT", "limit 0..100"));
    }
    if offset < 0 {
        return Err(ApiError::bad_request("INVALID_OFFSET", "offset >=0"));
    }
    let (rows, total) = state.storage.list_evidence(tree_id, limit, offset).await?;
    // enrich with source/citation info efficiently: batch fetch sources?
    let mut items = Vec::new();
    for row in rows {
        let source = state
            .storage
            .get_research_source(row.source_id)
            .await
            .ok()
            .flatten();
        let citation = if let Some(cid) = row.citation_id {
            state
                .storage
                .get_research_citation(cid)
                .await
                .ok()
                .flatten()
        } else {
            None
        };
        let mut v = to_json(row.clone());
        if let Some(s) = source {
            v["source"] = serde_json::json!({"id": s.id, "title": s.title, "type": s.r#type});
        }
        if let Some(c) = citation {
            v["citation"] = serde_json::json!({"id": c.id, "locator": c.locator});
        }
        items.push(v);
    }
    Ok(Json(Paginated {
        items,
        pagination: PaginationMeta {
            limit,
            offset,
            total,
        },
    }))
}

pub async fn get_evidence(
    State(state): State<AppState>,
    Path((tree_id, evidence_id)): Path<(i64, i64)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if tree_id <= 0 || evidence_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    let row = state
        .storage
        .get_evidence(evidence_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "EVIDENCE_NOT_FOUND",
                format!("Evidence {evidence_id} not found"),
            )
        })?;
    if row.tree_id != tree_id {
        return Err(ApiError::not_found(
            "EVIDENCE_NOT_FOUND",
            format!("Evidence {evidence_id} not in tree {tree_id}"),
        ));
    }
    let source = state
        .storage
        .get_research_source(row.source_id)
        .await
        .ok()
        .flatten();
    let citation = if let Some(cid) = row.citation_id {
        state
            .storage
            .get_research_citation(cid)
            .await
            .ok()
            .flatten()
    } else {
        None
    };
    let mut v = to_json(row);
    if let Some(s) = source {
        v["source"] = serde_json::json!({"id": s.id, "title": s.title, "type": s.r#type});
    }
    if let Some(c) = citation {
        v["citation"] = serde_json::json!({"id": c.id, "locator": c.locator, "text": c.text});
    }
    Ok(Json(v))
}

pub async fn create_evidence(
    State(state): State<AppState>,
    Path(tree_id): Path<i64>,
    Json(body): Json<CreateEvidenceBody>,
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
    if body.statement.trim().is_empty() {
        return Err(ApiError::bad_request(
            "INVALID_STATEMENT",
            "statement must not be empty",
        ));
    }
    let row = state
        .storage
        .create_evidence(
            tree_id,
            body.source_id,
            body.citation_id,
            &body.statement,
            body.notes.as_deref(),
        )
        .await
        .map_err(|e| match e {
            neogenealogy_storage::StorageError::NotFound(msg) if msg.contains("source") => {
                ApiError::not_found("SOURCE_NOT_FOUND", msg)
            }
            neogenealogy_storage::StorageError::NotFound(msg) if msg.contains("citation") => {
                ApiError::not_found("CITATION_NOT_FOUND", msg)
            }
            neogenealogy_storage::StorageError::Import(msg) if msg.contains("statement") => {
                ApiError::bad_request("INVALID_STATEMENT", msg)
            }
            neogenealogy_storage::StorageError::Import(msg)
                if msg.contains("citation does not belong") =>
            {
                ApiError::bad_request("INVALID_CITATION", msg)
            }
            other => ApiError::internal(other.to_string()),
        })?;
    Ok((StatusCode::CREATED, Json(to_json(row))))
}

pub async fn update_evidence(
    State(state): State<AppState>,
    Path((tree_id, evidence_id)): Path<(i64, i64)>,
    Json(body): Json<UpdateEvidenceBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if tree_id <= 0 || evidence_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    let existing = state
        .storage
        .get_evidence(evidence_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "EVIDENCE_NOT_FOUND",
                format!("Evidence {evidence_id} not found"),
            )
        })?;
    if existing.tree_id != tree_id {
        return Err(ApiError::not_found(
            "EVIDENCE_NOT_FOUND",
            format!("Evidence {evidence_id} not in tree {tree_id}"),
        ));
    }
    if let Some(ref s) = body.statement {
        if s.trim().is_empty() {
            return Err(ApiError::bad_request(
                "INVALID_STATEMENT",
                "statement must not be empty",
            ));
        }
    }
    // citation_id handling: if provided as Some, set; if null we need to allow clearing - but UpdateEvidenceBody uses Option<i64> so None means not update, need to handle clearing? For now, only set if Some.
    let citation_opt = body.citation_id.map(Some);
    let row = state
        .storage
        .update_evidence(
            evidence_id,
            body.statement.as_deref(),
            body.notes.as_deref(),
            citation_opt,
        )
        .await
        .map_err(|e| match e {
            neogenealogy_storage::StorageError::NotFound(msg) => {
                ApiError::not_found("CITATION_NOT_FOUND", msg)
            }
            neogenealogy_storage::StorageError::Import(msg) => {
                ApiError::bad_request("INVALID_CITATION", msg)
            }
            other => ApiError::internal(other.to_string()),
        })?;
    Ok(Json(to_json(row)))
}

pub async fn delete_evidence(
    State(state): State<AppState>,
    Path((tree_id, evidence_id)): Path<(i64, i64)>,
) -> Result<StatusCode, ApiError> {
    if tree_id <= 0 || evidence_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    let existing = state
        .storage
        .get_evidence(evidence_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "EVIDENCE_NOT_FOUND",
                format!("Evidence {evidence_id} not found"),
            )
        })?;
    if existing.tree_id != tree_id {
        return Err(ApiError::not_found(
            "EVIDENCE_NOT_FOUND",
            format!("Evidence {evidence_id} not in tree {tree_id}"),
        ));
    }
    state.storage.delete_evidence(evidence_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_outcome_evidence(
    State(state): State<AppState>,
    Path((tree_id, outcome_id)): Path<(i64, i64)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if tree_id <= 0 || outcome_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    let outcome = state
        .storage
        .get_research_outcome(outcome_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "OUTCOME_NOT_FOUND",
                format!("Outcome {outcome_id} not found"),
            )
        })?;
    if outcome.tree_id != tree_id {
        return Err(ApiError::not_found(
            "OUTCOME_NOT_FOUND",
            format!("Outcome {outcome_id} not in tree {tree_id}"),
        ));
    }
    let detailed = state
        .storage
        .list_outcome_evidence_detailed(outcome_id)
        .await?;
    Ok(Json(serde_json::json!({ "items": detailed })))
}

pub async fn attach_evidence(
    State(state): State<AppState>,
    Path((tree_id, outcome_id, evidence_id)): Path<(i64, i64, i64)>,
    Json(body): Json<AttachBody>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    if tree_id <= 0 || outcome_id <= 0 || evidence_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    let rel = body.relationship.to_uppercase();
    if !["SUPPORTS", "CONTRADICTS"].contains(&rel.as_str()) {
        return Err(ApiError::bad_request(
            "INVALID_EVIDENCE_RELATIONSHIP",
            "relationship must be SUPPORTS or CONTRADICTS",
        ));
    }
    let row = state
        .storage
        .attach_evidence_to_outcome(outcome_id, evidence_id, &rel)
        .await
        .map_err(|e| match e {
            neogenealogy_storage::StorageError::NotFound(msg) if msg.contains("outcome") => {
                ApiError::not_found("OUTCOME_NOT_FOUND", msg)
            }
            neogenealogy_storage::StorageError::NotFound(msg) if msg.contains("evidence") => {
                ApiError::not_found("EVIDENCE_NOT_FOUND", msg)
            }
            neogenealogy_storage::StorageError::NotFound(msg) => {
                ApiError::not_found("NOT_FOUND", msg)
            }
            neogenealogy_storage::StorageError::Import(msg) if msg.contains("already attached") => {
                ApiError {
                    status: StatusCode::CONFLICT,
                    code: "EVIDENCE_ALREADY_ATTACHED",
                    message: msg,
                }
            }
            neogenealogy_storage::StorageError::Import(msg) => {
                ApiError::bad_request("INVALID_EVIDENCE_RELATIONSHIP", msg)
            }
            other => ApiError::internal(other.to_string()),
        })?;
    Ok((
        StatusCode::CREATED,
        Json(
            serde_json::json!({"outcome_id": row.outcome_id, "evidence_id": row.evidence_id, "relationship": row.relationship}),
        ),
    ))
}

pub async fn detach_evidence(
    State(state): State<AppState>,
    Path((tree_id, outcome_id, evidence_id)): Path<(i64, i64, i64)>,
) -> Result<StatusCode, ApiError> {
    if tree_id <= 0 || outcome_id <= 0 || evidence_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;
    let outcome = state
        .storage
        .get_research_outcome(outcome_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "OUTCOME_NOT_FOUND",
                format!("Outcome {outcome_id} not found"),
            )
        })?;
    if outcome.tree_id != tree_id {
        return Err(ApiError::not_found(
            "OUTCOME_NOT_FOUND",
            format!("Outcome {outcome_id} not in tree {tree_id}"),
        ));
    }
    // verify evidence exists and same tree
    let evidence = state
        .storage
        .get_evidence(evidence_id)
        .await?
        .ok_or_else(|| {
            ApiError::not_found(
                "EVIDENCE_NOT_FOUND",
                format!("Evidence {evidence_id} not found"),
            )
        })?;
    if evidence.tree_id != tree_id {
        return Err(ApiError::not_found(
            "EVIDENCE_NOT_FOUND",
            format!("Evidence {evidence_id} not in tree {tree_id}"),
        ));
    }
    state
        .storage
        .detach_evidence_from_outcome(outcome_id, evidence_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
