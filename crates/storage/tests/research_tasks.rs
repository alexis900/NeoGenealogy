use neogenealogy_storage::db::in_memory_pool;
use neogenealogy_storage::{import_gedcom_content, Storage};
use sqlx::SqlitePool;

async fn pool() -> SqlitePool {
    in_memory_pool().await.unwrap()
}

#[tokio::test]
async fn test_create_and_get() {
    let p = pool().await;
    let content = "0 @I1@ INDI\n1 NAME Juan /Garcia/\n0 @I2@ INDI\n1 NAME Maria /Lopez/\n";
    import_gedcom_content(&p, content, "test.ged", None)
        .await
        .unwrap();
    let storage = Storage::new(p);
    let trees = storage.list_trees(None, None).await.unwrap();
    let tree_id = trees[0].id;
    let persons = storage.list_persons(tree_id, None, None).await.unwrap();
    let person_id = persons[0].id;
    let task = storage
        .create_research_task(tree_id, None, Some(person_id), "Find parents", Some("desc"))
        .await
        .unwrap();
    assert_eq!(task.title, "Find parents");
    assert_eq!(task.status, "OPEN");
    let fetched = storage.get_research_task(task.id).await.unwrap().unwrap();
    assert_eq!(fetched.id, task.id);
}

#[tokio::test]
async fn test_list_with_filters_and_pagination() {
    let p = pool().await;
    import_gedcom_content(&p, "0 @I1@ INDI\n1 NAME A /A/\n", "test.ged", None)
        .await
        .unwrap();
    let storage = Storage::new(p);
    let tree_id = storage.list_trees(None, None).await.unwrap()[0].id;
    for i in 0..5 {
        storage
            .create_research_task(tree_id, None, None, &format!("Task {}", i), None)
            .await
            .unwrap();
    }
    let (items, total) = storage
        .list_research_tasks(tree_id, None, None, None, 2, 0)
        .await
        .unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(total, 5);
    let (items2, _) = storage
        .list_research_tasks(tree_id, None, None, None, 2, 2)
        .await
        .unwrap();
    assert_eq!(items2.len(), 2);
    assert_ne!(items[0].id, items2[0].id);
}

#[tokio::test]
async fn test_update_status_timestamps() {
    let p = pool().await;
    import_gedcom_content(&p, "0 @I1@ INDI\n1 NAME A /A/\n", "test.ged", None)
        .await
        .unwrap();
    let storage = Storage::new(p);
    let tree_id = storage.list_trees(None, None).await.unwrap()[0].id;
    let task = storage
        .create_research_task(tree_id, None, None, "T", None)
        .await
        .unwrap();
    assert_eq!(task.status, "OPEN");
    assert!(task.started_at.is_none());
    let updated = storage
        .update_research_task(task.id, None, None, Some("IN_PROGRESS"), None)
        .await
        .unwrap();
    assert_eq!(updated.status, "IN_PROGRESS");
    assert!(updated.started_at.is_some());
    let updated2 = storage
        .update_research_task(task.id, None, None, Some("RESOLVED"), Some("done"))
        .await
        .unwrap();
    assert_eq!(updated2.status, "RESOLVED");
    assert!(updated2.completed_at.is_some());
    assert_eq!(updated2.resolution.as_deref(), Some("done"));
}

#[tokio::test]
async fn test_isolation_by_tree() {
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
    storage
        .create_research_task(t1, None, None, "Task t1", None)
        .await
        .unwrap();
    let (items1, _) = storage
        .list_research_tasks(t1, None, None, None, 10, 0)
        .await
        .unwrap();
    let (items2, _) = storage
        .list_research_tasks(t2, None, None, None, 10, 0)
        .await
        .unwrap();
    assert_eq!(items1.len(), 1);
    assert_eq!(items2.len(), 0);
    // Try to create task in t2 with person from t1 should fail
    let persons_t1 = storage.list_persons(t1, None, None).await.unwrap();
    let pid = persons_t1[0].id;
    let res = storage
        .create_research_task(t2, None, Some(pid), "bad", None)
        .await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_duplicate_active_reuse() {
    let p = pool().await;
    let content = "0 @I1@ INDI\n1 NAME A /A/\n";
    import_gedcom_content(&p, content, "test.ged", None)
        .await
        .unwrap();
    let storage = Storage::new(p);
    let tree_id = storage.list_trees(None, None).await.unwrap()[0].id;
    // Need an opportunity
    let opp_id: i64 = sqlx::query_scalar("SELECT id FROM research_opportunities WHERE tree_id=?1")
        .bind(tree_id)
        .fetch_one(&storage.pool)
        .await
        .unwrap();
    let task1 = storage
        .create_research_task(tree_id, Some(opp_id), None, "Task opp", None)
        .await
        .unwrap();
    let task2 = storage
        .create_research_task(tree_id, Some(opp_id), None, "Task opp duplicate", None)
        .await
        .unwrap();
    assert_eq!(task1.id, task2.id); // reused
                                    // After resolving, should allow new
    storage
        .update_research_task(task1.id, None, None, Some("RESOLVED"), None)
        .await
        .unwrap();
    let task3 = storage
        .create_research_task(tree_id, Some(opp_id), None, "Task new after resolved", None)
        .await
        .unwrap();
    assert_ne!(task1.id, task3.id);
}

#[tokio::test]
async fn test_delete() {
    let p = pool().await;
    import_gedcom_content(&p, "0 @I1@ INDI\n1 NAME A /A/\n", "test.ged", None)
        .await
        .unwrap();
    let storage = Storage::new(p);
    let tree_id = storage.list_trees(None, None).await.unwrap()[0].id;
    let task = storage
        .create_research_task(tree_id, None, None, "To delete", None)
        .await
        .unwrap();
    storage.delete_research_task(task.id).await.unwrap();
    let fetched = storage.get_research_task(task.id).await.unwrap();
    assert!(fetched.is_none());
}
