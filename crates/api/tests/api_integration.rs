use axum_test::TestServer;
use neogenealogy_api::{create_router, state::AppState};
use neogenealogy_storage::{db::in_memory_pool, import_gedcom_content, Storage};

async fn test_server() -> TestServer {
    let pool = in_memory_pool().await.unwrap();
    let content =
        std::fs::read_to_string("/home/amartinper/NeoGenealogy/test-data/complex.ged").unwrap();
    import_gedcom_content(&pool, &content, "complex.ged", None)
        .await
        .unwrap();
    let storage = Storage::new(pool);
    let state = AppState::new(storage);
    let app = create_router(state);
    TestServer::new(app).unwrap()
}

#[tokio::test]
async fn test_health() {
    let server = test_server().await;
    let resp = server.get("/health").await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn test_trees() {
    let server = test_server().await;
    let resp = server.get("/api/v1/trees").await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert!(!body["items"].as_array().unwrap().is_empty());
    assert!(body["pagination"]["total"].as_i64().unwrap() >= 1);
}

#[tokio::test]
async fn test_get_tree() {
    let server = test_server().await;
    let resp = server.get("/api/v1/trees/1").await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["id"], 1);
    assert!(body["persons"].as_i64().unwrap() >= 10);
}

#[tokio::test]
async fn test_persons_pagination() {
    let server = test_server().await;
    let resp = server.get("/api/v1/trees/1/persons?limit=5&offset=0").await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["items"].as_array().unwrap().len(), 5);
    assert_eq!(body["pagination"]["limit"], 5);
    // offset
    let resp2 = server.get("/api/v1/trees/1/persons?limit=5&offset=5").await;
    let body2: serde_json::Value = resp2.json();
    assert_eq!(body2["items"].as_array().unwrap().len(), 5);
    assert_ne!(body["items"][0]["id"], body2["items"][0]["id"]);
}

#[tokio::test]
async fn test_get_person() {
    let server = test_server().await;
    let resp = server.get("/api/v1/trees/1/persons/1").await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["tree_id"], 1);
}

#[tokio::test]
async fn test_families() {
    let server = test_server().await;
    let resp = server.get("/api/v1/trees/1/families").await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert!(!body["items"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_get_family() {
    let server = test_server().await;
    let resp = server.get("/api/v1/trees/1/families/1").await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["tree_id"], 1);
    assert!(body["members"].is_object());
}

#[tokio::test]
async fn test_findings() {
    let server = test_server().await;
    let resp = server.get("/api/v1/trees/1/findings").await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert!(!body["items"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_findings_filter_severity() {
    let server = test_server().await;
    let resp = server.get("/api/v1/trees/1/findings?severity=high").await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    for item in body["items"].as_array().unwrap() {
        assert_eq!(item["severity"], "high");
    }
}

#[tokio::test]
async fn test_research_opportunities() {
    let server = test_server().await;
    let resp = server.get("/api/v1/trees/1/research-opportunities").await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    let items = body["items"].as_array().unwrap();
    assert!(!items.is_empty());
    // check breakdown present
    assert!(items[0]["breakdown"].is_object());
}

#[tokio::test]
async fn test_research_opportunities_sort_and_filter() {
    let server = test_server().await;
    let resp = server
        .get("/api/v1/trees/1/research-opportunities?priority=high&sort=score&limit=5")
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert!(body["items"].as_array().unwrap().len() <= 5);
}

#[tokio::test]
async fn test_top_opportunities() {
    let server = test_server().await;
    let resp = server
        .get("/api/v1/trees/1/research-opportunities/top")
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert!(!body["items"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_branches() {
    let server = test_server().await;
    let resp = server.get("/api/v1/trees/1/branches").await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert!(!body["items"].as_array().unwrap().is_empty());
    let first = &body["items"].as_array().unwrap()[0];
    assert!(first["branch_score"].is_number() || first["score"].is_number());
}

#[tokio::test]
async fn test_source_coverage() {
    let server = test_server().await;
    let resp = server.get("/api/v1/trees/1/source-coverage").await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert!(body["overall"].is_number());
}

#[tokio::test]
async fn test_analysis_runs() {
    let server = test_server().await;
    let resp = server.get("/api/v1/trees/1/analysis-runs").await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert!(!body["items"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_error_tree_not_found() {
    let server = test_server().await;
    let resp = server.get("/api/v1/trees/9999").await;
    resp.assert_status(axum::http::StatusCode::NOT_FOUND);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["error"]["code"], "TREE_NOT_FOUND");
}

#[tokio::test]
async fn test_error_invalid_severity() {
    let server = test_server().await;
    let resp = server
        .get("/api/v1/trees/1/findings?severity=invalid")
        .await;
    resp.assert_status(axum::http::StatusCode::BAD_REQUEST);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["error"]["code"], "INVALID_SEVERITY");
}

#[tokio::test]
async fn test_error_invalid_limit() {
    let server = test_server().await;
    let resp = server.get("/api/v1/trees/1/persons?limit=9999").await;
    resp.assert_status(axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_error_invalid_sort() {
    let server = test_server().await;
    let resp = server
        .get("/api/v1/trees/1/research-opportunities?sort=bad")
        .await;
    resp.assert_status(axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_openapi() {
    let server = test_server().await;
    let resp = server.get("/api/v1/openapi.json").await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["openapi"], "3.0.0");
}
