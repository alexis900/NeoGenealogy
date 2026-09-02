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
async fn test_outcome_without_evidence_has_no_evidence_assessment() {
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
    let resp = server
        .get(&format!("/api/v1/trees/1/research-outcomes/{outcome_id}"))
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["evidence_assessment"]["status"], "NO_EVIDENCE");
    assert_eq!(body["evidence_assessment"]["score"], 0);
    assert_eq!(body["evidence_assessment"]["evidence_total"], 0);
    assert!(body["evidence"].as_array().unwrap().is_empty());
    assert!(body["evidence_assessment"]["reasons"]
        .as_array()
        .unwrap()
        .is_empty());
}

#[tokio::test]
async fn test_outcome_with_supporting_evidence_weak_then_supported() {
    let server = server().await;
    let task_id = server
        .post("/api/v1/trees/1/research-tasks")
        .json(&json!({"title":"Weak"}))
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
    let resp = server
        .get(&format!("/api/v1/trees/1/research-outcomes/{outcome_id}"))
        .await;
    let body: serde_json::Value = resp.json();
    assert_eq!(body["evidence_assessment"]["status"], "WEAK");
    assert!(
        body["evidence_assessment"]["supporting_count"]
            .as_i64()
            .unwrap()
            >= 1
    );
    assert!(body["evidence_assessment"]["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["code"] == "SUPPORTING_EVIDENCE"));

    // add second supporting -> SUPPORTED
    let ev2 = server
        .post("/api/v1/trees/1/evidence")
        .json(&json!({"source_id":src,"statement":"stmt2"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    server
        .post(&format!(
            "/api/v1/trees/1/research-outcomes/{outcome_id}/evidence/{ev2}"
        ))
        .json(&json!({"relationship":"SUPPORTS"}))
        .await;
    let resp2 = server
        .get(&format!("/api/v1/trees/1/research-outcomes/{outcome_id}"))
        .await;
    let body2: serde_json::Value = resp2.json();
    assert_eq!(body2["evidence_assessment"]["status"], "SUPPORTED");
    assert!(
        body2["evidence_assessment"]["score"].as_i64().unwrap()
            > body["evidence_assessment"]["score"].as_i64().unwrap()
    );
}

#[tokio::test]
async fn test_outcome_with_citation_and_multiple_sources_strongly() {
    let server = server().await;
    let task_id = server
        .post("/api/v1/trees/1/research-tasks")
        .json(&json!({"title":"Strong"}))
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
    let cit1 = server
        .post(&format!("/api/v1/trees/1/sources/{src1}/citations"))
        .json(&json!({"locator":"folio 1"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let ev1 = server
        .post("/api/v1/trees/1/evidence")
        .json(&json!({"source_id":src1,"citation_id":cit1,"statement":"stmt1"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let ev2 = server
        .post("/api/v1/trees/1/evidence")
        .json(&json!({"source_id":src2,"statement":"stmt2"}))
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
        .json(&json!({"relationship":"SUPPORTS"}))
        .await;
    let resp = server
        .get(&format!("/api/v1/trees/1/research-outcomes/{outcome_id}"))
        .await;
    let body: serde_json::Value = resp.json();
    assert_eq!(body["evidence_assessment"]["status"], "STRONGLY_SUPPORTED");
    assert!(body["evidence_assessment"]["cited_count"].as_i64().unwrap() >= 1);
    assert!(
        body["evidence_assessment"]["sources_count"]
            .as_i64()
            .unwrap()
            >= 2
    );
    // reasons should include multiple sources and citation
    let reasons = body["evidence_assessment"]["reasons"].as_array().unwrap();
    assert!(reasons.iter().any(|r| r["code"] == "MULTIPLE_SOURCES"));
    assert!(reasons
        .iter()
        .any(|r| r["code"] == "SUPPORTING_EVIDENCE_HAS_CITATION"));
}

#[tokio::test]
async fn test_outcome_mixed() {
    let server = server().await;
    let task_id = server
        .post("/api/v1/trees/1/research-tasks")
        .json(&json!({"title":"Mixed"}))
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
    let ev_s = server
        .post("/api/v1/trees/1/evidence")
        .json(&json!({"source_id":src,"statement":"support"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let ev_c = server
        .post("/api/v1/trees/1/evidence")
        .json(&json!({"source_id":src,"statement":"contra"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    server
        .post(&format!(
            "/api/v1/trees/1/research-outcomes/{outcome_id}/evidence/{ev_s}"
        ))
        .json(&json!({"relationship":"SUPPORTS"}))
        .await;
    server
        .post(&format!(
            "/api/v1/trees/1/research-outcomes/{outcome_id}/evidence/{ev_c}"
        ))
        .json(&json!({"relationship":"CONTRADICTS"}))
        .await;
    let resp = server
        .get(&format!("/api/v1/trees/1/research-outcomes/{outcome_id}"))
        .await;
    let body: serde_json::Value = resp.json();
    assert_eq!(body["evidence_assessment"]["status"], "MIXED");
    assert_eq!(body["evidence_assessment"]["contradicting_count"], 1);
    assert!(body["evidence_assessment"]["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["code"] == "CONTRADICTING_EVIDENCE"));
}

#[tokio::test]
async fn test_history_filter_by_assessment() {
    let server = server().await;
    // create outcomes with different assessments
    let task1 = server
        .post("/api/v1/trees/1/research-tasks")
        .json(&json!({"title":"Hist1"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let outcome1 = server
        .post(&format!("/api/v1/trees/1/research-tasks/{task1}/outcome"))
        .json(&json!({"type":"CONFIRMED","summary":"no ev"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let task2 = server
        .post("/api/v1/trees/1/research-tasks")
        .json(&json!({"title":"Hist2"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let outcome2 = server
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
            "/api/v1/trees/1/research-outcomes/{outcome2}/evidence/{ev}"
        ))
        .json(&json!({"relationship":"SUPPORTS"}))
        .await;
    // filter NO_EVIDENCE should include outcome1
    let resp = server
        .get("/api/v1/trees/1/research-outcomes?assessment_status=NO_EVIDENCE")
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert!(body["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|o| o["id"] == outcome1));
    assert!(!body["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|o| o["id"] == outcome2));
    // filter WEAK should include outcome2 (single supporting)
    let resp2 = server
        .get("/api/v1/trees/1/research-outcomes?assessment_status=WEAK")
        .await;
    let body2: serde_json::Value = resp2.json();
    assert!(body2["items"]
        .as_array()
        .unwrap()
        .iter()
        .any(|o| o["id"] == outcome2));
}

#[tokio::test]
async fn test_list_includes_assessment_batch() {
    let server = server().await;
    // ensure list includes assessment without N+1
    for i in 0..2 {
        let task_id = server
            .post("/api/v1/trees/1/research-tasks")
            .json(&json!({"title": format!("List batch {i}")}))
            .await
            .json::<serde_json::Value>()["id"]
            .as_i64()
            .unwrap();
        server
            .post(&format!("/api/v1/trees/1/research-tasks/{task_id}/outcome"))
            .json(&json!({"type":"CONFIRMED","summary": format!("sum {i}")}))
            .await;
    }
    let resp = server
        .get("/api/v1/trees/1/research-outcomes?limit=10")
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    for item in body["items"].as_array().unwrap() {
        assert!(item.get("evidence_assessment").is_some());
        assert!(item["evidence_assessment"].get("score").is_some());
        assert!(item["evidence_assessment"].get("status").is_some());
        assert!(item["evidence_assessment"].get("reasons").is_some());
    }
}

#[tokio::test]
async fn test_evidence_deleted_updates_assessment() {
    let server = server().await;
    let task_id = server
        .post("/api/v1/trees/1/research-tasks")
        .json(&json!({"title":"Update"}))
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
    let before = server
        .get(&format!("/api/v1/trees/1/research-outcomes/{outcome_id}"))
        .await
        .json::<serde_json::Value>();
    assert_eq!(before["evidence_assessment"]["status"], "WEAK");
    // delete evidence
    server
        .delete(&format!("/api/v1/trees/1/evidence/{ev}"))
        .await;
    let after = server
        .get(&format!("/api/v1/trees/1/research-outcomes/{outcome_id}"))
        .await
        .json::<serde_json::Value>();
    assert_eq!(after["evidence_assessment"]["status"], "NO_EVIDENCE");
    assert_eq!(after["evidence_assessment"]["evidence_total"], 0);
}

#[tokio::test]
async fn test_summary_includes_assessment() {
    let server = server().await;
    let resp = server.get("/api/v1/trees/1/research/summary").await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert!(body.get("evidence").is_some());
    assert!(body.get("assessment").is_some());
    assert!(body["assessment"].get("no_evidence").is_some());
}
