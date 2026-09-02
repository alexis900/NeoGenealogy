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
    // second tree for cross-tree tests
    import_gedcom_content(
        &pool,
        "0 @I1@ INDI\n1 NAME Second /Tree/\n",
        "second.ged",
        None,
    )
    .await
    .unwrap();
    let storage = Storage::new(pool);
    let state = AppState::new(storage);
    let app = create_router(state);
    TestServer::new(app).unwrap()
}

#[tokio::test]
async fn test_source_crud() {
    let server = test_server().await;
    // POST
    let resp = server.post("/api/v1/trees/1/sources").json(&json!({"title":"Registro parroquial","author":"Parroquia","publication":"Pub","date":"1874","type":"PARISH_RECORD"})).await;
    resp.assert_status(axum::http::StatusCode::CREATED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["title"], "Registro parroquial");
    assert_eq!(body["type"], "PARISH_RECORD");
    for f in ["id", "tree_id", "title", "type", "created_at", "updated_at"] {
        assert!(body.get(f).is_some(), "missing {f}");
    }
    let sid = body["id"].as_i64().unwrap();
    // GET
    let resp2 = server.get(&format!("/api/v1/trees/1/sources/{sid}")).await;
    resp2.assert_status_ok();
    assert_eq!(resp2.json::<serde_json::Value>()["id"], sid);
    // PATCH
    let resp3 = server
        .patch(&format!("/api/v1/trees/1/sources/{sid}"))
        .json(&json!({"title":"Updated"}))
        .await;
    resp3.assert_status_ok();
    assert_eq!(resp3.json::<serde_json::Value>()["title"], "Updated");
    // LIST
    let resp4 = server.get("/api/v1/trees/1/sources?limit=10").await;
    let body4: serde_json::Value = resp4.json();
    assert!(!body4["items"].as_array().unwrap().is_empty());
    // DELETE
    let del = server
        .delete(&format!("/api/v1/trees/1/sources/{sid}"))
        .await;
    del.assert_status(axum::http::StatusCode::NO_CONTENT);
    let get2 = server.get(&format!("/api/v1/trees/1/sources/{sid}")).await;
    get2.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_source_errors_and_filters() {
    let server = test_server().await;
    // invalid type
    let resp = server
        .post("/api/v1/trees/1/sources")
        .json(&json!({"title":"T","type":"INVALID"}))
        .await;
    resp.assert_status(axum::http::StatusCode::BAD_REQUEST);
    assert_eq!(
        resp.json::<serde_json::Value>()["error"]["code"],
        "INVALID_SOURCE_TYPE"
    );
    // empty title
    let resp2 = server
        .post("/api/v1/trees/1/sources")
        .json(&json!({"title":"   ","type":"BOOK"}))
        .await;
    resp2.assert_status(axum::http::StatusCode::BAD_REQUEST);
    // pagination
    for i in 0..3 {
        server
            .post("/api/v1/trees/1/sources")
            .json(&json!({"title": format!("Src {i}"),"type":"BOOK"}))
            .await;
    }
    let resp3 = server
        .get("/api/v1/trees/1/sources?type=BOOK&limit=2&offset=0")
        .await;
    let body3: serde_json::Value = resp3.json();
    assert!(body3["items"]
        .as_array()
        .unwrap()
        .iter()
        .all(|v| v["type"] == "BOOK"));
    // cross-tree
    let sid = server
        .post("/api/v1/trees/1/sources")
        .json(&json!({"title":"Cross","type":"BOOK"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let cross = server.get(&format!("/api/v1/trees/2/sources/{sid}")).await;
    cross.assert_status(axum::http::StatusCode::NOT_FOUND);
    // tree isolation list
    let resp4 = server.get("/api/v1/trees/2/sources").await;
    assert_eq!(resp4.json::<serde_json::Value>()["pagination"]["total"], 0);
}

#[tokio::test]
async fn test_citation_crud() {
    let server = test_server().await;
    let sid = server
        .post("/api/v1/trees/1/sources")
        .json(&json!({"title":"Src","type":"BOOK"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    // POST citation
    let resp = server
        .post(&format!("/api/v1/trees/1/sources/{sid}/citations"))
        .json(&json!({"locator":"Libro III folio 42","text":"Partida"}))
        .await;
    resp.assert_status(axum::http::StatusCode::CREATED);
    let cid = resp.json::<serde_json::Value>()["id"].as_i64().unwrap();
    // GET citation
    let resp2 = server
        .get(&format!("/api/v1/trees/1/citations/{cid}"))
        .await;
    resp2.assert_status_ok();
    assert_eq!(
        resp2.json::<serde_json::Value>()["locator"],
        "Libro III folio 42"
    );
    // PATCH
    let resp3 = server
        .patch(&format!("/api/v1/trees/1/citations/{cid}"))
        .json(&json!({"locator":"New loc"}))
        .await;
    assert_eq!(resp3.json::<serde_json::Value>()["locator"], "New loc");
    // LIST citations for source
    let resp4 = server
        .get(&format!("/api/v1/trees/1/sources/{sid}/citations"))
        .await;
    assert!(!resp4.json::<serde_json::Value>()["items"]
        .as_array()
        .unwrap()
        .is_empty());
    // DELETE
    let del = server
        .delete(&format!("/api/v1/trees/1/citations/{cid}"))
        .await;
    del.assert_status(axum::http::StatusCode::NO_CONTENT);
    let get2 = server
        .get(&format!("/api/v1/trees/1/citations/{cid}"))
        .await;
    get2.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_citation_cross_tree() {
    let server = test_server().await;
    let sid1 = server
        .post("/api/v1/trees/1/sources")
        .json(&json!({"title":"Src1","type":"BOOK"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let resp = server
        .post(&format!("/api/v1/trees/2/sources/{sid1}/citations"))
        .json(&json!({"locator":"x"}))
        .await;
    resp.assert_status(axum::http::StatusCode::NOT_FOUND);
    // create citation then try get via wrong tree
    let cid = server
        .post(&format!("/api/v1/trees/1/sources/{sid1}/citations"))
        .json(&json!({"locator":"loc"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let get_cross = server
        .get(&format!("/api/v1/trees/2/citations/{cid}"))
        .await;
    get_cross.assert_status(axum::http::StatusCode::NOT_FOUND);
    // cascade check: delete source deletes citation
    let sid2 = server
        .post("/api/v1/trees/1/sources")
        .json(&json!({"title":"Src2","type":"BOOK"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let cid2 = server
        .post(&format!("/api/v1/trees/1/sources/{sid2}/citations"))
        .json(&json!({"locator":"loc2"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    server
        .delete(&format!("/api/v1/trees/1/sources/{sid2}"))
        .await;
    let get_cid2 = server
        .get(&format!("/api/v1/trees/1/citations/{cid2}"))
        .await;
    get_cid2.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_evidence_crud() {
    let server = test_server().await;
    let sid = server
        .post("/api/v1/trees/1/sources")
        .json(&json!({"title":"Src","type":"CENSUS"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let cid = server
        .post(&format!("/api/v1/trees/1/sources/{sid}/citations"))
        .json(&json!({"locator":"p42"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    // POST evidence with citation
    let resp = server.post("/api/v1/trees/1/evidence").json(&json!({"source_id":sid,"citation_id":cid,"statement":"La partida identifica a Josep","notes":"nota"}))
        .await;
    resp.assert_status(axum::http::StatusCode::CREATED);
    let eid = resp.json::<serde_json::Value>()["id"].as_i64().unwrap();
    // GET
    let resp2 = server.get(&format!("/api/v1/trees/1/evidence/{eid}")).await;
    resp2.assert_status_ok();
    let body2: serde_json::Value = resp2.json();
    assert_eq!(body2["statement"], "La partida identifica a Josep");
    assert!(body2.get("source").is_some());
    assert!(body2.get("citation").is_some());
    // POST without citation
    let resp3 = server
        .post("/api/v1/trees/1/evidence")
        .json(&json!({"source_id":sid,"statement":"No citation"}))
        .await;
    resp3.assert_status(axum::http::StatusCode::CREATED);
    // PATCH
    let resp4 = server
        .patch(&format!("/api/v1/trees/1/evidence/{eid}"))
        .json(&json!({"statement":"Updated"}))
        .await;
    assert_eq!(resp4.json::<serde_json::Value>()["statement"], "Updated");
    // LIST
    let resp5 = server.get("/api/v1/trees/1/evidence?limit=10").await;
    assert!(
        resp5.json::<serde_json::Value>()["items"]
            .as_array()
            .unwrap()
            .len()
            >= 2
    );
    // DELETE
    let del = server
        .delete(&format!("/api/v1/trees/1/evidence/{eid}"))
        .await;
    del.assert_status(axum::http::StatusCode::NO_CONTENT);
    let get2 = server.get(&format!("/api/v1/trees/1/evidence/{eid}")).await;
    get2.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_evidence_errors() {
    let server = test_server().await;
    let sid1 = server
        .post("/api/v1/trees/1/sources")
        .json(&json!({"title":"Src1","type":"BOOK"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let sid2 = server
        .post("/api/v1/trees/2/sources")
        .json(&json!({"title":"Src2","type":"BOOK"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    // cross-tree source
    let resp = server
        .post("/api/v1/trees/1/evidence")
        .json(&json!({"source_id":sid2,"statement":"stmt"}))
        .await;
    resp.assert_status(axum::http::StatusCode::NOT_FOUND);
    // empty statement
    let resp2 = server
        .post("/api/v1/trees/1/evidence")
        .json(&json!({"source_id":sid1,"statement":"   "}))
        .await;
    resp2.assert_status(axum::http::StatusCode::BAD_REQUEST);
    assert_eq!(
        resp2.json::<serde_json::Value>()["error"]["code"],
        "INVALID_STATEMENT"
    );
    // citation mismatch
    let cid2 = server
        .post(&format!("/api/v1/trees/2/sources/{sid2}/citations"))
        .json(&json!({"locator":"loc"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let resp3 = server
        .post("/api/v1/trees/1/evidence")
        .json(&json!({"source_id":sid1,"citation_id":cid2,"statement":"stmt"}))
        .await;
    resp3.assert_status(axum::http::StatusCode::BAD_REQUEST);
    // tree isolation list
    server
        .post("/api/v1/trees/1/evidence")
        .json(&json!({"source_id":sid1,"statement":"t1"}))
        .await;
    let resp4 = server.get("/api/v1/trees/2/evidence").await;
    assert_eq!(resp4.json::<serde_json::Value>()["pagination"]["total"], 0);
}

#[tokio::test]
async fn test_outcome_evidence_attach() {
    let server = test_server().await;
    // create task + outcome
    let task_id = server
        .post("/api/v1/trees/1/research-tasks")
        .json(&json!({"title":"Task for evidence"}))
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
    let sid = server
        .post("/api/v1/trees/1/sources")
        .json(&json!({"title":"Src","type":"PARISH_RECORD"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let eid = server
        .post("/api/v1/trees/1/evidence")
        .json(&json!({"source_id":sid,"statement":"Evidence stmt"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    // attach SUPPORTS
    let resp = server
        .post(&format!(
            "/api/v1/trees/1/research-outcomes/{outcome_id}/evidence/{eid}"
        ))
        .json(&json!({"relationship":"SUPPORTS"}))
        .await;
    resp.assert_status(axum::http::StatusCode::CREATED);
    assert_eq!(resp.json::<serde_json::Value>()["relationship"], "SUPPORTS");
    // duplicate -> 409
    let resp2 = server
        .post(&format!(
            "/api/v1/trees/1/research-outcomes/{outcome_id}/evidence/{eid}"
        ))
        .json(&json!({"relationship":"SUPPORTS"}))
        .await;
    resp2.assert_status(axum::http::StatusCode::CONFLICT);
    assert_eq!(
        resp2.json::<serde_json::Value>()["error"]["code"],
        "EVIDENCE_ALREADY_ATTACHED"
    );
    // invalid relationship
    let sid2 = server
        .post("/api/v1/trees/1/sources")
        .json(&json!({"title":"Src2","type":"BOOK"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let eid2 = server
        .post("/api/v1/trees/1/evidence")
        .json(&json!({"source_id":sid2,"statement":"Other"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let resp3 = server
        .post(&format!(
            "/api/v1/trees/1/research-outcomes/{outcome_id}/evidence/{eid2}"
        ))
        .json(&json!({"relationship":"INVALID"}))
        .await;
    resp3.assert_status(axum::http::StatusCode::BAD_REQUEST);
    assert_eq!(
        resp3.json::<serde_json::Value>()["error"]["code"],
        "INVALID_EVIDENCE_RELATIONSHIP"
    );
    // attach CONTRADICTS
    let resp4 = server
        .post(&format!(
            "/api/v1/trees/1/research-outcomes/{outcome_id}/evidence/{eid2}"
        ))
        .json(&json!({"relationship":"CONTRADICTS"}))
        .await;
    resp4.assert_status(axum::http::StatusCode::CREATED);
    assert_eq!(
        resp4.json::<serde_json::Value>()["relationship"],
        "CONTRADICTS"
    );
    // list outcome evidence
    let resp5 = server
        .get(&format!(
            "/api/v1/trees/1/research-outcomes/{outcome_id}/evidence"
        ))
        .await;
    resp5.assert_status_ok();
    let body5: serde_json::Value = resp5.json();
    assert_eq!(body5["items"].as_array().unwrap().len(), 2);
    // get outcome includes evidence
    let resp6 = server
        .get(&format!("/api/v1/trees/1/research-outcomes/{outcome_id}"))
        .await;
    let body6: serde_json::Value = resp6.json();
    assert!(body6.get("evidence").is_some());
    assert_eq!(body6["evidence"].as_array().unwrap().len(), 2);
    assert!(body6["evidence"][0].get("source").is_some());
    assert!(
        body6["evidence"][0].get("citation").is_some()
            || body6["evidence"][0]["citation"].is_null()
    );
    // detach
    let del = server
        .delete(&format!(
            "/api/v1/trees/1/research-outcomes/{outcome_id}/evidence/{eid}"
        ))
        .await;
    del.assert_status(axum::http::StatusCode::NO_CONTENT);
    let resp7 = server
        .get(&format!(
            "/api/v1/trees/1/research-outcomes/{outcome_id}/evidence"
        ))
        .await;
    assert_eq!(
        resp7.json::<serde_json::Value>()["items"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    // cross-tree
    let sid_cross = server
        .post("/api/v1/trees/2/sources")
        .json(&json!({"title":"Cross","type":"BOOK"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let eid_cross = server
        .post("/api/v1/trees/2/evidence")
        .json(&json!({"source_id":sid_cross,"statement":"cross"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let resp8 = server
        .post(&format!(
            "/api/v1/trees/1/research-outcomes/{outcome_id}/evidence/{eid_cross}"
        ))
        .json(&json!({"relationship":"SUPPORTS"}))
        .await;
    resp8.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_outcome_evidence_cascade() {
    let server = test_server().await;
    let task_id = server
        .post("/api/v1/trees/1/research-tasks")
        .json(&json!({"title":"Cascade task"}))
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
    let sid = server
        .post("/api/v1/trees/1/sources")
        .json(&json!({"title":"Src","type":"BOOK"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let eid = server
        .post("/api/v1/trees/1/evidence")
        .json(&json!({"source_id":sid,"statement":"stmt"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    server
        .post(&format!(
            "/api/v1/trees/1/research-outcomes/{outcome_id}/evidence/{eid}"
        ))
        .json(&json!({"relationship":"SUPPORTS"}))
        .await;
    // delete evidence should remove relation but outcome remains
    server
        .delete(&format!("/api/v1/trees/1/evidence/{eid}"))
        .await;
    let resp = server
        .get(&format!(
            "/api/v1/trees/1/research-outcomes/{outcome_id}/evidence"
        ))
        .await;
    assert_eq!(
        resp.json::<serde_json::Value>()["items"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    let outcome_get = server
        .get(&format!("/api/v1/trees/1/research-outcomes/{outcome_id}"))
        .await;
    outcome_get.assert_status_ok();
    // delete outcome should remove link but evidence remains (if not deleted)
    let eid2 = server
        .post("/api/v1/trees/1/evidence")
        .json(&json!({"source_id":sid,"statement":"stmt2"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let task2 = server
        .post("/api/v1/trees/1/research-tasks")
        .json(&json!({"title":"Task2 cascade"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let outcome2 = server
        .post(&format!("/api/v1/trees/1/research-tasks/{task2}/outcome"))
        .json(&json!({"type":"CONFIRMED","summary":"sum2"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    server
        .post(&format!(
            "/api/v1/trees/1/research-outcomes/{outcome2}/evidence/{eid2}"
        ))
        .json(&json!({"relationship":"SUPPORTS"}))
        .await;
    server
        .delete(&format!("/api/v1/trees/1/research-outcomes/{outcome2}"))
        .await;
    // evidence should still exist
    let ev_get = server
        .get(&format!("/api/v1/trees/1/evidence/{eid2}"))
        .await;
    ev_get.assert_status_ok();
}

#[tokio::test]
async fn test_outcome_without_evidence_has_empty_array() {
    let server = test_server().await;
    let task_id = server
        .post("/api/v1/trees/1/research-tasks")
        .json(&json!({"title":"No evidence task"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let outcome_id = server
        .post(&format!("/api/v1/trees/1/research-tasks/{task_id}/outcome"))
        .json(&json!({"type":"INCONCLUSIVE","summary":"no ev"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let resp = server
        .get(&format!("/api/v1/trees/1/research-outcomes/{outcome_id}"))
        .await;
    let body: serde_json::Value = resp.json();
    assert_eq!(body["evidence"].as_array().unwrap().len(), 0);
}
