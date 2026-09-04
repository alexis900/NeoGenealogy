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
            "/api/v1/trees/{tree_id}/research-outcomes/{outcome_id}": { "get": { "summary": "Get research outcome", "description": "Returns outcome with evidence[], evidence_assessment {score,status,...reasons}, evidence_gaps [{code,severity,title,description}], research_followups [{code,priority,title,description,gap_code}] and followup_actions [{id,task_id,outcome_id,followup_code,status,notes,created_at,updated_at,completed_at}]" }, "patch": { "summary": "Update research outcome" }, "delete": { "summary": "Delete research outcome" } },
            "/api/v1/trees/{tree_id}/research/summary": { "get": { "summary": "Research summary (opportunities/tasks/outcomes/sources/evidence/assessment/evidence_gaps/followup_actions counts)" } },
            "/api/v1/trees/{tree_id}/sources": { "get": { "summary": "List research sources", "parameters": [{"name":"type"}] }, "post": { "summary": "Create research source" } },
            "/api/v1/trees/{tree_id}/sources/{source_id}": { "get": { "summary": "Get research source" }, "patch": { "summary": "Update research source" }, "delete": { "summary": "Delete research source" } },
            "/api/v1/trees/{tree_id}/sources/{source_id}/citations": { "get": { "summary": "List citations for source" }, "post": { "summary": "Create citation" } },
            "/api/v1/trees/{tree_id}/citations/{citation_id}": { "get": { "summary": "Get citation" }, "patch": { "summary": "Update citation" }, "delete": { "summary": "Delete citation" } },
            "/api/v1/trees/{tree_id}/evidence": { "get": { "summary": "List evidence" }, "post": { "summary": "Create evidence" } },
            "/api/v1/trees/{tree_id}/evidence/{evidence_id}": { "get": { "summary": "Get evidence" }, "patch": { "summary": "Update evidence" }, "delete": { "summary": "Delete evidence" } },
            "/api/v1/trees/{tree_id}/research-outcomes/{outcome_id}/evidence": { "get": { "summary": "List outcome evidence" } },
            "/api/v1/trees/{tree_id}/research-outcomes/{outcome_id}/evidence/{evidence_id}": { "post": { "summary": "Attach evidence to outcome" }, "delete": { "summary": "Detach evidence from outcome" } },
            "/api/v1/trees/{tree_id}/research-outcomes/{outcome_id}/followup-actions": { "get": { "summary": "List followup actions for outcome" }, "post": { "summary": "Create followup action", "description": "Validates FOLLOWUP_NOT_ACTIVE if followup not currently active" } },
            "/api/v1/trees/{tree_id}/research-followup-actions": { "get": { "summary": "List followup actions", "parameters": [{"name":"task_id"},{"name":"outcome_id"},{"name":"status"},{"name":"followup_code"}] } },
            "/api/v1/trees/{tree_id}/research-followup-actions/{action_id}": { "get": { "summary": "Get followup action" }, "patch": { "summary": "Update followup action" }, "delete": { "summary": "Delete followup action" } },
            "/api/v1/trees/{tree_id}/research-tasks/{task_id}/followup-actions": { "get": { "summary": "List followup actions for task" } },
            "/api/v1/trees/{tree_id}/research-tasks/{task_id}/case-summary": { "get": { "summary": "Get research case summary", "description": "Derived view: task + person + opportunity + outcome + evidence_assessment + evidence_gaps + research_followups + followup_actions + timeline + closure_warnings. 404 only if TASK_NOT_FOUND. No persistent CaseStatus." } },
            "/api/v1/trees/{tree_id}/research/plan": { "get": { "summary": "Get research plan", "description": "Deterministic planning: research_score*0.55 + researchability*0.20 + confidence*0.10 + evidence_gap*0.10 + task_state*0.05. No persistence. Sorted planning_score DESC, research_score DESC, confidence DESC, opportunity_id ASC. Returns recommended (top 10 by default, limit param) and deferred. Query params: limit 1..100 default 10, min_score 0..100, priority, researchability.", "parameters": [{"name":"limit","description":"Recommended size, default 10 max 100"},{"name":"min_score","description":"Minimum planning_score 0..100"},{"name":"priority","description":"Filter by priority: low,info,medium,warning,high,critical"},{"name":"researchability","description":"Filter by researchability: low,medium,high"}] } },
            "/api/v1/trees/{tree_id}/research-sessions": { "get": { "summary": "List research sessions", "parameters": [{"name":"status"},{"name":"person_id"},{"name":"opportunity_id"},{"name":"limit"},{"name":"offset"},{"name":"history","description":"If true, delegates to history (COMPLETED/ABANDONED)"}] }, "post": { "summary": "Create research session" } },
            "/api/v1/trees/{tree_id}/research-sessions/history": { "get": { "summary": "List research session history (COMPLETED/ABANDONED, ordered completed_at DESC fallback updated_at DESC)", "parameters": [{"name":"status","description":"COMPLETED or ABANDONED"},{"name":"person_id"},{"name":"limit"},{"name":"offset"},{"name":"page"}] } },
            "/api/v1/trees/{tree_id}/research-sessions/{session_id}": { "get": { "summary": "Get research session (with person/opportunity/tasks/summary/stats/timeline)", "description": "Includes derived stats {total_tasks,completed_tasks,open_tasks,in_progress_tasks,inconclusive_tasks,rejected_tasks,total_outcomes,confirmed_outcomes,false_lead_outcomes,inconclusive_outcomes,new_lead_outcomes,no_evidence_outcomes,total_evidence,supporting_evidence,contradicting_evidence,open_followups,completed_followup_actions,skipped_followup_actions} and timeline [{event_type,timestamp,label}] (20 latest, DESC). No persistence." }, "patch": { "summary": "Update research session (title/description/status/person_id/opportunity_id)" }, "delete": { "summary": "Delete research session" } },
            "/api/v1/trees/{tree_id}/research-sessions/{session_id}/tasks": { "get": { "summary": "List tasks for session" } },
            "/api/v1/trees/{tree_id}/research-tasks/{task_id}/session": { "post": { "summary": "Assign task to session" }, "delete": { "summary": "Remove task from session" } },
            "/api/v1/research-sessions": { "get": { "summary": "List research sessions (generic)" }, "post": { "summary": "Create research session (generic, tree_id in body)" } },
            "/api/v1/research-sessions/{session_id}": { "get": { "summary": "Get research session generic" }, "patch": { "summary": "Update research session generic" }, "delete": { "summary": "Delete research session generic" } },
            "/api/v1/research-sessions/history": { "get": { "summary": "List research session history (generic, requires tree_id)", "parameters": [{"name":"tree_id"},{"name":"status"},{"name":"person_id"},{"name":"limit"},{"name":"offset"},{"name":"page"}] } },
            "/api/v1/trees/{tree_id}/research-tasks/{task_id}/research-queries": { "get": { "summary": "List queries for task", "parameters": [{"name":"limit"},{"name":"offset"}] }, "post": { "summary": "Create research query (PENDING, provider=mock|familysearch)", "description": "Payload {provider, query}. Provider must be mock or familysearch. FamilySearch requires NEOGENEALOGY_FAMILYSEARCH_CLIENT_ID or ACCESS_TOKEN; otherwise run returns AUTH_REQUIRED. See docs/FAMILYSEARCH.md" } },
            "/api/v1/trees/{tree_id}/research-queries": { "get": { "summary": "List research queries", "parameters": [{"name":"task_id"},{"name":"provider"},{"name":"status"},{"name":"limit"},{"name":"offset"}] } },
            "/api/v1/trees/{tree_id}/research-queries/{query_id}": { "get": { "summary": "Get research query with latest_execution" }, "delete": { "summary": "Delete research query (cascade executions/results)" } },
            "/api/v1/trees/{tree_id}/research-queries/{query_id}/run": { "post": { "summary": "Run research query (PENDING→RUNNING→COMPLETED/FAILED)", "description": "Synchronous execution via mock provider; creates execution and results. Rerun allowed (COMPLETED/FAILED→RUNNING creates new execution, preserves history)." } },
            "/api/v1/trees/{tree_id}/research-queries/{query_id}/executions": { "get": { "summary": "List executions for query", "parameters": [{"name":"limit"},{"name":"offset"}] } },
            "/api/v1/trees/{tree_id}/research-queries/{query_id}/results": { "get": { "summary": "List results for latest execution", "parameters": [{"name":"limit"},{"name":"offset"}] } },
            "/api/v1/trees/{tree_id}/research-results/{result_id}": { "get": { "summary": "Get research result (candidate, not evidence)", "description": "External Research Result is a candidate, not evidence." } },
            "/api/v1/research-tasks/{task_id}/research-queries": { "post": { "summary": "Create research query (generic)" } },
            "/api/v1/research-queries": { "get": { "summary": "List research queries (generic, requires tree_id)" } },
            "/api/v1/research-queries/{query_id}": { "get": { "summary": "Get research query (generic)" }, "delete": { "summary": "Delete research query" } },
            "/api/v1/research-queries/{query_id}/run": { "post": { "summary": "Run research query (generic)" } },
            "/api/v1/research-queries/{query_id}/executions": { "get": { "summary": "List executions" } },
            "/api/v1/research-queries/{query_id}/results": { "get": { "summary": "List results (latest execution)" } },
            "/api/v1/research-results/{result_id}": { "get": { "summary": "Get research result (generic)" } },
            "/api/v1/research-providers": { "get": { "summary": "List research providers", "description": "Returns [{name, display_name, configured, enabled, status, requires_auth}]. Mock always configured. FamilySearch configured when NEOGENEALOGY_FAMILYSEARCH_CLIENT_ID or ACCESS_TOKEN is set. See docs/FAMILYSEARCH.md" } },
            "/api/v1/trees/{tree_id}/research-providers": { "get": { "summary": "List research providers for tree", "description": "Tree-scoped alias for /research-providers" } },
            "/api/v1/auth/familysearch/authorize": { "get": { "summary": "Get FamilySearch authorization URL", "description": "Returns {authorization_url, state}. Redirect user to authorization_url (FamilySearch login, may show Google). Requires NEOGENEALOGY_FAMILYSEARCH_CLIENT_ID and pre-registered redirect_uri. See docs/FAMILYSEARCH.md" } },
            "/api/v1/auth/familysearch/callback": { "get": { "summary": "FamilySearch OAuth callback", "description": "Handles redirect from FamilySearch with code & state, exchanges for token, stores it, redirects to frontend. Query params: code, state, error. See docs/FAMILYSEARCH.md" } },
            "/api/v1/auth/familysearch/status": { "get": { "summary": "FamilySearch connection status", "description": "Returns {configured, enabled, connected, status, expires_at, requires_auth, redirect_uri}. Connected true when stored token valid or env token present." } },
            "/api/v1/auth/familysearch/disconnect": { "post": { "summary": "Disconnect FamilySearch", "description": "Deletes stored token" } },
            "/api/v1/familysearch/search": { "get": { "summary": "FamilySearch global search without tree", "description": "Query params: q or query or givenName/surname/birthLikeDate/birthLikePlace. Uses stored OAuth token or env token or unauthenticated_session. Returns {provider, query, results[], result_count}. No ResearchQuery persistence. See docs/FAMILYSEARCH.md" } },
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
                        "research_followups": { "type": "array", "items": { "$ref": "#/components/schemas/ResearchFollowUp" } },
                        "followup_actions": { "type": "array", "items": { "$ref": "#/components/schemas/ResearchFollowupAction" } },
                        "followup_actions_count": { "type": "integer" }
                    }
                },
                "ResearchFollowupAction": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "integer" },
                        "tree_id": { "type": "integer" },
                        "task_id": { "type": "integer" },
                        "outcome_id": { "type": "integer" },
                        "followup_code": { "type": "string", "enum": ["ADD_SUPPORTING_EVIDENCE","ADD_CITATION","REVIEW_CONTRADICTION","ADD_SECOND_SUPPORTING_EVIDENCE","REVIEW_SOURCE_COVERAGE"] },
                        "status": { "type": "string", "enum": ["OPEN","COMPLETED","SKIPPED"] },
                        "notes": { "type": "string", "nullable": true },
                        "created_at": { "type": "string" },
                        "updated_at": { "type": "string" },
                        "completed_at": { "type": "string", "nullable": true }
                    }
                },
                "ResearchFollowupActionStatus": { "type": "string", "enum": ["OPEN","COMPLETED","SKIPPED"] },
                "ResearchCaseTimelineEvent": {
                    "type": "object",
                    "properties": {
                        "event_type": { "type": "string", "enum": ["TASK_CREATED","TASK_STARTED","OUTCOME_CREATED","OUTCOME_UPDATED","FOLLOWUP_ACTION_CREATED","FOLLOWUP_ACTION_COMPLETED","TASK_COMPLETED"] },
                        "timestamp": { "type": "string", "format": "date-time" },
                        "label": { "type": "string" }
                    }
                },
                "ResearchCaseClosureWarning": {
                    "type": "object",
                    "properties": {
                        "code": { "type": "string", "enum": ["RESOLVED_WITHOUT_OUTCOME","CONFIRMED_WITHOUT_SUPPORT","RESOLVED_WITH_EVIDENCE_GAPS","REJECTED_WITH_CONFIRMED_OUTCOME","INCONCLUSIVE_WITH_CONFIRMED_OUTCOME"] },
                        "severity": { "type": "string", "enum": ["INFO","WARNING","CRITICAL"] },
                        "title": { "type": "string" },
                        "description": { "type": "string" }
                    }
                },
                "ClosureWarningCode": { "type": "string", "enum": ["RESOLVED_WITHOUT_OUTCOME","CONFIRMED_WITHOUT_SUPPORT","RESOLVED_WITH_EVIDENCE_GAPS","REJECTED_WITH_CONFIRMED_OUTCOME","INCONCLUSIVE_WITH_CONFIRMED_OUTCOME"] },
                "ClosureWarningSeverity": { "type": "string", "enum": ["INFO","WARNING","CRITICAL"] },
                "ResearchCaseSummary": {
                    "type": "object",
                    "properties": {
                        "task": { "type": "object", "description": "Task fields: id,title,description,status,resolution,created_at,started_at,completed_at,updated_at" },
                        "person": { "type": "object", "nullable": true, "properties": { "person_id": { "type": "integer" }, "person_name": { "type": "string" } } },
                        "opportunity": { "type": "object", "nullable": true, "properties": { "opportunity_id": { "type": "integer" }, "score": { "type": "integer", "nullable": true }, "priority": { "type": "string", "nullable": true }, "researchability": { "type": "string", "nullable": true }, "confidence": { "type": "number", "nullable": true }, "title": { "type": "string", "nullable": true } } },
                        "outcome": { "type": "object", "nullable": true, "properties": { "outcome_id": { "type": "integer" }, "type": { "type": "string" }, "summary": { "type": "string" }, "details": { "type": "string", "nullable": true }, "created_at": { "type": "string" }, "updated_at": { "type": "string" } } },
                        "evidence_assessment": { "$ref": "#/components/schemas/EvidenceAssessment", "nullable": true },
                        "evidence_gaps": { "type": "array", "items": { "$ref": "#/components/schemas/EvidenceGap" } },
                        "research_followups": { "type": "array", "items": { "$ref": "#/components/schemas/ResearchFollowUp" } },
                        "followup_actions": { "type": "array", "items": { "$ref": "#/components/schemas/ResearchFollowupAction" } },
                        "timeline": { "type": "array", "items": { "$ref": "#/components/schemas/ResearchCaseTimelineEvent" } },
                        "closure_warnings": { "type": "array", "items": { "$ref": "#/components/schemas/ResearchCaseClosureWarning" } }
                    }
                },
                "ResearchPlanningReasonCode": { "type": "string", "enum": ["HIGH_RESEARCH_SCORE","HIGH_RESEARCHABILITY","HIGH_CONFIDENCE","CRITICAL_EVIDENCE_GAP","WARNING_EVIDENCE_GAP","INFO_EVIDENCE_GAP","NO_ACTIVE_TASK","ACTIVE_TASK","PREVIOUSLY_INCONCLUSIVE"] },
                "ResearchPlanningReason": {
                    "type": "object",
                    "properties": {
                        "code": { "$ref": "#/components/schemas/ResearchPlanningReasonCode" },
                        "label": { "type": "string" },
                        "description": { "type": "string" }
                    }
                },
                "ResearchPlanItem": {
                    "type": "object",
                    "properties": {
                        "opportunity_id": { "type": "integer" },
                        "person_id": { "type": "integer" },
                        "title": { "type": "string" },
                        "priority": { "type": "string", "enum": ["LOW","MEDIUM","HIGH","CRITICAL","INFO","WARNING"] },
                        "research_score": { "type": "integer" },
                        "planning_score": { "type": "number", "minimum": 0, "maximum": 100 },
                        "researchability": { "type": "string", "enum": ["LOW","MEDIUM","HIGH"] },
                        "confidence": { "type": "number", "minimum": 0, "maximum": 1 },
                        "active_task": { "type": "boolean" },
                        "task_status": { "type": "string", "nullable": true, "enum": ["OPEN","IN_PROGRESS","INCONCLUSIVE"] },
                        "reasons": { "type": "array", "items": { "$ref": "#/components/schemas/ResearchPlanningReason" } }
                    }
                },
                "ResearchPlanSummary": {
                    "type": "object",
                    "properties": {
                        "total_candidates": { "type": "integer" },
                        "recommended_count": { "type": "integer" },
                        "deferred_count": { "type": "integer" },
                        "active_count": { "type": "integer" },
                        "inconclusive_count": { "type": "integer" },
                        "high_priority_count": { "type": "integer" },
                        "critical_gap_count": { "type": "integer" }
                    }
                },
                "ResearchPlan": {
                    "type": "object",
                    "properties": {
                        "generated_at": { "type": "string", "format": "date-time" },
                        "total_candidates": { "type": "integer" },
                        "summary": { "$ref": "#/components/schemas/ResearchPlanSummary" },
                        "recommended": { "type": "array", "items": { "$ref": "#/components/schemas/ResearchPlanItem" } },
                        "deferred": { "type": "array", "items": { "$ref": "#/components/schemas/ResearchPlanItem" } }
                    }
                },
                "ResearchSessionStatus": { "type": "string", "enum": ["PLANNED","ACTIVE","COMPLETED","ABANDONED"] },
                "ResearchSession": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "integer" },
                        "tree_id": { "type": "integer" },
                        "title": { "type": "string" },
                        "description": { "type": "string", "nullable": true },
                        "status": { "$ref": "#/components/schemas/ResearchSessionStatus" },
                        "person_id": { "type": "integer", "nullable": true },
                        "opportunity_id": { "type": "integer", "nullable": true },
                        "created_at": { "type": "string", "format": "date-time" },
                        "updated_at": { "type": "string", "format": "date-time" },
                        "started_at": { "type": "string", "nullable": true, "format": "date-time" },
                        "completed_at": { "type": "string", "nullable": true, "format": "date-time" },
                        "person": { "type": "object", "nullable": true },
                        "opportunity": { "type": "object", "nullable": true },
                        "tasks": { "type": "array" },
                        "summary": { "type": "object", "properties": { "total_tasks": { "type": "integer" }, "open_tasks": { "type": "integer" }, "in_progress_tasks": { "type": "integer" }, "terminal_tasks": { "type": "integer" }, "outcomes_count": { "type": "integer" } } },
                        "stats": { "$ref": "#/components/schemas/ResearchSessionStats" },
                        "timeline": { "type": "array", "items": { "$ref": "#/components/schemas/ResearchSessionTimelineEvent" } }
                    }
                },
                "ResearchSessionSummary": {
                    "type": "object",
                    "properties": {
                        "total_tasks": { "type": "integer" },
                        "open_tasks": { "type": "integer" },
                        "in_progress_tasks": { "type": "integer" },
                        "terminal_tasks": { "type": "integer" },
                        "outcomes_count": { "type": "integer" }
                    }
                },
                "ResearchSessionStats": {
                    "type": "object",
                    "description": "100% derived from research_sessions/research_tasks/research_outcomes/evidence/followups. No persistence.",
                    "properties": {
                        "total_tasks": { "type": "integer" },
                        "completed_tasks": { "type": "integer", "description": "RESOLVED" },
                        "open_tasks": { "type": "integer" },
                        "in_progress_tasks": { "type": "integer" },
                        "inconclusive_tasks": { "type": "integer" },
                        "rejected_tasks": { "type": "integer" },
                        "total_outcomes": { "type": "integer" },
                        "confirmed_outcomes": { "type": "integer" },
                        "false_lead_outcomes": { "type": "integer" },
                        "inconclusive_outcomes": { "type": "integer" },
                        "new_lead_outcomes": { "type": "integer" },
                        "no_evidence_outcomes": { "type": "integer" },
                        "total_evidence": { "type": "integer" },
                        "supporting_evidence": { "type": "integer" },
                        "contradicting_evidence": { "type": "integer" },
                        "open_followups": { "type": "integer", "description": "OPEN followup actions" },
                        "completed_followup_actions": { "type": "integer" },
                        "skipped_followup_actions": { "type": "integer" }
                    }
                },
                "ResearchSessionTimelineEvent": {
                    "type": "object",
                    "description": "Derived from timestamps, 20 latest DESC, no event table.",
                    "properties": {
                        "event_type": { "type": "string", "enum": ["SESSION_CREATED","SESSION_STARTED","SESSION_COMPLETED","TASK_CREATED","TASK_STARTED","TASK_COMPLETED","OUTCOME_CREATED","OUTCOME_UPDATED","EVIDENCE_ADDED","FOLLOWUP_ACTION_CREATED","FOLLOWUP_ACTION_COMPLETED"] },
                        "timestamp": { "type": "string", "format": "date-time" },
                        "label": { "type": "string" }
                    }
                },
                "ResearchActivitySummary": {
                    "type": "object",
                    "properties": {
                        "tasks": { "type": "object", "properties": { "open": {"type":"integer"}, "in_progress": {"type":"integer"}, "resolved": {"type":"integer"}, "rejected": {"type":"integer"}, "inconclusive": {"type":"integer"}, "total": {"type":"integer"} } },
                        "outcomes": { "type": "object", "properties": { "total": {"type":"integer"}, "confirmed": {"type":"integer"}, "false_lead": {"type":"integer"}, "inconclusive": {"type":"integer"}, "new_lead": {"type":"integer"}, "no_evidence": {"type":"integer"} } },
                        "evidence": { "type": "object", "properties": { "total": {"type":"integer"}, "supporting": {"type":"integer"}, "contradicting": {"type":"integer"} } },
                        "followups": { "type": "object", "properties": { "open": {"type":"integer"}, "completed": {"type":"integer"}, "skipped": {"type":"integer"}, "total": {"type":"integer"} } }
                    }
                },
                "ResearchSummarySessions": {
                    "type": "object",
                    "properties": {
                        "total": {"type":"integer"},
                        "active": {"type":"integer"},
                        "planned": {"type":"integer"},
                        "completed": {"type":"integer"},
                        "abandoned": {"type":"integer"}
                    }
                },
                "ResearchQueryStatus": { "type": "string", "enum": ["PENDING","RUNNING","COMPLETED","FAILED"] },
                "ResearchProviderError": { "type": "string", "enum": ["NO_RESULTS","PROVIDER_UNAVAILABLE","AUTH_REQUIRED","RATE_LIMITED","INVALID_QUERY","TIMEOUT","UNKNOWN"] },
                "ResearchQuery": {
                    "type": "object",
                    "properties": {
                        "id": {"type":"integer"},
                        "tree_id": {"type":"integer"},
                        "task_id": {"type":"integer"},
                        "provider": {"type":"string", "example": "mock"},
                        "query": {"type":"string"},
                        "status": {"$ref":"#/components/schemas/ResearchQueryStatus"},
                        "created_at": {"type":"string"},
                        "started_at": {"type":"string","nullable":true},
                        "completed_at": {"type":"string","nullable":true},
                        "error_code": {"type":"string","nullable":true},
                        "error_message": {"type":"string","nullable":true},
                        "latest_execution": {"$ref":"#/components/schemas/ResearchQueryExecution","nullable":true}
                    }
                },
                "ResearchQueryExecution": {
                    "type": "object",
                    "properties": {
                        "id": {"type":"integer"},
                        "query_id": {"type":"integer"},
                        "status": {"$ref":"#/components/schemas/ResearchQueryStatus"},
                        "started_at": {"type":"string","nullable":true},
                        "completed_at": {"type":"string","nullable":true},
                        "error_code": {"type":"string","nullable":true},
                        "error_message": {"type":"string","nullable":true},
                        "provider_request_id": {"type":"string","nullable":true},
                        "provider_metadata": {"type":"object","nullable":true},
                        "created_at": {"type":"string"},
                        "result_count": {"type":"integer"}
                    }
                },
                "ResearchResult": {
                    "type": "object",
                    "properties": {
                        "id": {"type":"integer"},
                        "execution_id": {"type":"integer"},
                        "query_id": {"type":"integer"},
                        "provider": {"type":"string"},
                        "external_id": {"type":"string","nullable":true},
                        "title": {"type":"string"},
                        "description": {"type":"string","nullable":true},
                        "url": {"type":"string","nullable":true},
                        "record_type": {"type":"string","nullable":true},
                        "date": {"type":"string","nullable":true},
                        "place": {"type":"string","nullable":true},
                        "metadata": {"type":"object"},
                        "position": {"type":"integer"},
                        "created_at": {"type":"string"}
                    }
                },
                "ResearchProvider": { "type": "string", "enum": ["mock", "familysearch"] },
                "ResearchProviderInfo": {
                    "type": "object",
                    "properties": {
                        "name": {"type":"string", "enum":["mock","familysearch"]},
                        "display_name": {"type":"string"},
                        "configured": {"type":"boolean"},
                        "enabled": {"type":"boolean"},
                        "status": {"type":"string", "enum":["configured","not_configured","disabled","connected"]},
                        "requires_auth": {"type":"boolean"},
                        "connected": {"type":"boolean"}
                    }
                },
                "ExternalResearchSummary": {
                    "type": "object",
                    "properties": {
                        "queries": {"type":"integer"},
                        "executions": {"type":"integer"},
                        "successful": {"type":"integer"},
                        "failed": {"type":"integer"},
                        "pending": {"type":"integer"},
                        "results": {"type":"integer"}
                    }
                }
            }
        }
    }))
}

pub async fn get_docs() -> axum::response::Html<String> {
    axum::response::Html(r#"<!doctype html><html><head><title>NeoGenealogy API Docs</title></head><body><h1>NeoGenealogy API v1</h1><p>See <a href="/api/v1/openapi.json">openapi.json</a></p></body></html>"#.to_string())
}
