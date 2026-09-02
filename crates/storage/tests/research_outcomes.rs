use neogenealogy_storage::db::in_memory_pool;
use neogenealogy_storage::{import_gedcom_content, Storage};
use sqlx::SqlitePool;

async fn pool() -> SqlitePool {
    in_memory_pool().await.unwrap()
}

async fn setup_tree(storage: &Storage) -> i64 {
    storage.list_trees(None, None).await.unwrap()[0].id
}

async fn setup_tree_and_person(pool: &SqlitePool) -> (i64, i64) {
    let content = "0 @I1@ INDI\n1 NAME Juan /Garcia/\n0 @I2@ INDI\n1 NAME Maria /Lopez/\n";
    import_gedcom_content(pool, content, "test.ged", None)
        .await
        .unwrap();
    let storage = Storage::new(pool.clone());
    let tree_id = setup_tree(&storage).await;
    let persons = storage.list_persons(tree_id, None, None).await.unwrap();
    let person_id = persons[0].id;
    (tree_id, person_id)
}

// -- CRUD

#[tokio::test]
async fn test_create_and_get() {
    let p = pool().await;
    let (tree_id, person_id) = setup_tree_and_person(&p).await;
    let storage = Storage::new(p);
    let task = storage
        .create_research_task(tree_id, None, Some(person_id), "Find parents", None)
        .await
        .unwrap();
    let outcome = storage
        .create_research_outcome(
            tree_id,
            task.id,
            "CONFIRMED",
            "found record",
            Some("details"),
        )
        .await
        .unwrap();
    assert_eq!(outcome.task_id, task.id);
    assert_eq!(outcome.r#type, "CONFIRMED");
    assert_eq!(outcome.summary, "found record");
    assert_eq!(outcome.details.as_deref(), Some("details"));
    assert_eq!(outcome.tree_id, tree_id);

    let fetched = storage
        .get_research_outcome(outcome.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(fetched.id, outcome.id);
}

#[tokio::test]
async fn test_get_by_task() {
    let p = pool().await;
    let (tree_id, person_id) = setup_tree_and_person(&p).await;
    let storage = Storage::new(p);
    let task = storage
        .create_research_task(tree_id, None, Some(person_id), "Task1", None)
        .await
        .unwrap();
    storage
        .create_research_outcome(tree_id, task.id, "NEW_LEAD", "summary", None)
        .await
        .unwrap();
    let by_task = storage
        .get_research_outcome_by_task(task.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(by_task.task_id, task.id);
    assert_eq!(by_task.r#type, "NEW_LEAD");
}

#[tokio::test]
async fn test_update() {
    let p = pool().await;
    let (tree_id, _) = setup_tree_and_person(&p).await;
    let storage = Storage::new(p);
    let task = storage
        .create_research_task(tree_id, None, None, "Task upd", None)
        .await
        .unwrap();
    let outcome = storage
        .create_research_outcome(tree_id, task.id, "INCONCLUSIVE", "initial", None)
        .await
        .unwrap();
    let updated = storage
        .update_research_outcome(
            outcome.id,
            Some("CONFIRMED"),
            Some("updated summary"),
            Some("new details"),
        )
        .await
        .unwrap();
    assert_eq!(updated.r#type, "CONFIRMED");
    assert_eq!(updated.summary, "updated summary");
    assert_eq!(updated.details.as_deref(), Some("new details"));
}

#[tokio::test]
async fn test_delete() {
    let p = pool().await;
    let (tree_id, _) = setup_tree_and_person(&p).await;
    let storage = Storage::new(p);
    let task = storage
        .create_research_task(tree_id, None, None, "Task del", None)
        .await
        .unwrap();
    let outcome = storage
        .create_research_outcome(tree_id, task.id, "NO_EVIDENCE", "summary", None)
        .await
        .unwrap();
    storage.delete_research_outcome(outcome.id).await.unwrap();
    let fetched = storage.get_research_outcome(outcome.id).await.unwrap();
    assert!(fetched.is_none());
    let by_task = storage.get_research_outcome_by_task(task.id).await.unwrap();
    assert!(by_task.is_none());
}

// -- Types

#[tokio::test]
async fn test_all_five_types() {
    let valid = [
        "CONFIRMED",
        "FALSE_LEAD",
        "INCONCLUSIVE",
        "NEW_LEAD",
        "NO_EVIDENCE",
    ];
    for t in valid {
        let p = pool().await;
        let (tree_id, _) = setup_tree_and_person(&p).await;
        let storage = Storage::new(p);
        let task = storage
            .create_research_task(tree_id, None, None, &format!("Task {t}"), None)
            .await
            .unwrap();
        let outcome = storage
            .create_research_outcome(tree_id, task.id, t, &format!("summary {t}"), None)
            .await
            .unwrap();
        assert_eq!(outcome.r#type, t);
    }
}

// -- Validations

#[tokio::test]
async fn test_summary_empty() {
    let p = pool().await;
    let (tree_id, _) = setup_tree_and_person(&p).await;
    let storage = Storage::new(p);
    let task = storage
        .create_research_task(tree_id, None, None, "Task s", None)
        .await
        .unwrap();
    let res = storage
        .create_research_outcome(tree_id, task.id, "CONFIRMED", "   ", None)
        .await;
    assert!(res.is_err());
    assert!(res.unwrap_err().to_string().contains("summary"));

    // also update with empty summary should fail
    let task2 = storage
        .create_research_task(tree_id, None, None, "Task s2", None)
        .await
        .unwrap();
    let oc = storage
        .create_research_outcome(tree_id, task2.id, "CONFIRMED", "ok", None)
        .await
        .unwrap();
    let res2 = storage
        .update_research_outcome(oc.id, None, Some("  "), None)
        .await;
    assert!(res2.is_err());
}

#[tokio::test]
async fn test_invalid_type() {
    let p = pool().await;
    let (tree_id, _) = setup_tree_and_person(&p).await;
    let storage = Storage::new(p);
    let task = storage
        .create_research_task(tree_id, None, None, "Task type", None)
        .await
        .unwrap();
    let res = storage
        .create_research_outcome(tree_id, task.id, "INVALID", "summary", None)
        .await;
    assert!(res.is_err());
    assert!(res
        .unwrap_err()
        .to_string()
        .contains("invalid outcome type"));

    // update with invalid type
    let task2 = storage
        .create_research_task(tree_id, None, None, "Task type2", None)
        .await
        .unwrap();
    let oc = storage
        .create_research_outcome(tree_id, task2.id, "CONFIRMED", "ok", None)
        .await
        .unwrap();
    let res2 = storage
        .update_research_outcome(oc.id, Some("BAD"), None, None)
        .await;
    assert!(res2.is_err());
}

#[tokio::test]
async fn test_task_inexistente() {
    let p = pool().await;
    let (tree_id, _) = setup_tree_and_person(&p).await;
    let storage = Storage::new(p);
    let res = storage
        .create_research_outcome(tree_id, 99999, "CONFIRMED", "summary", None)
        .await;
    assert!(res.is_err());
    // NotFound error
    let msg = res.unwrap_err().to_string();
    assert!(msg.contains("not found") || msg.contains("NotFound") || msg.contains("task"));
}

#[tokio::test]
async fn test_outcome_inexistente() {
    let p = pool().await;
    let _ = setup_tree_and_person(&p).await;
    let storage = Storage::new(p);
    let fetched = storage.get_research_outcome(99999).await.unwrap();
    assert!(fetched.is_none());
    let by_task = storage.get_research_outcome_by_task(99999).await.unwrap();
    assert!(by_task.is_none());
    let upd = storage
        .update_research_outcome(99999, None, Some("x"), None)
        .await;
    assert!(upd.is_err());
}

// -- Unique constraint

#[tokio::test]
async fn test_duplicate_outcome_same_task() {
    let p = pool().await;
    let (tree_id, _) = setup_tree_and_person(&p).await;
    let storage = Storage::new(p);
    let task = storage
        .create_research_task(tree_id, None, None, "Task dup", None)
        .await
        .unwrap();
    storage
        .create_research_outcome(tree_id, task.id, "CONFIRMED", "first", None)
        .await
        .unwrap();
    let res = storage
        .create_research_outcome(tree_id, task.id, "FALSE_LEAD", "second", None)
        .await;
    assert!(res.is_err());
    let msg = res.unwrap_err().to_string();
    assert!(msg.contains("already exists"));
}

// -- Tree isolation

#[tokio::test]
async fn test_tree_isolation() {
    let p = pool().await;
    import_gedcom_content(&p, "0 @I1@ INDI\n1 NAME A /A/\n", "tree1.ged", None)
        .await
        .unwrap();
    import_gedcom_content(&p, "0 @I1@ INDI\n1 NAME B /B/\n", "tree2.ged", None)
        .await
        .unwrap();
    let storage = Storage::new(p);
    let trees = storage.list_trees(None, None).await.unwrap();
    let t1 = trees.iter().find(|t| t.name == "tree1").unwrap().id;
    let t2 = trees.iter().find(|t| t.name == "tree2").unwrap().id;

    let task1 = storage
        .create_research_task(t1, None, None, "Task t1", None)
        .await
        .unwrap();
    let outcome = storage
        .create_research_outcome(t1, task1.id, "CONFIRMED", "summary", None)
        .await
        .unwrap();

    // outcome tree_id should be t1
    assert_eq!(outcome.tree_id, t1);

    // listing outcomes for t2 should not contain outcome from t1
    let (items_t2, total_t2) = storage
        .list_research_outcomes(t2, None, None, 10, 0)
        .await
        .unwrap();
    assert_eq!(items_t2.len(), 0);
    assert_eq!(total_t2, 0);

    // creating outcome for task1 with tree_id=t2 should fail (cross-tree)
    let res = storage
        .create_research_outcome(t2, task1.id, "CONFIRMED", "cross", None)
        .await;
    assert!(res.is_err());

    // get still returns but belongs to t1; service layer should block cross-tree (API does).
    let fetched = storage
        .get_research_outcome(outcome.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(fetched.tree_id, t1);
    assert_ne!(fetched.tree_id, t2);
}

// -- Listado

#[tokio::test]
async fn test_list_pagination_and_filters() {
    let p = pool().await;
    let (tree_id, _) = setup_tree_and_person(&p).await;
    let storage = Storage::new(p);

    for i in 0..5 {
        let t = storage
            .create_research_task(tree_id, None, None, &format!("Task {i}"), None)
            .await
            .unwrap();
        let typ = if i % 2 == 0 {
            "CONFIRMED"
        } else {
            "FALSE_LEAD"
        };
        storage
            .create_research_outcome(tree_id, t.id, typ, &format!("summary {i}"), None)
            .await
            .unwrap();
    }

    let (items, total) = storage
        .list_research_outcomes(tree_id, None, None, 2, 0)
        .await
        .unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(total, 5);

    let (items2, _) = storage
        .list_research_outcomes(tree_id, None, None, 2, 2)
        .await
        .unwrap();
    assert_eq!(items2.len(), 2);

    // filter by type
    let (confirmed, total_c) = storage
        .list_research_outcomes(tree_id, Some("CONFIRMED"), None, 10, 0)
        .await
        .unwrap();
    assert_eq!(total_c, 3);
    assert!(confirmed.iter().all(|o| o.r#type == "CONFIRMED"));

    // filter by task
    let first_task_outcome_id = confirmed[0].task_id;
    let (by_task, total_t) = storage
        .list_research_outcomes(tree_id, None, Some(first_task_outcome_id), 10, 0)
        .await
        .unwrap();
    assert_eq!(total_t, 1);
    assert_eq!(by_task[0].task_id, first_task_outcome_id);
}

#[tokio::test]
async fn test_list_with_person() {
    let p = pool().await;
    let (tree_id, person_id) = setup_tree_and_person(&p).await;
    let storage = Storage::new(p);

    // task with person
    let task_with_person = storage
        .create_research_task(tree_id, None, Some(person_id), "Task with person", None)
        .await
        .unwrap();
    storage
        .create_research_outcome(tree_id, task_with_person.id, "NEW_LEAD", "lead", None)
        .await
        .unwrap();

    // task without person
    let task_no_person = storage
        .create_research_task(tree_id, None, None, "Task no person", None)
        .await
        .unwrap();
    storage
        .create_research_outcome(tree_id, task_no_person.id, "NO_EVIDENCE", "none", None)
        .await
        .unwrap();

    let (filtered, total) = storage
        .list_research_outcomes_with_person(tree_id, None, Some(person_id), 10, 0)
        .await
        .unwrap();
    assert_eq!(total, 1);
    assert_eq!(filtered[0].task_id, task_with_person.id);

    // type filter + person
    let (_filtered2, total2) = storage
        .list_research_outcomes_with_person(tree_id, Some("NEW_LEAD"), Some(person_id), 10, 0)
        .await
        .unwrap();
    assert_eq!(total2, 1);

    let (filtered3, total3) = storage
        .list_research_outcomes_with_person(tree_id, Some("NO_EVIDENCE"), Some(person_id), 10, 0)
        .await
        .unwrap();
    assert_eq!(total3, 0);
    assert_eq!(filtered3.len(), 0);
}

// -- Cascade

#[tokio::test]
async fn test_cascade_delete_task_removes_outcome() {
    let p = pool().await;
    let (tree_id, _) = setup_tree_and_person(&p).await;
    let storage = Storage::new(p);
    let task = storage
        .create_research_task(tree_id, None, None, "Cascade task", None)
        .await
        .unwrap();
    let outcome = storage
        .create_research_outcome(tree_id, task.id, "CONFIRMED", "to cascade", None)
        .await
        .unwrap();
    storage.delete_research_task(task.id).await.unwrap();
    let fetched = storage.get_research_outcome(outcome.id).await.unwrap();
    assert!(fetched.is_none());
    let by_task = storage.get_research_outcome_by_task(task.id).await.unwrap();
    assert!(by_task.is_none());
}

// -- Rollback / No partial persists on failure

#[tokio::test]
async fn test_rollback_on_duplicate_does_not_partially_persist() {
    let p = pool().await;
    let (tree_id, _) = setup_tree_and_person(&p).await;
    let storage = Storage::new(p);

    let task = storage
        .create_research_task(tree_id, None, None, "Rollback task", None)
        .await
        .unwrap();
    // first succeeds
    storage
        .create_research_outcome(tree_id, task.id, "CONFIRMED", "first", None)
        .await
        .unwrap();
    let count_before: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM research_outcomes WHERE tree_id=?1")
            .bind(tree_id)
            .fetch_one(&storage.pool)
            .await
            .unwrap();

    // second fails
    let res = storage
        .create_research_outcome(tree_id, task.id, "CONFIRMED", "second", None)
        .await;
    assert!(res.is_err());

    let count_after: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM research_outcomes WHERE tree_id=?1")
            .bind(tree_id)
            .fetch_one(&storage.pool)
            .await
            .unwrap();
    assert_eq!(count_before, count_after);

    // failed type validation also leaves no partial row
    let task2 = storage
        .create_research_task(tree_id, None, None, "Rollback task2", None)
        .await
        .unwrap();
    let res2 = storage
        .create_research_outcome(tree_id, task2.id, "BAD_TYPE", "summary", None)
        .await;
    assert!(res2.is_err());
    let by_task = storage
        .get_research_outcome_by_task(task2.id)
        .await
        .unwrap();
    assert!(by_task.is_none());
}
