use neogenealogy_storage::db::in_memory_pool;
use neogenealogy_storage::{import_gedcom_content, Storage};

async fn pool_with_tree() -> (Storage, i64) {
    let p = in_memory_pool().await.unwrap();
    import_gedcom_content(
        &p,
        "0 @I1@ INDI\n1 NAME A /A/\n0 @I2@ INDI\n1 NAME B /B/\n",
        "test.ged",
        None,
    )
    .await
    .unwrap();
    let s = Storage::new(p);
    let tree_id = s.list_trees(None, None).await.unwrap()[0].id;
    (s, tree_id)
}

#[tokio::test]
async fn test_task_filtering_by_status_and_ordering() {
    let (storage, tree_id) = pool_with_tree().await;
    // create tasks with different statuses and updated_at
    let t_open = storage
        .create_research_task(tree_id, None, None, "Open task", None)
        .await
        .unwrap();
    let t_prog = storage
        .create_research_task(tree_id, None, None, "In progress", None)
        .await
        .unwrap();
    storage
        .update_research_task(t_prog.id, None, None, Some("IN_PROGRESS"), None)
        .await
        .unwrap();
    let t_resolved = storage
        .create_research_task(tree_id, None, None, "Resolved", None)
        .await
        .unwrap();
    storage
        .update_research_task(t_resolved.id, None, None, Some("RESOLVED"), None)
        .await
        .unwrap();
    let t_rejected = storage
        .create_research_task(tree_id, None, None, "Rejected", None)
        .await
        .unwrap();
    storage
        .update_research_task(t_rejected.id, None, None, Some("REJECTED"), None)
        .await
        .unwrap();

    // filter by status
    let (open_items, total) = storage
        .list_research_tasks_filtered(tree_id, Some("OPEN"), None, None, None, 10, 0)
        .await
        .unwrap();
    assert_eq!(total, 1);
    assert_eq!(open_items[0].id, t_open.id);

    // ordering: IN_PROGRESS first, then OPEN, then rest by updated_at DESC
    let (all, _) = storage
        .list_research_tasks_filtered(tree_id, None, None, None, None, 10, 0)
        .await
        .unwrap();
    assert_eq!(all[0].status, "IN_PROGRESS");
    assert_eq!(all[1].status, "OPEN");
    // remaining should be RESOLVED/REJECTED after
    assert!(["RESOLVED", "REJECTED"].contains(&all[2].status.as_str()));
}

#[tokio::test]
async fn test_task_filtering_by_has_outcome() {
    let (storage, tree_id) = pool_with_tree().await;
    let t1 = storage
        .create_research_task(tree_id, None, None, "With outcome", None)
        .await
        .unwrap();
    let t2 = storage
        .create_research_task(tree_id, None, None, "Without", None)
        .await
        .unwrap();
    storage
        .create_research_outcome(tree_id, t1.id, "CONFIRMED", "summary", None)
        .await
        .unwrap();

    let (with_out, total_with) = storage
        .list_research_tasks_filtered(tree_id, None, None, None, Some(true), 10, 0)
        .await
        .unwrap();
    assert_eq!(total_with, 1);
    assert_eq!(with_out[0].id, t1.id);

    let (without, total_without) = storage
        .list_research_tasks_filtered(tree_id, None, None, None, Some(false), 10, 0)
        .await
        .unwrap();
    assert_eq!(total_without, 1);
    assert_eq!(without[0].id, t2.id);

    // combined filters: status OPEN + has_outcome false
    let t3 = storage
        .create_research_task(tree_id, None, None, "Open no outcome", None)
        .await
        .unwrap();
    let (combined, _) = storage
        .list_research_tasks_filtered(tree_id, Some("OPEN"), None, None, Some(false), 10, 0)
        .await
        .unwrap();
    assert!(combined.iter().any(|t| t.id == t2.id));
    assert!(combined.iter().any(|t| t.id == t3.id));
    assert!(!combined.iter().any(|t| t.id == t1.id));
}

#[tokio::test]
async fn test_task_pagination_and_tree_isolation() {
    let p = in_memory_pool().await.unwrap();
    import_gedcom_content(&p, "0 @I1@ INDI\n1 NAME X /X/\n", "tree1.ged", None)
        .await
        .unwrap();
    import_gedcom_content(&p, "0 @I1@ INDI\n1 NAME Y /Y/\n", "tree2.ged", None)
        .await
        .unwrap();
    let s = Storage::new(p);
    let trees = s.list_trees(None, None).await.unwrap();
    let t1 = trees.iter().find(|t| t.name == "tree1").unwrap().id;
    let t2 = trees.iter().find(|t| t.name == "tree2").unwrap().id;
    for i in 0..5 {
        s.create_research_task(t1, None, None, &format!("T {i}"), None)
            .await
            .unwrap();
    }
    let (page1, total) = s
        .list_research_tasks_filtered(t1, None, None, None, None, 2, 0)
        .await
        .unwrap();
    assert_eq!(page1.len(), 2);
    assert_eq!(total, 5);
    let (page2, _) = s
        .list_research_tasks_filtered(t1, None, None, None, None, 2, 2)
        .await
        .unwrap();
    assert_eq!(page2.len(), 2);
    assert_ne!(page1[0].id, page2[0].id);
    // tree isolation
    let (t2_items, total2) = s
        .list_research_tasks_filtered(t2, None, None, None, None, 10, 0)
        .await
        .unwrap();
    assert_eq!(total2, 0);
    assert!(t2_items.is_empty());
}

#[tokio::test]
async fn test_has_outcome_map_batch() {
    let (storage, tree_id) = pool_with_tree().await;
    let t1 = storage
        .create_research_task(tree_id, None, None, "T1", None)
        .await
        .unwrap();
    let t2 = storage
        .create_research_task(tree_id, None, None, "T2", None)
        .await
        .unwrap();
    storage
        .create_research_outcome(tree_id, t1.id, "NO_EVIDENCE", "s", None)
        .await
        .unwrap();
    let map = storage
        .get_tasks_has_outcome_map(&[t1.id, t2.id])
        .await
        .unwrap();
    assert!(map[&t1.id]);
    assert!(!map[&t2.id]);
}

#[tokio::test]
async fn test_outcome_history_ordering() {
    let (storage, tree_id) = pool_with_tree().await;
    // create tasks and outcomes sequentially
    let mut ids = vec![];
    for i in 0..3 {
        let t = storage
            .create_research_task(tree_id, None, None, &format!("H {i}"), None)
            .await
            .unwrap();
        let o = storage
            .create_research_outcome(tree_id, t.id, "CONFIRMED", &format!("sum {i}"), None)
            .await
            .unwrap();
        ids.push(o.id);
        // small delay to ensure distinct created_at? use updated_at increment? but created_at is now_iso with same second maybe same order by id
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }
    let (items, _) = storage
        .list_research_outcomes(tree_id, None, None, 10, 0)
        .await
        .unwrap();
    // should be ordered by created_at DESC -> last created first
    assert_eq!(items[0].id, ids[2]);
    assert_eq!(items[1].id, ids[1]);
    assert_eq!(items[2].id, ids[0]);
}

#[tokio::test]
async fn test_summary_counts() {
    let (storage, tree_id) = pool_with_tree().await;
    // opportunities: from import we have at least 1? Let's just check summary returns counts
    // create tasks with statuses
    let _t1 = storage
        .create_research_task(tree_id, None, None, "Open", None)
        .await
        .unwrap();
    let t2 = storage
        .create_research_task(tree_id, None, None, "Prog", None)
        .await
        .unwrap();
    storage
        .update_research_task(t2.id, None, None, Some("IN_PROGRESS"), None)
        .await
        .unwrap();
    let t3 = storage
        .create_research_task(tree_id, None, None, "Res", None)
        .await
        .unwrap();
    storage
        .update_research_task(t3.id, None, None, Some("RESOLVED"), None)
        .await
        .unwrap();
    storage
        .create_research_outcome(tree_id, t3.id, "CONFIRMED", "s", None)
        .await
        .unwrap();

    let summary = storage.research_summary(tree_id).await.unwrap();
    assert_eq!(summary["tasks"]["open"], 1);
    assert_eq!(summary["tasks"]["in_progress"], 1);
    assert_eq!(summary["tasks"]["resolved"], 1);
    assert_eq!(summary["outcomes"]["total"], 1);
    // opportunities high/medium/low should be numbers
    assert!(summary["opportunities"]["high"].is_number());
    assert!(summary["opportunities"]["medium"].is_number());
    assert!(summary["opportunities"]["low"].is_number());
    // tree isolation for summary
    let p = storage.pool.clone();
    import_gedcom_content(&p, "0 @I1@ INDI\n1 NAME Z /Z/\n", "empty.ged", None)
        .await
        .unwrap();
    let s2 = Storage::new(p);
    let trees = s2.list_trees(None, None).await.unwrap();
    let t_empty = trees.iter().find(|t| t.name == "empty").unwrap().id;
    let summary2 = s2.research_summary(t_empty).await.unwrap();
    assert_eq!(summary2["tasks"]["open"], 0);
    assert_eq!(summary2["outcomes"]["total"], 0);
}
