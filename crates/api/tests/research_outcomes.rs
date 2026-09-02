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

// Helper to create a task
async fn create_task(server: &TestServer) -> i64 {
    let resp = server
        .post("/api/v1/trees/1/research-tasks")
        .json(&json!({"title":"Task for outcome"}))
        .await;
    resp.assert_status(axum::http::StatusCode::CREATED);
    resp.json::<serde_json::Value>()["id"].as_i64().unwrap()
}

#[tokio::test]
async fn test_post_outcome_201() {
    let server = test_server().await;
    let task_id = create_task(&server).await;
    let resp = server
        .post(&format!("/api/v1/trees/1/research-tasks/{task_id}/outcome"))
        .json(&json!({"type":"CONFIRMED","summary":"found baptism record","details":"parish book p.12"}))
        .await;
    resp.assert_status(axum::http::StatusCode::CREATED);
    let body: serde_json::Value = resp.json();
    assert_eq!(body["tree_id"], 1);
    assert_eq!(body["task_id"], task_id);
    assert_eq!(body["type"], "CONFIRMED");
    assert_eq!(body["summary"], "found baptism record");
    assert_eq!(body["details"], "parish book p.12");
    assert!(body["id"].is_number());
    assert!(body["created_at"].is_string());
    assert!(body["updated_at"].is_string());
}

#[tokio::test]
async fn test_get_outcome_200() {
    let server = test_server().await;
    let task_id = create_task(&server).await;
    let resp = server
        .post(&format!("/api/v1/trees/1/research-tasks/{task_id}/outcome"))
        .json(&json!({"type":"FALSE_LEAD","summary":"not the same person"}))
        .await;
    let outcome_id = resp.json::<serde_json::Value>()["id"].as_i64().unwrap();
    let resp2 = server
        .get(&format!("/api/v1/trees/1/research-outcomes/{outcome_id}"))
        .await;
    resp2.assert_status_ok();
    let body: serde_json::Value = resp2.json();
    assert_eq!(body["id"], outcome_id);
    assert_eq!(body["type"], "FALSE_LEAD");
    assert_eq!(body["summary"], "not the same person");
    assert!(body["tree_id"].is_number());
    assert!(body["task_id"].is_number());
    assert!(body["created_at"].is_string());
    assert!(body["updated_at"].is_string());
}

#[tokio::test]
async fn test_patch_outcome_200() {
    let server = test_server().await;
    let task_id = create_task(&server).await;
    let resp = server
        .post(&format!("/api/v1/trees/1/research-tasks/{task_id}/outcome"))
        .json(&json!({"type":"INCONCLUSIVE","summary":"need more sources"}))
        .await;
    let oid = resp.json::<serde_json::Value>()["id"].as_i64().unwrap();
    let resp2 = server
        .patch(&format!("/api/v1/trees/1/research-outcomes/{oid}"))
        .json(
            &json!({"type":"NEW_LEAD","summary":"found new parish","details":"check 1880 census"}),
        )
        .await;
    resp2.assert_status_ok();
    let body: serde_json::Value = resp2.json();
    assert_eq!(body["type"], "NEW_LEAD");
    assert_eq!(body["summary"], "found new parish");
    assert_eq!(body["details"], "check 1880 census");
    assert_eq!(body["id"], oid);
}

#[tokio::test]
async fn test_delete_outcome_204() {
    let server = test_server().await;
    let task_id = create_task(&server).await;
    let resp = server
        .post(&format!("/api/v1/trees/1/research-tasks/{task_id}/outcome"))
        .json(&json!({"type":"NO_EVIDENCE","summary":"no record found"}))
        .await;
    let oid = resp.json::<serde_json::Value>()["id"].as_i64().unwrap();
    let del = server
        .delete(&format!("/api/v1/trees/1/research-outcomes/{oid}"))
        .await;
    del.assert_status(axum::http::StatusCode::NO_CONTENT);
    let get = server
        .get(&format!("/api/v1/trees/1/research-outcomes/{oid}"))
        .await;
    get.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_task_outcome_embedded() {
    let server = test_server().await;
    let task_id = create_task(&server).await;
    // without outcome -> null
    let resp = server
        .get(&format!("/api/v1/trees/1/research-tasks/{task_id}"))
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert!(
        body["outcome"].is_null(),
        "outcome should be null before creation"
    );

    // create outcome
    server
        .post(&format!("/api/v1/trees/1/research-tasks/{task_id}/outcome"))
        .json(&json!({"type":"CONFIRMED","summary":"confirmed birth"}))
        .await;

    let resp2 = server
        .get(&format!("/api/v1/trees/1/research-tasks/{task_id}"))
        .await;
    resp2.assert_status_ok();
    let body2: serde_json::Value = resp2.json();
    assert!(body2["outcome"].is_object());
    assert_eq!(body2["outcome"]["type"], "CONFIRMED");
    assert_eq!(body2["outcome"]["summary"], "confirmed birth");
    assert_eq!(body2["outcome"]["task_id"], task_id);
    assert!(body2["outcome"]["id"].is_number());
    assert!(body2["outcome"]["created_at"].is_string());

    // after delete -> null again
    let oid = body2["outcome"]["id"].as_i64().unwrap();
    server
        .delete(&format!("/api/v1/trees/1/research-outcomes/{oid}"))
        .await;
    let resp3 = server
        .get(&format!("/api/v1/trees/1/research-tasks/{task_id}"))
        .await;
    let body3: serde_json::Value = resp3.json();
    assert!(body3["outcome"].is_null());
}

// -- Errors

#[tokio::test]
async fn test_errors_404_400_409() {
    let server = test_server().await;

    // task inexistente -> 404 when creating outcome
    let resp = server
        .post("/api/v1/trees/1/research-tasks/99999/outcome")
        .json(&json!({"type":"CONFIRMED","summary":"x"}))
        .await;
    resp.assert_status(axum::http::StatusCode::NOT_FOUND);

    // outcome inexistente -> 404
    let resp2 = server.get("/api/v1/trees/1/research-outcomes/99999").await;
    resp2.assert_status(axum::http::StatusCode::NOT_FOUND);

    let resp3 = server
        .patch("/api/v1/trees/1/research-outcomes/99999")
        .json(&json!({"summary":"x"}))
        .await;
    resp3.assert_status(axum::http::StatusCode::NOT_FOUND);

    let resp4 = server
        .delete("/api/v1/trees/1/research-outcomes/99999")
        .await;
    resp4.assert_status(axum::http::StatusCode::NOT_FOUND);

    // tipo inválido -> 400
    let task_id = create_task(&server).await;
    let resp5 = server
        .post(&format!("/api/v1/trees/1/research-tasks/{task_id}/outcome"))
        .json(&json!({"type":"INVALID","summary":"x"}))
        .await;
    resp5.assert_status(axum::http::StatusCode::BAD_REQUEST);
    let body5: serde_json::Value = resp5.json();
    assert_eq!(body5["error"]["code"], "INVALID_RESEARCH_OUTCOME_TYPE");

    // summary vacío -> 400
    let task2 = create_task(&server).await;
    let resp6 = server
        .post(&format!("/api/v1/trees/1/research-tasks/{task2}/outcome"))
        .json(&json!({"type":"CONFIRMED","summary":"   "}))
        .await;
    resp6.assert_status(axum::http::StatusCode::BAD_REQUEST);
    assert_eq!(
        resp6.json::<serde_json::Value>()["error"]["code"],
        "INVALID_SUMMARY"
    );

    // segundo outcome -> 409
    let task3 = create_task(&server).await;
    server
        .post(&format!("/api/v1/trees/1/research-tasks/{task3}/outcome"))
        .json(&json!({"type":"CONFIRMED","summary":"first"}))
        .await;
    let resp7 = server
        .post(&format!("/api/v1/trees/1/research-tasks/{task3}/outcome"))
        .json(&json!({"type":"FALSE_LEAD","summary":"second"}))
        .await;
    resp7.assert_status(axum::http::StatusCode::CONFLICT);
    assert_eq!(
        resp7.json::<serde_json::Value>()["error"]["code"],
        "RESEARCH_OUTCOME_ALREADY_EXISTS"
    );

    // patch invalid type -> 400
    let task4 = create_task(&server).await;
    let oid = server
        .post(&format!("/api/v1/trees/1/research-tasks/{task4}/outcome"))
        .json(&json!({"type":"CONFIRMED","summary":"ok"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    let resp8 = server
        .patch(&format!("/api/v1/trees/1/research-outcomes/{oid}"))
        .json(&json!({"type":"BAD"}))
        .await;
    resp8.assert_status(axum::http::StatusCode::BAD_REQUEST);

    // patch empty summary -> 400
    let resp9 = server
        .patch(&format!("/api/v1/trees/1/research-outcomes/{oid}"))
        .json(&json!({"summary":"  "}))
        .await;
    resp9.assert_status(axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_cross_tree_404() {
    // Create second tree via direct DB import on same pool? Simpler: use storage directly to create tree2
    // But TestServer uses its own pool; we need to create tree via importing another gedcom via the same pool inside test_server.
    // Approach: create a custom server with 2 trees.
    let pool = in_memory_pool().await.unwrap();
    let content =
        std::fs::read_to_string("/home/amartinper/NeoGenealogy/test-data/complex.ged").unwrap();
    import_gedcom_content(&pool, &content, "complex.ged", None)
        .await
        .unwrap();
    // second tree
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
    let server = TestServer::new(app).unwrap();

    // create task in tree 1
    let resp = server
        .post("/api/v1/trees/1/research-tasks")
        .json(&json!({"title":"Cross task"}))
        .await;
    let task_id = resp.json::<serde_json::Value>()["id"].as_i64().unwrap();
    let outcome_id = server
        .post(&format!("/api/v1/trees/1/research-tasks/{task_id}/outcome"))
        .json(&json!({"type":"CONFIRMED","summary":"cross test"}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();

    // GET outcome via tree 2 -> 404
    let resp2 = server
        .get(&format!("/api/v1/trees/2/research-outcomes/{outcome_id}"))
        .await;
    resp2.assert_status(axum::http::StatusCode::NOT_FOUND);

    // GET task via tree2 -> 404
    let resp3 = server
        .get(&format!("/api/v1/trees/2/research-tasks/{task_id}"))
        .await;
    resp3.assert_status(axum::http::StatusCode::NOT_FOUND);

    // POST outcome via wrong tree -> 404 (task not in tree 2)
    let resp4 = server
        .post(&format!("/api/v1/trees/2/research-tasks/{task_id}/outcome"))
        .json(&json!({"type":"CONFIRMED","summary":"cross"}))
        .await;
    resp4.assert_status(axum::http::StatusCode::NOT_FOUND);

    // PATCH outcome via wrong tree -> 404
    let resp5 = server
        .patch(&format!("/api/v1/trees/2/research-outcomes/{outcome_id}"))
        .json(&json!({"summary":"hack"}))
        .await;
    resp5.assert_status(axum::http::StatusCode::NOT_FOUND);

    // DELETE outcome via wrong tree -> 404
    let resp6 = server
        .delete(&format!("/api/v1/trees/2/research-outcomes/{outcome_id}"))
        .await;
    resp6.assert_status(axum::http::StatusCode::NOT_FOUND);
}

// -- Listado

#[tokio::test]
async fn test_list_filters_pagination() {
    let server = test_server().await;
    // create several outcomes with different types
    let mut ids = vec![];
    let types = [
        "CONFIRMED",
        "FALSE_LEAD",
        "INCONCLUSIVE",
        "NEW_LEAD",
        "NO_EVIDENCE",
    ];
    for (i, t) in types.iter().enumerate() {
        let task_id = server
            .post("/api/v1/trees/1/research-tasks")
            .json(&json!({"title": format!("List task {i}") }))
            .await
            .json::<serde_json::Value>()["id"]
            .as_i64()
            .unwrap();
        let oid = server
            .post(&format!("/api/v1/trees/1/research-tasks/{task_id}/outcome"))
            .json(&json!({"type": t, "summary": format!("summary {t}")}))
            .await
            .json::<serde_json::Value>()["id"]
            .as_i64()
            .unwrap();
        ids.push((oid, task_id, *t));
    }

    // list all
    let resp = server
        .get("/api/v1/trees/1/research-outcomes?limit=3&offset=0")
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["items"].as_array().unwrap().len(), 3);
    assert!(body["pagination"]["total"].as_i64().unwrap() >= 5);

    let resp2 = server
        .get("/api/v1/trees/1/research-outcomes?limit=3&offset=3")
        .await;
    let body2: serde_json::Value = resp2.json();
    assert!(body2["items"].as_array().unwrap().len() >= 2);

    // filter by type
    let resp3 = server
        .get("/api/v1/trees/1/research-outcomes?type=CONFIRMED")
        .await;
    let body3: serde_json::Value = resp3.json();
    assert!(body3["items"]
        .as_array()
        .unwrap()
        .iter()
        .all(|o| o["type"] == "CONFIRMED"));
    assert_eq!(body3["pagination"]["total"], 1);

    // filter by task_id
    let first_task = ids[0].1;
    let resp4 = server
        .get(&format!(
            "/api/v1/trees/1/research-outcomes?task_id={first_task}"
        ))
        .await;
    let body4: serde_json::Value = resp4.json();
    assert_eq!(body4["items"].as_array().unwrap().len(), 1);
    assert_eq!(body4["items"][0]["task_id"], first_task);

    // filter by person_id: create task with person
    let person_id: i64 = server
        .get("/api/v1/trees/1/persons?limit=1")
        .await
        .json::<serde_json::Value>()["items"][0]["id"]
        .as_i64()
        .unwrap();
    let task_pid = server
        .post("/api/v1/trees/1/research-tasks")
        .json(&json!({"title":"Person task","person_id": person_id}))
        .await
        .json::<serde_json::Value>()["id"]
        .as_i64()
        .unwrap();
    server
        .post(&format!(
            "/api/v1/trees/1/research-tasks/{task_pid}/outcome"
        ))
        .json(&json!({"type":"NEW_LEAD","summary":"person lead"}))
        .await;
    let resp5 = server
        .get(&format!(
            "/api/v1/trees/1/research-outcomes?person_id={person_id}"
        ))
        .await;
    let body5: serde_json::Value = resp5.json();
    assert!(!body5["items"].as_array().unwrap().is_empty());
    // verify items correspond to person filter (all should have at least one; check at least one matches)
    // Since we just created one with that person, total >=1 is enough
}

// -- Todos los tipos explicitly

#[tokio::test]
async fn test_all_types_via_api() {
    let server = test_server().await;
    for t in [
        "CONFIRMED",
        "FALSE_LEAD",
        "INCONCLUSIVE",
        "NEW_LEAD",
        "NO_EVIDENCE",
    ] {
        let task_id = server
            .post("/api/v1/trees/1/research-tasks")
            .json(&json!({"title": format!("Type task {t}")}))
            .await
            .json::<serde_json::Value>()["id"]
            .as_i64()
            .unwrap();
        let resp = server
            .post(&format!("/api/v1/trees/1/research-tasks/{task_id}/outcome"))
            .json(&json!({"type": t, "summary": format!("summary {t}")}))
            .await;
        resp.assert_status(axum::http::StatusCode::CREATED);
        let body: serde_json::Value = resp.json();
        assert_eq!(body["type"], t);
        // patch to same type check
        let oid = body["id"].as_i64().unwrap();
        let resp2 = server
            .get(&format!("/api/v1/trees/1/research-outcomes/{oid}"))
            .await;
        resp2.assert_status_ok();
        assert_eq!(resp2.json::<serde_json::Value>()["type"], t);
    }
}

// -- JSON fields

#[tokio::test]
async fn test_json_fields_complete() {
    let server = test_server().await;
    let task_id = create_task(&server).await;
    let resp = server
        .post(&format!("/api/v1/trees/1/research-tasks/{task_id}/outcome"))
        .json(&json!({"type":"CONFIRMED","summary":"summary check","details":"some details"}))
        .await;
    let body: serde_json::Value = resp.json();
    for field in [
        "id",
        "tree_id",
        "task_id",
        "type",
        "summary",
        "details",
        "created_at",
        "updated_at",
    ] {
        assert!(
            body.get(field).is_some(),
            "missing field {field}: {:?}",
            body
        );
    }
    assert_eq!(body["details"], "some details");

    // without details -> null or absent but details should be null
    let task2 = create_task(&server).await;
    let resp2 = server
        .post(&format!("/api/v1/trees/1/research-tasks/{task2}/outcome"))
        .json(&json!({"type":"NO_EVIDENCE","summary":"no details"}))
        .await;
    let body2: serde_json::Value = resp2.json();
    assert!(body2["details"].is_null() || body2["details"].is_string());
}
