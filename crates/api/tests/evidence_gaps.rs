use axum_test::TestServer;
use neogenealogy_api::{create_router, state::AppState};
use neogenealogy_storage::{db::in_memory_pool, import_gedcom_content, Storage};
use serde_json::json;

async fn server() -> TestServer {
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
async fn test_outcome_without_evidence_has_gaps() {
    let server = server().await;
    let task_id = server
        .post("/api/v1/trees/1/research-tasks")
        .json(&json!({"title":"No ev"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let outcome_id = server
        .post(&format!("/api/v1/trees/1/research-tasks/{task_id}/outcome"))
        .json(&json!({"type":"CONFIRMED","summary":"sum"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let body: serde_json::Value = server
        .get(&format!("/api/v1/trees/1/research-outcomes/{outcome_id}"))
        .await
        .json();
    // CONFIRMED without support -> CRITICAL CONFIRMED_WITHOUT_SUPPORT
    let gaps = body["evidence_gaps"].as_array().unwrap();
    assert!(gaps
        .iter()
        .any(|g| g["code"] == "CONFIRMED_WITHOUT_SUPPORT" && g["severity"] == "CRITICAL"));
    // should not have NO_SUPPORTING_EVIDENCE for CONFIRMED
    assert!(!gaps.iter().any(|g| g["code"] == "NO_SUPPORTING_EVIDENCE"));
    // other type should have NO_SUPPORTING_EVIDENCE
    let task2 = server
        .post("/api/v1/trees/1/research-tasks")
        .json(&json!({"title":"Other"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let out2 = server
        .post(&format!("/api/v1/trees/1/research-tasks/{task2}/outcome"))
        .json(&json!({"type":"INCONCLUSIVE","summary":"sum"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let b2: serde_json::Value = server
        .get(&format!("/api/v1/trees/1/research-outcomes/{out2}"))
        .await
        .json();
    assert!(b2["evidence_gaps"]
        .as_array()
        .unwrap()
        .iter()
        .any(|g| g["code"] == "NO_SUPPORTING_EVIDENCE"));
}

#[tokio::test]
async fn test_single_support_gap_and_no_citation() {
    let server = server().await;
    let task_id = server
        .post("/api/v1/trees/1/research-tasks")
        .json(&json!({"title":"Single"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let outcome_id = server
        .post(&format!("/api/v1/trees/1/research-tasks/{task_id}/outcome"))
        .json(&json!({"type":"CONFIRMED","summary":"sum"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let src = server
        .post("/api/v1/trees/1/sources")
        .json(&json!({"title":"Src","type":"BOOK"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let ev = server
        .post("/api/v1/trees/1/evidence")
        .json(&json!({"source_id":src,"statement":"stmt"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    server
        .post(&format!(
            "/api/v1/trees/1/research-outcomes/{outcome_id}/evidence/{ev}"
        ))
        .json(&json!({"relationship":"SUPPORTS"}))
        .await;
    let body: serde_json::Value = server
        .get(&format!("/api/v1/trees/1/research-outcomes/{outcome_id}"))
        .await
        .json();
    let gaps = body["evidence_gaps"].as_array().unwrap();
    assert!(gaps
        .iter()
        .any(|g| g["code"] == "SINGLE_SUPPORTING_EVIDENCE" && g["severity"] == "WARNING"));
    assert!(gaps.iter().any(|g| g["code"] == "NO_CITATION"));
    assert!(gaps.iter().any(|g| g["code"] == "SINGLE_SOURCE"));
    // add citation -> NO_CITATION disappears
    let cit = server
        .post(&format!("/api/v1/trees/1/sources/{src}/citations"))
        .json(&json!({"locator":"folio"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    // need new evidence with citation; update existing evidence citation via PATCH
    server
        .patch(&format!("/api/v1/trees/1/evidence/{ev}"))
        .json(&json!({"citation_id":cit}))
        .await;
    let body2: serde_json::Value = server
        .get(&format!("/api/v1/trees/1/research-outcomes/{outcome_id}"))
        .await
        .json();
    assert!(!body2["evidence_gaps"]
        .as_array()
        .unwrap()
        .iter()
        .any(|g| g["code"] == "NO_CITATION"));
}

#[tokio::test]
async fn test_contradictory_gap() {
    let server = server().await;
    let task_id = server
        .post("/api/v1/trees/1/research-tasks")
        .json(&json!({"title":"Contra"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let outcome_id = server
        .post(&format!("/api/v1/trees/1/research-tasks/{task_id}/outcome"))
        .json(&json!({"type":"CONFIRMED","summary":"sum"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let src = server
        .post("/api/v1/trees/1/sources")
        .json(&json!({"title":"Src","type":"BOOK"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let ev1 = server
        .post("/api/v1/trees/1/evidence")
        .json(&json!({"source_id":src,"statement":"s1"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let ev2 = server
        .post("/api/v1/trees/1/evidence")
        .json(&json!({"source_id":src,"statement":"c1"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    server
        .post(&format!(
            "/api/v1/trees/1/research-outcomes/{outcome_id}/evidence/{ev1}"
        ))
        .json(&json!({"relationship":"SUPPORTS"}))
        .await;
    server
        .post(&format!(
            "/api/v1/trees/1/research-outcomes/{outcome_id}/evidence/{ev2}"
        ))
        .json(&json!({"relationship":"CONTRADICTS"}))
        .await;
    let body: serde_json::Value = server
        .get(&format!("/api/v1/trees/1/research-outcomes/{outcome_id}"))
        .await
        .json();
    assert!(body["evidence_gaps"]
        .as_array()
        .unwrap()
        .iter()
        .any(|g| g["code"] == "CONTRADICTORY_EVIDENCE"));
    // detach -> disappears
    server
        .delete(&format!(
            "/api/v1/trees/1/research-outcomes/{outcome_id}/evidence/{ev2}"
        ))
        .await;
    let body2: serde_json::Value = server
        .get(&format!("/api/v1/trees/1/research-outcomes/{outcome_id}"))
        .await
        .json();
    assert!(!body2["evidence_gaps"]
        .as_array()
        .unwrap()
        .iter()
        .any(|g| g["code"] == "CONTRADICTORY_EVIDENCE"));
}

#[tokio::test]
async fn test_single_source_vs_multiple() {
    let server = server().await;
    let task_id = server
        .post("/api/v1/trees/1/research-tasks")
        .json(&json!({"title":"SrcGap"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let outcome_id = server
        .post(&format!("/api/v1/trees/1/research-tasks/{task_id}/outcome"))
        .json(&json!({"type":"CONFIRMED","summary":"sum"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let src1 = server
        .post("/api/v1/trees/1/sources")
        .json(&json!({"title":"Src1","type":"BOOK"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let src2 = server
        .post("/api/v1/trees/1/sources")
        .json(&json!({"title":"Src2","type":"CENSUS"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let ev1 = server
        .post("/api/v1/trees/1/evidence")
        .json(&json!({"source_id":src1,"statement":"s1"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let ev2 = server
        .post("/api/v1/trees/1/evidence")
        .json(&json!({"source_id":src2,"statement":"s2"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    server
        .post(&format!(
            "/api/v1/trees/1/research-outcomes/{outcome_id}/evidence/{ev1}"
        ))
        .json(&json!({"relationship":"SUPPORTS"}))
        .await;
    let body1: serde_json::Value = server
        .get(&format!("/api/v1/trees/1/research-outcomes/{outcome_id}"))
        .await
        .json();
    assert!(body1["evidence_gaps"]
        .as_array()
        .unwrap()
        .iter()
        .any(|g| g["code"] == "SINGLE_SOURCE"));
    server
        .post(&format!(
            "/api/v1/trees/1/research-outcomes/{outcome_id}/evidence/{ev2}"
        ))
        .json(&json!({"relationship":"SUPPORTS"}))
        .await;
    let body2: serde_json::Value = server
        .get(&format!("/api/v1/trees/1/research-outcomes/{outcome_id}"))
        .await
        .json();
    assert!(!body2["evidence_gaps"]
        .as_array()
        .unwrap()
        .iter()
        .any(|g| g["code"] == "SINGLE_SOURCE"));
}

#[tokio::test]
async fn test_list_includes_gaps_and_filter() {
    let server = server().await;
    let task1 = server
        .post("/api/v1/trees/1/research-tasks")
        .json(&json!({"title":"ListGap1"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let out1 = server
        .post(&format!("/api/v1/trees/1/research-tasks/{task1}/outcome"))
        .json(&json!({"type":"CONFIRMED","summary":"no ev"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let task2 = server
        .post("/api/v1/trees/1/research-tasks")
        .json(&json!({"title":"ListGap2"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let out2 = server
        .post(&format!("/api/v1/trees/1/research-tasks/{task2}/outcome"))
        .json(&json!({"type":"CONFIRMED","summary":"with ev"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let src = server
        .post("/api/v1/trees/1/sources")
        .json(&json!({"title":"Src","type":"BOOK"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let ev = server
        .post("/api/v1/trees/1/evidence")
        .json(&json!({"source_id":src,"statement":"stmt"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    server
        .post(&format!(
            "/api/v1/trees/1/research-outcomes/{out2}/evidence/{ev}"
        ))
        .json(&json!({"relationship":"SUPPORTS"}))
        .await;

    let resp = server
        .get("/api/v1/trees/1/research-outcomes?limit=10")
        .await;
    let body: serde_json::Value = resp.json();
    for item in body["items"].as_array().unwrap() {
        assert!(item.get("evidence_gaps").is_some());
    }
    // filter by gap
    let filt = server
        .get("/api/v1/trees/1/research-outcomes?gap=CONFIRMED_WITHOUT_SUPPORT")
        .await
        .json::<serde_json::Value>();
    assert!(filt["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|o| o["id"] == out1));
    assert!(!filt["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|o| o["id"] == out2));
    let filt2 = server
        .get("/api/v1/trees/1/research-outcomes?gap=SINGLE_SUPPORTING_EVIDENCE")
        .await
        .json::<serde_json::Value>();
    assert!(filt2["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|o| o["id"] == out2));
}

#[tokio::test]
async fn test_summary_includes_gaps() {
    let server = server().await;
    let resp = server.get("/api/v1/trees/1/research/summary").await;
    let body: serde_json::Value = resp.json();
    assert!(body.get("evidence_gaps").is_some());
    assert!(body["evidence_gaps"].get("critical").is_some());
    assert!(body["evidence_gaps"].get("warning").is_some());
    assert!(body["evidence_gaps"].get("info").is_some());
}

#[tokio::test]
async fn test_outcome_type_change_updates_gaps() {
    let server = server().await;
    let task_id = server
        .post("/api/v1/trees/1/research-tasks")
        .json(&json!({"title":"ChangeType"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let outcome_id = server
        .post(&format!("/api/v1/trees/1/research-tasks/{task_id}/outcome"))
        .json(&json!({"type":"INCONCLUSIVE","summary":"sum"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let body: serde_json::Value = server
        .get(&format!("/api/v1/trees/1/research-outcomes/{outcome_id}"))
        .await
        .json();
    assert!(body["evidence_gaps"]
        .as_array()
        .unwrap()
        .iter()
        .any(|g| g["code"] == "NO_SUPPORTING_EVIDENCE"));
    // change to CONFIRMED -> should become CONFIRMED_WITHOUT_SUPPORT
    server
        .patch(&format!("/api/v1/trees/1/research-outcomes/{outcome_id}"))
        .json(&json!({"type":"CONFIRMED"}))
        .await;
    let body2: serde_json::Value = server
        .get(&format!("/api/v1/trees/1/research-outcomes/{outcome_id}"))
        .await
        .json();
    assert!(body2["evidence_gaps"]
        .as_array()
        .unwrap()
        .iter()
        .any(|g| g["code"] == "CONFIRMED_WITHOUT_SUPPORT"));
    assert!(!body2["evidence_gaps"]
        .as_array()
        .unwrap()
        .iter()
        .any(|g| g["code"] == "NO_SUPPORTING_EVIDENCE"));
}
