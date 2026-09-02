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
            "/api/v1/trees/{tree_id}/research-tasks": { "get": { "summary": "List research tasks", "parameters": [{"name":"status"},{"name":"person_id"},{"name":"opportunity_id"}] }, "post": { "summary": "Create research task" } },
            "/api/v1/trees/{tree_id}/research-tasks/{task_id}": { "get": { "summary": "Get research task (outcome embedded)" }, "patch": { "summary": "Update research task" }, "delete": { "summary": "Delete research task" } },
            "/api/v1/trees/{tree_id}/research-opportunities/{opportunity_id}/tasks": { "post": { "summary": "Create task from opportunity" } },
            "/api/v1/trees/{tree_id}/research-tasks/{task_id}/outcome": { "post": { "summary": "Create research outcome for task" } },
            "/api/v1/trees/{tree_id}/research-outcomes": { "get": { "summary": "List research outcomes", "parameters": [{"name":"type"},{"name":"task_id"},{"name":"person_id"}] } },
            "/api/v1/trees/{tree_id}/research-outcomes/{outcome_id}": { "get": { "summary": "Get research outcome" }, "patch": { "summary": "Update research outcome" }, "delete": { "summary": "Delete research outcome" } },
        }
    }))
}

pub async fn get_docs() -> axum::response::Html<String> {
    axum::response::Html(r#"<!doctype html><html><head><title>NeoGenealogy API Docs</title></head><body><h1>NeoGenealogy API v1</h1><p>See <a href="/api/v1/openapi.json">openapi.json</a></p></body></html>"#.to_string())
}
