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
    pub assessment_status: Option<String>,
    pub gap: Option<String>,
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
        "updated_at": row.updated_at,
        "evidence": []
    })
}

async fn to_json_with_evidence(
    state: &crate::state::AppState,
    row: neogenealogy_storage::models::ResearchOutcomeRow,
) -> serde_json::Value {
    let evidence = state
        .storage
        .list_outcome_evidence_detailed(row.id)
        .await
        .unwrap_or_default();
    let assessment = state.storage.get_outcome_assessment(row.id).await.ok();
    let gaps = state
        .storage
        .get_outcome_gaps(row.id)
        .await
        .unwrap_or_default();
    let stats = state
        .storage
        .get_outcome_evidence_stats(row.id)
        .await
        .unwrap_or(neogenealogy_storage::assessment::EvidenceStats {
            evidence_total: 0,
            supporting_count: 0,
            contradicting_count: 0,
            sources_count: 0,
            cited_count: 0,
            uncited_count: 0,
            cited_supporting_count: 0,
        });
    let followups =
        neogenealogy_storage::assessment::calculate_research_followups(&row.r#type, &stats, &gaps);
    let actions = state
        .storage
        .list_outcome_followup_actions(row.id)
        .await
        .unwrap_or_default();
    let assessment_json = assessment.map(|a| {
        serde_json::json!({
            "score": a.score,
            "status": a.status,
            "evidence_total": a.evidence_total,
            "supporting_count": a.supporting_count,
            "contradicting_count": a.contradicting_count,
            "sources_count": a.sources_count,
            "cited_count": a.cited_count,
            "uncited_count": a.uncited_count,
            "cited_supporting_count": a.cited_supporting_count,
            "reasons": a.reasons.iter().map(|r| serde_json::json!({"code": r.code, "points": r.points, "message": r.message})).collect::<Vec<_>>()
        })
    }).unwrap_or(serde_json::json!(null));
    let gaps_json = gaps
        .iter()
        .map(|g| {
            serde_json::json!({
                "code": g.code,
                "severity": g.severity,
                "title": g.title,
                "description": g.description
            })
        })
        .collect::<Vec<_>>();
    let followups_json = followups
        .iter()
        .map(|f| {
            serde_json::json!({
                "code": f.code,
                "priority": f.priority,
                "title": f.title,
                "description": f.description,
                "gap_code": f.gap_code
            })
        })
        .collect::<Vec<_>>();
    let actions_json = actions
        .iter()
        .map(|a| {
            serde_json::json!({
                "id": a.id,
                "tree_id": a.tree_id,
                "task_id": a.task_id,
                "outcome_id": a.outcome_id,
                "followup_code": a.followup_code,
                "status": a.status,
                "notes": a.notes,
                "created_at": a.created_at,
                "updated_at": a.updated_at,
                "completed_at": a.completed_at
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "id": row.id,
        "tree_id": row.tree_id,
        "task_id": row.task_id,
        "type": row.r#type,
        "summary": row.summary,
        "details": row.details,
        "created_at": row.created_at,
        "updated_at": row.updated_at,
        "evidence": evidence,
        "evidence_assessment": assessment_json,
        "evidence_gaps": gaps_json,
        "research_followups": followups_json,
        "followup_actions": actions_json
    })
}

fn validate_assessment_status(s: &str) -> Result<(), ApiError> {
    let allowed = [
        "NO_EVIDENCE",
        "WEAK",
        "MIXED",
        "SUPPORTED",
        "STRONGLY_SUPPORTED",
    ];
    if !allowed.contains(&s) {
        return Err(ApiError::bad_request(
            "INVALID_ASSESSMENT_STATUS",
            format!("assessment_status must be one of {}", allowed.join(",")),
        ));
    }
    Ok(())
}

fn validate_gap(s: &str) -> Result<(), ApiError> {
    let allowed = [
        "NO_SUPPORTING_EVIDENCE",
        "NO_CITATION",
        "SINGLE_SUPPORTING_EVIDENCE",
        "CONTRADICTORY_EVIDENCE",
        "SINGLE_SOURCE",
        "CONFIRMED_WITHOUT_SUPPORT",
    ];
    if !allowed.contains(&s) {
        return Err(ApiError::bad_request(
            "INVALID_GAP_CODE",
            format!("gap must be one of {}", allowed.join(",")),
        ));
    }
    Ok(())
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
    if let Some(ref a) = params.assessment_status {
        validate_assessment_status(&a.to_uppercase())?;
    }
    if let Some(ref g) = params.gap {
        validate_gap(&g.to_uppercase())?;
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
    let assessment_filter = params
        .assessment_status
        .as_deref()
        .map(|s| s.to_uppercase());
    let gap_filter = params.gap.as_deref().map(|s| s.to_uppercase());

    // If assessment or gap filter present, need to fetch all and filter in Rust to avoid N+1, then paginate
    if assessment_filter.is_some() || gap_filter.is_some() {
        // fetch all matching other filters (without pagination)
        let (all_rows, _) = if params.person_id.is_some() {
            state
                .storage
                .list_research_outcomes_with_person(
                    tree_id,
                    ttype.as_deref(),
                    params.person_id,
                    1000,
                    0,
                )
                .await?
        } else {
            state
                .storage
                .list_research_outcomes(tree_id, ttype.as_deref(), params.task_id, 1000, 0)
                .await?
        };
        let ids: Vec<i64> = all_rows.iter().map(|r| r.id).collect();
        let assessments = state
            .storage
            .get_outcomes_assessments(&ids)
            .await
            .unwrap_or_default();
        let gaps_map = state
            .storage
            .get_outcomes_gaps(&ids)
            .await
            .unwrap_or_default();
        let stats_map = state
            .storage
            .get_outcomes_evidence_stats(&ids)
            .await
            .unwrap_or_default();
        let actions_counts = state
            .storage
            .get_outcomes_followup_actions_counts(&ids)
            .await
            .unwrap_or_default();
        let mut filtered: Vec<serde_json::Value> = Vec::new();
        for row in all_rows {
            // assessment filter
            if let Some(ref filter_status) = assessment_filter {
                let status = assessments
                    .get(&row.id)
                    .map(|a| a.status.as_str())
                    .unwrap_or("NO_EVIDENCE");
                if status != filter_status.as_str() {
                    continue;
                }
            }
            // gap filter
            if let Some(ref filter_gap) = gap_filter {
                let gaps = gaps_map.get(&row.id);
                let has_gap = gaps
                    .map(|v| v.iter().any(|g| g.code == filter_gap.as_str()))
                    .unwrap_or(false);
                if !has_gap {
                    continue;
                }
            }
            // enrich with assessment, evidence and gaps
            let evidence = state
                .storage
                .list_outcome_evidence_detailed(row.id)
                .await
                .unwrap_or_default();
            let assessment_json = assessments.get(&row.id).map(|a| {
                serde_json::json!({
                    "score": a.score,
                    "status": a.status,
                    "evidence_total": a.evidence_total,
                    "supporting_count": a.supporting_count,
                    "contradicting_count": a.contradicting_count,
                    "sources_count": a.sources_count,
                    "cited_count": a.cited_count,
                    "uncited_count": a.uncited_count,
                    "cited_supporting_count": a.cited_supporting_count,
                    "reasons": a.reasons.iter().map(|r| serde_json::json!({"code": r.code, "points": r.points, "message": r.message})).collect::<Vec<_>>()
                })
            }).unwrap_or(serde_json::json!(null));
            let gaps_json = gaps_map.get(&row.id).map(|v| {
                v.iter().map(|g| serde_json::json!({"code": g.code, "severity": g.severity, "title": g.title, "description": g.description})).collect::<Vec<_>>()
            }).unwrap_or_default();
            let gaps_for_fu = gaps_map.get(&row.id).cloned().unwrap_or_default();
            let stats_for_fu = stats_map.get(&row.id).cloned().unwrap_or(
                neogenealogy_storage::assessment::EvidenceStats {
                    evidence_total: 0,
                    supporting_count: 0,
                    contradicting_count: 0,
                    sources_count: 0,
                    cited_count: 0,
                    uncited_count: 0,
                    cited_supporting_count: 0,
                },
            );
            let row_type = row.r#type.clone();
            let followups = neogenealogy_storage::assessment::calculate_research_followups(
                &row_type,
                &stats_for_fu,
                &gaps_for_fu,
            );
            let followups_json = followups.iter().map(|f| serde_json::json!({"code": f.code, "priority": f.priority, "title": f.title, "description": f.description, "gap_code": f.gap_code})).collect::<Vec<_>>();
            let actions_count = actions_counts.get(&row.id).cloned().unwrap_or(0);
            let mut v = to_json(row);
            v["evidence"] = serde_json::json!(evidence);
            v["evidence_assessment"] = assessment_json;
            v["evidence_gaps"] = serde_json::json!(gaps_json);
            v["research_followups"] = serde_json::json!(followups_json);
            v["followup_actions_count"] = serde_json::json!(actions_count);
            filtered.push(v);
        }
        let total = filtered.len() as i64;
        let paginated: Vec<serde_json::Value> = filtered
            .into_iter()
            .skip(offset as usize)
            .take(limit as usize)
            .collect();
        return Ok(Json(Paginated {
            items: paginated,
            pagination: PaginationMeta {
                limit,
                offset,
                total,
            },
        }));
    }

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
    // batch assessment + gaps for listed rows
    let ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
    let assessments = state
        .storage
        .get_outcomes_assessments(&ids)
        .await
        .unwrap_or_default();
    let gaps_map = state
        .storage
        .get_outcomes_gaps(&ids)
        .await
        .unwrap_or_default();
    let stats_map = state
        .storage
        .get_outcomes_evidence_stats(&ids)
        .await
        .unwrap_or_default();
    let actions_counts = state
        .storage
        .get_outcomes_followup_actions_counts(&ids)
        .await
        .unwrap_or_default();
    let mut items = Vec::new();
    for row in rows {
        let evidence = state
            .storage
            .list_outcome_evidence_detailed(row.id)
            .await
            .unwrap_or_default();
        let assessment = assessments.get(&row.id);
        let assessment_json = assessment.map(|a| {
            serde_json::json!({
                "score": a.score,
                "status": a.status,
                "evidence_total": a.evidence_total,
                "supporting_count": a.supporting_count,
                "contradicting_count": a.contradicting_count,
                "sources_count": a.sources_count,
                "cited_count": a.cited_count,
                "uncited_count": a.uncited_count,
                "cited_supporting_count": a.cited_supporting_count,
                "reasons": a.reasons.iter().map(|r| serde_json::json!({"code": r.code, "points": r.points, "message": r.message})).collect::<Vec<_>>()
            })
        }).unwrap_or(serde_json::json!({
            "score": 0,
            "status": "NO_EVIDENCE",
            "evidence_total": 0,
            "supporting_count": 0,
            "contradicting_count": 0,
            "sources_count": 0,
            "cited_count": 0,
            "uncited_count": 0,
            "cited_supporting_count": 0,
            "reasons": []
        }));
        let gaps_json = gaps_map.get(&row.id).map(|v| {
            v.iter().map(|g| serde_json::json!({"code": g.code, "severity": g.severity, "title": g.title, "description": g.description})).collect::<Vec<_>>()
        }).unwrap_or_default();
        let gaps_for_fu = gaps_map.get(&row.id).cloned().unwrap_or_default();
        let stats_for_fu = stats_map.get(&row.id).cloned().unwrap_or(
            neogenealogy_storage::assessment::EvidenceStats {
                evidence_total: 0,
                supporting_count: 0,
                contradicting_count: 0,
                sources_count: 0,
                cited_count: 0,
                uncited_count: 0,
                cited_supporting_count: 0,
            },
        );
        let row_type_clone = row.r#type.clone();
        let followups = neogenealogy_storage::assessment::calculate_research_followups(
            &row_type_clone,
            &stats_for_fu,
            &gaps_for_fu,
        );
        let followups_json = followups.iter().map(|f| serde_json::json!({"code": f.code, "priority": f.priority, "title": f.title, "description": f.description, "gap_code": f.gap_code})).collect::<Vec<_>>();
        let actions_count = actions_counts.get(&row.id).cloned().unwrap_or(0);
        let mut v = to_json(row);
        v["evidence"] = serde_json::json!(evidence);
        v["evidence_assessment"] = assessment_json;
        v["evidence_gaps"] = serde_json::json!(gaps_json);
        v["research_followups"] = serde_json::json!(followups_json);
        v["followup_actions_count"] = serde_json::json!(actions_count);
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
    Ok(Json(to_json_with_evidence(&state, row).await))
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
