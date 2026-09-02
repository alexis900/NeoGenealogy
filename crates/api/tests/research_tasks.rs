use axum_test::TestServer;
use neogenealogy_api::{create_router, state::AppState};
use neogenealogy_storage::{db::in_memory_pool, import_gedcom_content, Storage};
use serde_json::json;

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
async fn test_create_and_get_task() {
    let server = test_server().await;
    let resp = server
        .post("/api/v1/trees/1/research-tasks")
        .json(&json!({"title":"Find parents","description":"test"}))
        .await;
    resp.assert_status(axum::http::StatusCode::CREATED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["title"], "Find parents");
    let task_id = body["id"].as_i64().unwrap();
    let resp2 = server
        .get(&format!("/api/v1/trees/1/research-tasks/{task_id}"))
        .await;
    resp2.assert_status_ok();
    let body2: serde_json::Value = resp2.json();
    assert_eq!(body2["id"], task_id);
}

#[tokio::test]
async fn test_list_tasks_with_filters_and_pagination() {
    let server = test_server().await;
    for i in 0..3 {
        server
            .post("/api/v1/trees/1/research-tasks")
            .json(&json!({"title": format!("Task {}", i)}))
            .await;
    }
    let resp = server
        .get("/api/v1/trees/1/research-tasks?limit=2&offset=0")
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["items"].as_array().unwrap().len(), 2);
    assert!(body["pagination"]["total"].as_i64().unwrap() >= 3);
}

#[tokio::test]
async fn test_update_status_and_timestamps() {
    let server = test_server().await;
    let resp = server
        .post("/api/v1/trees/1/research-tasks")
        .json(&json!({"title":"To update"}))
        .await;
    let task_id = resp.json::<serde_json::Value>()["id"].as_i64().unwrap();
    let resp2 = server
        .patch(&format!("/api/v1/trees/1/research-tasks/{task_id}"))
        .json(&json!({"status":"IN_PROGRESS"}))
        .await;
    resp2.assert_status_ok();
    let body: serde_json::Value = resp2.json();
    assert_eq!(body["status"], "IN_PROGRESS");
    assert!(body["started_at"].is_string());
    let resp3 = server
        .patch(&format!("/api/v1/trees/1/research-tasks/{task_id}"))
        .json(&json!({"status":"RESOLVED","resolution":"done"}))
        .await;
    let body3: serde_json::Value = resp3.json();
    assert_eq!(body3["status"], "RESOLVED");
    assert!(body3["completed_at"].is_string());
}

#[tokio::test]
async fn test_create_from_opportunity() {
    let server = test_server().await;
    // get an opportunity
    let resp = server
        .get("/api/v1/trees/1/research-opportunities?limit=1")
        .await;
    let body: serde_json::Value = resp.json();
    let opp_id = body["items"].as_array().unwrap()[0]["id"].as_i64().unwrap();
    let resp2 = server
        .post(&format!(
            "/api/v1/trees/1/research-opportunities/{opp_id}/tasks"
        ))
        .json(&json!({"title":"Research from opp"}))
        .await;
    resp2.assert_status(axum::http::StatusCode::CREATED);
    let body2: serde_json::Value = resp2.json();
    assert_eq!(body2["opportunity_id"], opp_id);
}

#[tokio::test]
async fn test_duplicate_active_reuse_via_api() {
    let server = test_server().await;
    let resp = server
        .get("/api/v1/trees/1/research-opportunities?limit=1")
        .await;
    let opp_id = resp.json::<serde_json::Value>()["items"]
        .as_array()
        .unwrap()[0]["id"]
        .as_i64()
        .unwrap();
    let resp1 = server
        .post(&format!(
            "/api/v1/trees/1/research-opportunities/{opp_id}/tasks"
        ))
        .json(&json!({}))
        .await;
    let id1 = resp1.json::<serde_json::Value>()["id"].as_i64().unwrap();
    let resp2 = server
        .post(&format!(
            "/api/v1/trees/1/research-opportunities/{opp_id}/tasks"
        ))
        .json(&json!({}))
        .await;
    let id2 = resp2.json::<serde_json::Value>()["id"].as_i64().unwrap();
    assert_eq!(id1, id2);
}

#[tokio::test]
async fn test_delete() {
    let server = test_server().await;
    let resp = server
        .post("/api/v1/trees/1/research-tasks")
        .json(&json!({"title":"To delete"}))
        .await;
    let task_id = resp.json::<serde_json::Value>()["id"].as_i64().unwrap();
    let resp2 = server
        .delete(&format!("/api/v1/trees/1/research-tasks/{task_id}"))
        .await;
    resp2.assert_status(axum::http::StatusCode::NO_CONTENT);
    let resp3 = server
        .get(&format!("/api/v1/trees/1/research-tasks/{task_id}"))
        .await;
    resp3.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_404_and_400() {
    let server = test_server().await;
    let resp = server.get("/api/v1/trees/999/research-tasks").await;
    resp.assert_status(axum::http::StatusCode::NOT_FOUND);
    let resp2 = server
        .post("/api/v1/trees/1/research-tasks")
        .json(&json!({"title":""}))
        .await;
    resp2.assert_status(axum::http::StatusCode::BAD_REQUEST);
    let resp3 = server.get("/api/v1/trees/1/research-tasks?limit=999").await;
    resp3.assert_status(axum::http::StatusCode::BAD_REQUEST);
    let resp4 = server
        .post("/api/v1/trees/1/research-tasks")
        .json(&json!({"title":"ok"}))
        .await;
    let task_id = resp4.json::<serde_json::Value>()["id"].as_i64().unwrap();
    let resp5 = server
        .patch(&format!("/api/v1/trees/1/research-tasks/{task_id}"))
        .json(&json!({"status":"BAD_STATUS"}))
        .await;
    resp5.assert_status(axum::http::StatusCode::BAD_REQUEST);
}
