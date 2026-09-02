use axum_test::TestServer;
use neogenealogy_api::{create_router, state::AppState};
use neogenealogy_storage::{db::in_memory_pool, import_gedcom_content, Storage};
use serde_json::json;

async fn server_with_complex() -> TestServer {
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

async fn server_empty() -> TestServer {
    let pool = in_memory_pool().await.unwrap();
    // create tree with minimal gedcom (no persons? minimal)
    let minimal = "0 HEAD\n1 GEDC\n2 VERS 5.5.1\n0 TRLR\n";
    neogenealogy_storage::import_gedcom_content(
        &pool,
        minimal,
        "empty.ged",
        Some("Empty Tree".into()),
    )
    .await
    .unwrap();
    let storage = Storage::new(pool);
    let state = AppState::new(storage);
    let app = create_router(state);
    TestServer::new(app).unwrap()
}

#[tokio::test]
async fn test_plan_empty_tree() {
    let server = server_empty().await;
    let resp = server.get("/api/v1/trees/1/research/plan").await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["summary"]["total_candidates"].as_u64().unwrap(), 0);
    assert_eq!(body["recommended"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_plan_tree_isolation() {
    let pool = in_memory_pool().await.unwrap();
    let content =
        std::fs::read_to_string("/home/amartinper/NeoGenealogy/test-data/complex.ged").unwrap();
    import_gedcom_content(&pool, &content, "complex.ged", None)
        .await
        .unwrap();
    let minimal = "0 HEAD\n1 GEDC\n2 VERS 5.5.1\n0 TRLR\n";
    import_gedcom_content(&pool, minimal, "empty.ged", Some("Empty2".into()))
        .await
        .unwrap();
    // tree 1 has opportunities, tree 2 is empty
    let storage = Storage::new(pool);
    let state = AppState::new(storage);
    let app = create_router(state);
    let server = TestServer::new(app).unwrap();
    let r1: serde_json::Value = server.get("/api/v1/trees/1/research/plan").await.json();
    let r2: serde_json::Value = server.get("/api/v1/trees/2/research/plan").await.json();
    assert!(r1["summary"]["total_candidates"].as_u64().unwrap() > 0);
    assert_eq!(r2["summary"]["total_candidates"].as_u64().unwrap(), 0);
    // ensure tree 2 plan not leaking opportunities from tree1
    for item in r2["recommended"].as_array().unwrap() {
        assert_ne!(
            item["opportunity_id"].as_i64().unwrap(),
            r1["recommended"][0]["opportunity_id"]
                .as_i64()
                .unwrap_or(-1)
        );
    }
}

#[tokio::test]
async fn test_plan_active_task_penalization() {
    let server = server_with_complex().await;
    // get plan baseline
    let resp: serde_json::Value = server
        .get("/api/v1/trees/1/research/plan?limit=100")
        .await
        .json();
    let first_id = resp["recommended"][0]["opportunity_id"].as_i64().unwrap();
    // first should be no active task
    assert!(!resp["recommended"][0]["active_task"].as_bool().unwrap());
    // create task for that opportunity
    let task_resp: serde_json::Value = server
        .post(&format!(
            "/api/v1/trees/1/research-opportunities/{first_id}/tasks"
        ))
        .json(&json!({"title":"test active"}))
        .await
        .json();
    assert_eq!(task_resp["opportunity_id"].as_i64().unwrap(), first_id);
    let resp2: serde_json::Value = server
        .get("/api/v1/trees/1/research/plan?limit=100")
        .await
        .json();
    let item = resp2["recommended"]
        .as_array()
        .unwrap()
        .iter()
        .chain(resp2["deferred"].as_array().unwrap().iter())
        .find(|x| x["opportunity_id"].as_i64().unwrap() == first_id)
        .unwrap();
    assert!(item["active_task"].as_bool().unwrap());
    assert!(item["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["code"] == "ACTIVE_TASK"));
    // should be demoted vs previously – its planning_score should be lower than before
    let before_ps = resp["recommended"][0]["planning_score"].as_f64().unwrap();
    let after_ps = item["planning_score"].as_f64().unwrap();
    assert!(after_ps < before_ps);
}

#[tokio::test]
async fn test_plan_terminal_excluded() {
    let server = server_with_complex().await;
    let resp: serde_json::Value = server
        .get("/api/v1/trees/1/research/plan?limit=100")
        .await
        .json();
    let opp_id = resp["recommended"][0]["opportunity_id"].as_i64().unwrap();
    let task: serde_json::Value = server
        .post(&format!(
            "/api/v1/trees/1/research-opportunities/{opp_id}/tasks"
        ))
        .json(&json!({}))
        .await
        .json();
    let task_id = task["id"].as_i64().unwrap();
    server
        .patch(&format!("/api/v1/trees/1/research-tasks/{task_id}"))
        .json(&json!({"status":"RESOLVED"}))
        .await;
    let resp2: serde_json::Value = server
        .get("/api/v1/trees/1/research/plan?limit=100")
        .await
        .json();
    let all: Vec<i64> = resp2["recommended"]
        .as_array()
        .unwrap()
        .iter()
        .chain(resp2["deferred"].as_array().unwrap().iter())
        .map(|x| x["opportunity_id"].as_i64().unwrap())
        .collect();
    assert!(
        !all.contains(&opp_id),
        "RESOLVED opportunity should be excluded"
    );

    // REJECTED also excluded
    let opp2 = resp["recommended"][1]["opportunity_id"].as_i64().unwrap();
    let task2: serde_json::Value = server
        .post(&format!(
            "/api/v1/trees/1/research-opportunities/{opp2}/tasks"
        ))
        .json(&json!({}))
        .await
        .json();
    let tid2 = task2["id"].as_i64().unwrap();
    server
        .patch(&format!("/api/v1/trees/1/research-tasks/{tid2}"))
        .json(&json!({"status":"REJECTED"}))
        .await;
    let resp3: serde_json::Value = server
        .get("/api/v1/trees/1/research/plan?limit=100")
        .await
        .json();
    let all3: Vec<i64> = resp3["recommended"]
        .as_array()
        .unwrap()
        .iter()
        .chain(resp3["deferred"].as_array().unwrap().iter())
        .map(|x| x["opportunity_id"].as_i64().unwrap())
        .collect();
    assert!(!all3.contains(&opp2));
}

#[tokio::test]
async fn test_plan_inconclusive_visible_with_penalty() {
    let server = server_with_complex().await;
    let resp: serde_json::Value = server
        .get("/api/v1/trees/1/research/plan?limit=100")
        .await
        .json();
    let opp_id = resp["recommended"][0]["opportunity_id"].as_i64().unwrap();
    let before_ps = resp["recommended"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["opportunity_id"].as_i64().unwrap() == opp_id)
        .unwrap()["planning_score"]
        .as_f64()
        .unwrap();
    let task: serde_json::Value = server
        .post(&format!(
            "/api/v1/trees/1/research-opportunities/{opp_id}/tasks"
        ))
        .json(&json!({}))
        .await
        .json();
    let tid = task["id"].as_i64().unwrap();
    server
        .patch(&format!("/api/v1/trees/1/research-tasks/{tid}"))
        .json(&json!({"status":"INCONCLUSIVE"}))
        .await;
    let resp2: serde_json::Value = server
        .get("/api/v1/trees/1/research/plan?limit=100")
        .await
        .json();
    let item = resp2["recommended"]
        .as_array()
        .unwrap()
        .iter()
        .chain(resp2["deferred"].as_array().unwrap().iter())
        .find(|x| x["opportunity_id"].as_i64().unwrap() == opp_id)
        .expect("inconclusive should remain");
    assert!(item["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["code"] == "PREVIOUSLY_INCONCLUSIVE"));
    let after_ps = item["planning_score"].as_f64().unwrap();
    assert!(after_ps < before_ps);
}

#[tokio::test]
async fn test_plan_critical_gap_ranking() {
    let server = server_with_complex().await;
    // Create outcome with gaps for first opportunity to get critical gap
    let resp: serde_json::Value = server
        .get("/api/v1/trees/1/research/plan?limit=100")
        .await
        .json();
    let opp_id = resp["recommended"].as_array().unwrap().last().unwrap()["opportunity_id"]
        .as_i64()
        .unwrap();
    // create task + outcome without evidence => critical gap
    let task: serde_json::Value = server
        .post(&format!(
            "/api/v1/trees/1/research-opportunities/{opp_id}/tasks"
        ))
        .json(&json!({}))
        .await
        .json();
    let tid = task["id"].as_i64().unwrap();
    server
        .post(&format!("/api/v1/trees/1/research-tasks/{tid}/outcome"))
        .json(&json!({"type":"CONFIRMED","summary":"test critical"} ))
        .await;
    let resp2: serde_json::Value = server
        .get("/api/v1/trees/1/research/plan?limit=100")
        .await
        .json();
    let item = resp2["recommended"]
        .as_array()
        .unwrap()
        .iter()
        .chain(resp2["deferred"].as_array().unwrap().iter())
        .find(|x| x["opportunity_id"].as_i64().unwrap() == opp_id)
        .unwrap();
    // should have critical gap reason and high planning score due to gap
    assert!(item["reasons"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["code"] == "CRITICAL_EVIDENCE_GAP"));
}

#[tokio::test]
async fn test_plan_filters_and_limit() {
    let server = server_with_complex().await;
    let resp: serde_json::Value = server
        .get("/api/v1/trees/1/research/plan?limit=5")
        .await
        .json();
    assert_eq!(resp["recommended"].as_array().unwrap().len(), 5);
    assert_eq!(resp["summary"]["recommended_count"].as_u64().unwrap(), 5);
    // filter priority high
    let r2: serde_json::Value = server
        .get("/api/v1/trees/1/research/plan?priority=high&limit=100")
        .await
        .json();
    for item in r2["recommended"]
        .as_array()
        .unwrap()
        .iter()
        .chain(r2["deferred"].as_array().unwrap().iter())
    {
        assert_eq!(item["priority"].as_str().unwrap().to_lowercase(), "high");
    }
    // researchability filter
    let r3: serde_json::Value = server
        .get("/api/v1/trees/1/research/plan?researchability=high&limit=100")
        .await
        .json();
    for item in r3["recommended"]
        .as_array()
        .unwrap()
        .iter()
        .chain(r3["deferred"].as_array().unwrap().iter())
    {
        assert_eq!(
            item["researchability"].as_str().unwrap().to_lowercase(),
            "high"
        );
    }
    // min_score filter
    let r4: serde_json::Value = server
        .get("/api/v1/trees/1/research/plan?min_score=90&limit=100")
        .await
        .json();
    for item in r4["recommended"]
        .as_array()
        .unwrap()
        .iter()
        .chain(r4["deferred"].as_array().unwrap().iter())
    {
        assert!(item["planning_score"].as_f64().unwrap() >= 90.0);
    }
}

#[tokio::test]
async fn test_plan_determinism() {
    let server = server_with_complex().await;
    let r1: serde_json::Value = server
        .get("/api/v1/trees/1/research/plan?limit=20")
        .await
        .json();
    let r2: serde_json::Value = server
        .get("/api/v1/trees/1/research/plan?limit=20")
        .await
        .json();
    assert_eq!(r1["recommended"], r2["recommended"]);
    assert_eq!(r1["deferred"], r2["deferred"]);
}

#[tokio::test]
async fn test_plan_top10_default() {
    let server = server_with_complex().await;
    let r: serde_json::Value = server.get("/api/v1/trees/1/research/plan").await.json();
    assert_eq!(
        r["summary"]["recommended_count"].as_u64().unwrap(),
        r["recommended"].as_array().unwrap().len() as u64
    );
    // if total <=10 recommended = total
    assert!(
        r["summary"]["total_candidates"].as_u64().unwrap() as usize
            >= r["recommended"].as_array().unwrap().len()
    );
    if r["summary"]["total_candidates"].as_u64().unwrap() > 10 {
        assert_eq!(r["recommended"].as_array().unwrap().len(), 10);
    }
}
