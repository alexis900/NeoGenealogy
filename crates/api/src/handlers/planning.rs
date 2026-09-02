use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;

use crate::{error::ApiError, state::AppState};

#[derive(Debug, Deserialize)]
pub struct PlanParams {
    pub limit: Option<usize>,
    pub min_score: Option<f64>,
    pub priority: Option<String>,
    pub researchability: Option<String>,
}

fn validate_priority(s: &str) -> Result<(), ApiError> {
    let allowed = ["low", "info", "medium", "warning", "high", "critical"];
    if !allowed.contains(&s.to_lowercase().as_str()) {
        return Err(ApiError::bad_request(
            "INVALID_PRIORITY",
            format!("priority must be one of {}", allowed.join(",")),
        ));
    }
    Ok(())
}

fn validate_researchability(s: &str) -> Result<(), ApiError> {
    let allowed = ["low", "medium", "high"];
    if !allowed.contains(&s.to_lowercase().as_str()) {
        return Err(ApiError::bad_request(
            "INVALID_RESEARCHABILITY",
            format!("researchability must be one of {}", allowed.join(",")),
        ));
    }
    Ok(())
}

pub async fn get_research_plan(
    State(state): State<AppState>,
    Path(tree_id): Path<i64>,
    Query(params): Query<PlanParams>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if tree_id <= 0 {
        return Err(ApiError::bad_request(
            "INVALID_TREE_ID",
            "tree_id must be >0",
        ));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;

    if let Some(ref p) = params.priority {
        validate_priority(p)?;
    }
    if let Some(ref r) = params.researchability {
        validate_researchability(r)?;
    }
    if let Some(ms) = params.min_score {
        if !(0.0..=100.0).contains(&ms) {
            return Err(ApiError::bad_request(
                "INVALID_MIN_SCORE",
                "min_score 0..100",
            ));
        }
    }
    let limit = params.limit.unwrap_or(10);
    if limit == 0 || limit > 100 {
        return Err(ApiError::bad_request(
            "INVALID_LIMIT",
            "limit must be 1..100",
        ));
    }

    // Fetch candidates efficiently (no N+1)
    let mut candidates = state
        .storage
        .get_research_planning_candidates(tree_id)
        .await
        .map_err(|e| ApiError::internal(e.to_string()))?;

    // Apply priority / researchability filters before scoring
    if let Some(ref p) = params.priority {
        let lower = p.to_lowercase();
        candidates.retain(|c| c.priority.to_lowercase() == lower);
    }
    if let Some(ref r) = params.researchability {
        let lower = r.to_lowercase();
        candidates.retain(|c| {
            c.researchability
                .as_deref()
                .map(|s| s.to_lowercase() == lower)
                .unwrap_or(false)
        });
    }

    // Calculate plan (pure)
    let mut plan = neogenealogy_storage::planning::calculate_research_plan(candidates, limit);

    // Apply min_score filter to planning_score if provided – filter both lists
    if let Some(ms) = params.min_score {
        plan.recommended.retain(|i| i.planning_score >= ms);
        plan.deferred.retain(|i| i.planning_score >= ms);
        // Recompute summary counts after min_score filtering? Keep original total_candidates but update derived counts
        // For simplicity, recompute summary based on filtered items (recommended+deferred)
        let all: Vec<_> = plan
            .recommended
            .iter()
            .chain(plan.deferred.iter())
            .cloned()
            .collect();
        let total = all.len();
        let active_count = all
            .iter()
            .filter(|i| matches!(i.task_status.as_deref(), Some("OPEN") | Some("IN_PROGRESS")))
            .count();
        let inconclusive_count = all
            .iter()
            .filter(|i| i.task_status.as_deref() == Some("INCONCLUSIVE"))
            .count();
        let high_priority_count = all
            .iter()
            .filter(|i| {
                let p = i.priority.to_uppercase();
                p == "HIGH" || p == "CRITICAL"
            })
            .count();
        let critical_gap_count = all
            .iter()
            .filter(|i| i.reasons.iter().any(|r| r.code == "CRITICAL_EVIDENCE_GAP"))
            .count();
        plan.total_candidates = total;
        plan.summary = neogenealogy_storage::planning::ResearchPlanSummary {
            total_candidates: total,
            recommended_count: plan.recommended.len(),
            deferred_count: plan.deferred.len(),
            active_count,
            inconclusive_count,
            high_priority_count,
            critical_gap_count,
        };
    }

    // Serialize to expected API shape
    let resp = serde_json::json!({
        "generated_at": plan.generated_at,
        "total_candidates": plan.total_candidates,
        "summary": {
            "total_candidates": plan.summary.total_candidates,
            "recommended_count": plan.summary.recommended_count,
            "deferred_count": plan.summary.deferred_count,
            "active_count": plan.summary.active_count,
            "inconclusive_count": plan.summary.inconclusive_count,
            "high_priority_count": plan.summary.high_priority_count,
            "critical_gap_count": plan.summary.critical_gap_count
        },
        "recommended": plan.recommended.iter().map(|item| serde_json::json!({
            "opportunity_id": item.opportunity_id,
            "person_id": item.person_id,
            "title": item.title,
            "priority": item.priority,
            "research_score": item.research_score,
            "planning_score": item.planning_score,
            "researchability": item.researchability,
            "confidence": item.confidence,
            "active_task": item.active_task,
            "task_status": item.task_status,
            "reasons": item.reasons
        })).collect::<Vec<_>>(),
        "deferred": plan.deferred.iter().map(|item| serde_json::json!({
            "opportunity_id": item.opportunity_id,
            "person_id": item.person_id,
            "title": item.title,
            "priority": item.priority,
            "research_score": item.research_score,
            "planning_score": item.planning_score,
            "researchability": item.researchability,
            "confidence": item.confidence,
            "active_task": item.active_task,
            "task_status": item.task_status,
            "reasons": item.reasons
        })).collect::<Vec<_>>()
    });

    Ok(Json(resp))
}
