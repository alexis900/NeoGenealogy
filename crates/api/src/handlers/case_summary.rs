use axum::{
    extract::{Path, State},
    Json,
};

use crate::{error::ApiError, state::AppState};

pub async fn get_case_summary(
    State(state): State<AppState>,
    Path((tree_id, task_id)): Path<(i64, i64)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if tree_id <= 0 || task_id <= 0 {
        return Err(ApiError::bad_request("INVALID_ID", "ids must be >0"));
    }
    state.storage.get_tree(tree_id).await?.ok_or_else(|| {
        ApiError::not_found("TREE_NOT_FOUND", format!("Tree {tree_id} was not found"))
    })?;

    let summary = state
        .storage
        .get_research_case_summary(tree_id, task_id)
        .await
        .map_err(|e| match e {
            neogenealogy_storage::StorageError::NotFound(msg) => {
                if msg.contains("task") {
                    ApiError::not_found("TASK_NOT_FOUND", msg)
                } else {
                    ApiError::not_found("NOT_FOUND", msg)
                }
            }
            other => ApiError::internal(other.to_string()),
        })?;

    // Build JSON response exactly as spec
    let task_json = serde_json::json!({
        "id": summary.task.id,
        "title": summary.task.title,
        "description": summary.task.description,
        "status": summary.task.status,
        "resolution": summary.task.resolution,
        "created_at": summary.task.created_at,
        "started_at": summary.task.started_at,
        "completed_at": summary.task.completed_at,
        "updated_at": summary.task.updated_at,
        "tree_id": tree_id,
    });

    let person_json = match summary.person {
        Some(p) => serde_json::json!({"person_id": p.person_id, "person_name": p.person_name}),
        None => serde_json::Value::Null,
    };

    let opportunity_json = match summary.opportunity {
        Some(o) => serde_json::json!({
            "opportunity_id": o.opportunity_id,
            "score": o.score,
            "priority": o.priority,
            "researchability": o.researchability,
            "confidence": o.confidence,
            "title": o.title
        }),
        None => serde_json::Value::Null,
    };

    let outcome_json = match summary.outcome {
        Some(o) => serde_json::json!({
            "outcome_id": o.outcome_id,
            "type": o.r#type,
            "summary": o.summary,
            "details": o.details,
            "created_at": o.created_at,
            "updated_at": o.updated_at
        }),
        None => serde_json::Value::Null,
    };

    let assessment_json = match summary.evidence_assessment {
        Some(a) => serde_json::json!({
            "score": a.score,
            "status": a.status,
            "evidence_total": a.evidence_total,
            "supporting": a.supporting_count,
            "supporting_count": a.supporting_count,
            "contradicting": a.contradicting_count,
            "contradicting_count": a.contradicting_count,
            "sources": a.sources_count,
            "sources_count": a.sources_count,
            "cited": a.cited_count,
            "cited_count": a.cited_count,
            "uncited": a.uncited_count,
            "uncited_count": a.uncited_count,
            "cited_supporting": a.cited_supporting_count,
            "cited_supporting_count": a.cited_supporting_count,
            "reasons": a.reasons.iter().map(|r| serde_json::json!({"code": r.code, "points": r.points, "message": r.message})).collect::<Vec<_>>()
        }),
        None => serde_json::Value::Null,
    };

    let gaps_json = summary
        .evidence_gaps
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

    let followups_json = summary
        .research_followups
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

    let actions_json = summary
        .followup_actions
        .iter()
        .map(|a| {
            serde_json::json!({
                "id": a.id,
                "followup_code": a.followup_code,
                "status": a.status,
                "notes": a.notes,
                "created_at": a.created_at,
                "updated_at": a.updated_at,
                "completed_at": a.completed_at
            })
        })
        .collect::<Vec<_>>();

    let timeline_json = summary
        .timeline
        .iter()
        .map(|t| {
            serde_json::json!({
                "event_type": t.event_type,
                "timestamp": t.timestamp,
                "label": t.label
            })
        })
        .collect::<Vec<_>>();

    let warnings_json = summary
        .closure_warnings
        .iter()
        .map(|w| {
            serde_json::json!({
                "code": w.code,
                "severity": w.severity,
                "title": w.title,
                "description": w.description
            })
        })
        .collect::<Vec<_>>();

    Ok(Json(serde_json::json!({
        "task": task_json,
        "person": person_json,
        "opportunity": opportunity_json,
        "outcome": outcome_json,
        "evidence_assessment": assessment_json,
        "evidence_gaps": gaps_json,
        "research_followups": followups_json,
        "followup_actions": actions_json,
        "timeline": timeline_json,
        "closure_warnings": warnings_json
    })))
}
