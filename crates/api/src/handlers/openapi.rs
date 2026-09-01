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
        }
    }))
}

pub async fn get_docs() -> axum::response::Html<String> {
    axum::response::Html(r#"<!doctype html><html><head><title>NeoGenealogy API Docs</title></head><body><h1>NeoGenealogy API v1</h1><p>See <a href="/api/v1/openapi.json">openapi.json</a></p></body></html>"#.to_string())
}
