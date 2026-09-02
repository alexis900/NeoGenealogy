use axum::Json;
use serde_json::{json, Value};

pub async fn get_openapi() -> Json<Value> {
    Json(json!({
        "openapi": "3.0.0",
        "info": { "title": "NeoGenealogy API", "version": "1.0.0" },
        "paths": {
            "/health": { "get": { "summary": "Health check" } },
            "/api/v1/trees": { "get": { "summary": "List trees" } },
            "/api/v1/trees/{tree_id}": { "get": { "summary": "Get tree" } },
            "/api/v1/trees/{tree_id}/persons": { "get": { "summary": "List persons", "parameters": [{"name":"limit"},{"name":"offset"}] } },
            "/api/v1/trees/{tree_id}/persons/{person_id}": { "get": { "summary": "Get person" } },
            "/api/v1/trees/{tree_id}/families": { "get": { "summary": "List families" } },
            "/api/v1/trees/{tree_id}/families/{family_id}": { "get": { "summary": "Get family" } },
            "/api/v1/trees/{tree_id}/findings": { "get": { "summary": "List findings", "parameters": [{"name":"severity"},{"name":"type"},{"name":"person_id"}] } },
            "/api/v1/trees/{tree_id}/research-opportunities": { "get": { "summary": "List research opportunities", "parameters": [{"name":"priority"},{"name":"min_score"},{"name":"sort"}] } },
            "/api/v1/trees/{tree_id}/research-opportunities/top": { "get": { "summary": "Top opportunities" } },
            "/api/v1/trees/{tree_id}/branches": { "get": { "summary": "List branches" } },
            "/api/v1/trees/{tree_id}/source-coverage": { "get": { "summary": "Source coverage" } },
            "/api/v1/trees/{tree_id}/analysis-runs": { "get": { "summary": "List analysis runs" } },
            "/api/v1/trees/{tree_id}/research-tasks": { "get": { "summary": "List research tasks", "parameters": [{"name":"status"},{"name":"person_id"},{"name":"opportunity_id"},{"name":"has_outcome"}] }, "post": { "summary": "Create research task" } },
            "/api/v1/trees/{tree_id}/research-tasks/{task_id}": { "get": { "summary": "Get research task (outcome embedded)" }, "patch": { "summary": "Update research task" }, "delete": { "summary": "Delete research task" } },
            "/api/v1/trees/{tree_id}/research-opportunities/{opportunity_id}/tasks": { "post": { "summary": "Create task from opportunity" } },
            "/api/v1/trees/{tree_id}/research-tasks/{task_id}/outcome": { "post": { "summary": "Create research outcome for task" } },
            "/api/v1/trees/{tree_id}/research-outcomes": { "get": { "summary": "List research outcomes", "parameters": [{"name":"type"},{"name":"task_id"},{"name":"person_id"},{"name":"assessment_status","description":"Filter by Evidence Assessment status: NO_EVIDENCE,WEAK,MIXED,SUPPORTED,STRONGLY_SUPPORTED"},{"name":"gap","description":"Filter by Evidence Gap code: NO_SUPPORTING_EVIDENCE,NO_CITATION,SINGLE_SUPPORTING_EVIDENCE,CONTRADICTORY_EVIDENCE,SINGLE_SOURCE,CONFIRMED_WITHOUT_SUPPORT"}] } },
            "/api/v1/trees/{tree_id}/research-outcomes/{outcome_id}": { "get": { "summary": "Get research outcome", "description": "Returns outcome with evidence[], evidence_assessment {score,status,...reasons}, evidence_gaps [{code,severity,title,description}] and research_followups [{code,priority,title,description,gap_code}]" }, "patch": { "summary": "Update research outcome" }, "delete": { "summary": "Delete research outcome" } },
            "/api/v1/trees/{tree_id}/research/summary": { "get": { "summary": "Research summary (opportunities/tasks/outcomes/sources/evidence/assessment/evidence_gaps counts)" } },
            "/api/v1/trees/{tree_id}/sources": { "get": { "summary": "List research sources", "parameters": [{"name":"type"}] }, "post": { "summary": "Create research source" } },
            "/api/v1/trees/{tree_id}/sources/{source_id}": { "get": { "summary": "Get research source" }, "patch": { "summary": "Update research source" }, "delete": { "summary": "Delete research source" } },
            "/api/v1/trees/{tree_id}/sources/{source_id}/citations": { "get": { "summary": "List citations for source" }, "post": { "summary": "Create citation" } },
            "/api/v1/trees/{tree_id}/citations/{citation_id}": { "get": { "summary": "Get citation" }, "patch": { "summary": "Update citation" }, "delete": { "summary": "Delete citation" } },
            "/api/v1/trees/{tree_id}/evidence": { "get": { "summary": "List evidence" }, "post": { "summary": "Create evidence" } },
            "/api/v1/trees/{tree_id}/evidence/{evidence_id}": { "get": { "summary": "Get evidence" }, "patch": { "summary": "Update evidence" }, "delete": { "summary": "Delete evidence" } },
            "/api/v1/trees/{tree_id}/research-outcomes/{outcome_id}/evidence": { "get": { "summary": "List outcome evidence" } },
            "/api/v1/trees/{tree_id}/research-outcomes/{outcome_id}/evidence/{evidence_id}": { "post": { "summary": "Attach evidence to outcome" }, "delete": { "summary": "Detach evidence from outcome" } },
        },
        "components": {
            "schemas": {
                "EvidenceStats": {
                    "type": "object",
                    "properties": {
                        "evidence_total": { "type": "integer" },
                        "supporting_count": { "type": "integer" },
                        "contradicting_count": { "type": "integer" },
                        "sources_count": { "type": "integer" },
                        "cited_count": { "type": "integer" },
                        "uncited_count": { "type": "integer" },
                        "cited_supporting_count": { "type": "integer" }
                    }
                },
                "EvidenceAssessmentReason": {
                    "type": "object",
                    "properties": {
                        "code": { "type": "string", "example": "SUPPORTING_EVIDENCE" },
                        "points": { "type": "integer", "example": 30 },
                        "message": { "type": "string", "example": "Supporting evidence exists" }
                    }
                },
                "EvidenceAssessment": {
                    "type": "object",
                    "properties": {
                        "score": { "type": "integer", "minimum": 0, "maximum": 100 },
                        "status": { "type": "string", "enum": ["NO_EVIDENCE","WEAK","MIXED","SUPPORTED","STRONGLY_SUPPORTED"] },
                        "evidence_total": { "type": "integer" },
                        "supporting_count": { "type": "integer" },
                        "contradicting_count": { "type": "integer" },
                        "sources_count": { "type": "integer" },
                        "cited_count": { "type": "integer" },
                        "uncited_count": { "type": "integer" },
                        "reasons": { "type": "array", "items": { "$ref": "#/components/schemas/EvidenceAssessmentReason" } }
                    }
                },
                "EvidenceGap": {
                    "type": "object",
                    "properties": {
                        "code": { "type": "string", "enum": ["NO_SUPPORTING_EVIDENCE","NO_CITATION","SINGLE_SUPPORTING_EVIDENCE","CONTRADICTORY_EVIDENCE","SINGLE_SOURCE","CONFIRMED_WITHOUT_SUPPORT"] },
                        "severity": { "type": "string", "enum": ["INFO","WARNING","CRITICAL"] },
                        "title": { "type": "string" },
                        "description": { "type": "string" }
                    }
                },
                "ResearchFollowUp": {
                    "type": "object",
                    "properties": {
                        "code": { "type": "string", "enum": ["ADD_SUPPORTING_EVIDENCE","ADD_CITATION","REVIEW_CONTRADICTION","ADD_SECOND_SUPPORTING_EVIDENCE","REVIEW_SOURCE_COVERAGE"] },
                        "priority": { "type": "string", "enum": ["HIGH","MEDIUM","LOW"] },
                        "title": { "type": "string" },
                        "description": { "type": "string" },
                        "gap_code": { "type": "string" }
                    }
                },
                "ResearchFollowUpCode": { "type": "string", "enum": ["ADD_SUPPORTING_EVIDENCE","ADD_CITATION","REVIEW_CONTRADICTION","ADD_SECOND_SUPPORTING_EVIDENCE","REVIEW_SOURCE_COVERAGE"] },
                "ResearchFollowUpPriority": { "type": "string", "enum": ["HIGH","MEDIUM","LOW"] },
                "ResearchOutcome": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "integer" },
                        "tree_id": { "type": "integer" },
                        "task_id": { "type": "integer" },
                        "type": { "type": "string", "enum": ["CONFIRMED","FALSE_LEAD","INCONCLUSIVE","NEW_LEAD","NO_EVIDENCE"] },
                        "summary": { "type": "string" },
                        "details": { "type": "string", "nullable": true },
                        "created_at": { "type": "string" },
                        "updated_at": { "type": "string" },
                        "evidence": { "type": "array" },
                        "evidence_assessment": { "$ref": "#/components/schemas/EvidenceAssessment" },
                        "evidence_gaps": { "type": "array", "items": { "$ref": "#/components/schemas/EvidenceGap" } },
                        "research_followups": { "type": "array", "items": { "$ref": "#/components/schemas/ResearchFollowUp" } }
                    }
                }
            }
        }
    }))
}

pub async fn get_docs() -> axum::response::Html<String> {
    axum::response::Html(r#"<!doctype html><html><head><title>NeoGenealogy API Docs</title></head><body><h1>NeoGenealogy API v1</h1><p>See <a href="/api/v1/openapi.json">openapi.json</a></p></body></html>"#.to_string())
}
