use neogenealogy_storage::db::in_memory_pool;
use neogenealogy_storage::{import_gedcom_content, Storage};

async fn setup() -> (Storage, i64) {
    let pool = in_memory_pool().await.unwrap();
    import_gedcom_content(&pool, "0 @I1@ INDI\n1 NAME A /A/\n", "test.ged", None)
        .await
        .unwrap();
    let s = Storage::new(pool);
    let tree_id = s.list_trees(None, None).await.unwrap()[0].id;
    (s, tree_id)
}

async fn setup_two_trees() -> (Storage, i64, i64) {
    let pool = in_memory_pool().await.unwrap();
    import_gedcom_content(&pool, "0 @I1@ INDI\n1 NAME A /A/\n", "tree1.ged", None)
        .await
        .unwrap();
    import_gedcom_content(&pool, "0 @I1@ INDI\n1 NAME B /B/\n", "tree2.ged", None)
        .await
        .unwrap();
    let s = Storage::new(pool);
    let trees = s.list_trees(None, None).await.unwrap();
    let t1 = trees.iter().find(|t| t.name == "tree1").unwrap().id;
    let t2 = trees.iter().find(|t| t.name == "tree2").unwrap().id;
    (s, t1, t2)
}

// Sources
#[tokio::test]
async fn test_source_crud_validation() {
    let (s, tree) = setup().await;
    let src = s
        .create_research_source(
            tree,
            "Registro parroquial",
            Some("Author"),
            Some("Pub"),
            Some("1874"),
            "PARISH_RECORD",
        )
        .await
        .unwrap();
    assert_eq!(src.title, "Registro parroquial");
    assert_eq!(src.r#type, "PARISH_RECORD");
    let fetched = s.get_research_source(src.id).await.unwrap().unwrap();
    assert_eq!(fetched.id, src.id);
    let updated = s
        .update_research_source(src.id, Some("New Title"), None, None, None, None)
        .await
        .unwrap();
    assert_eq!(updated.title, "New Title");
    s.delete_research_source(src.id).await.unwrap();
    assert!(s.get_research_source(src.id).await.unwrap().is_none());

    // validation: empty title
    let res = s
        .create_research_source(tree, "   ", None, None, None, "BOOK")
        .await;
    assert!(res.is_err());
    // invalid type
    let res2 = s
        .create_research_source(tree, "Title", None, None, None, "INVALID")
        .await;
    assert!(res2.is_err());
}

#[tokio::test]
async fn test_source_tree_isolation_pagination_type_filter() {
    let (s, t1, t2) = setup_two_trees().await;
    s.create_research_source(t1, "Source High", None, None, None, "BOOK")
        .await
        .unwrap();
    s.create_research_source(t1, "Source Low", None, None, None, "CENSUS")
        .await
        .unwrap();
    s.create_research_source(t1, "Another Book", None, None, None, "BOOK")
        .await
        .unwrap();
    let (items, total) = s.list_research_sources(t1, None, 2, 0).await.unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(total, 3);
    let (books, total_books) = s
        .list_research_sources(t1, Some("BOOK"), 10, 0)
        .await
        .unwrap();
    assert_eq!(total_books, 2);
    assert!(books.iter().all(|r| r.r#type == "BOOK"));
    let (t2_items, total2) = s.list_research_sources(t2, None, 10, 0).await.unwrap();
    assert_eq!(total2, 0);
    assert!(t2_items.is_empty());
    // tree isolation: fetching source from other tree should be considered not in tree
    let src_t1 = s
        .create_research_source(t1, "Iso", None, None, None, "OTHER")
        .await
        .unwrap();
    // list for t2 should not contain it
    let (t2_again, _) = s.list_research_sources(t2, None, 10, 0).await.unwrap();
    assert!(!t2_again.iter().any(|r| r.id == src_t1.id));
}

#[tokio::test]
async fn test_citation_crud_and_cascade() {
    let (s, tree) = setup().await;
    let src = s
        .create_research_source(tree, "Src", None, None, None, "BOOK")
        .await
        .unwrap();
    let cit = s
        .create_research_citation(src.id, Some("Libro III folio 42"), Some("Partida"))
        .await
        .unwrap();
    assert_eq!(cit.locator.as_deref(), Some("Libro III folio 42"));
    let fetched = s.get_research_citation(cit.id).await.unwrap().unwrap();
    assert_eq!(fetched.id, cit.id);
    let (list, total) = s.list_research_citations(src.id, 10, 0).await.unwrap();
    assert_eq!(total, 1);
    assert_eq!(list[0].id, cit.id);
    let updated = s
        .update_research_citation(cit.id, Some("New loc"), None)
        .await
        .unwrap();
    assert_eq!(updated.locator.as_deref(), Some("New loc"));
    s.delete_research_citation(cit.id).await.unwrap();
    assert!(s.get_research_citation(cit.id).await.unwrap().is_none());

    // cascade: delete source should delete citation
    let src2 = s
        .create_research_source(tree, "Src2", None, None, None, "BOOK")
        .await
        .unwrap();
    let cit2 = s
        .create_research_citation(src2.id, Some("loc"), None)
        .await
        .unwrap();
    s.delete_research_source(src2.id).await.unwrap();
    assert!(s.get_research_citation(cit2.id).await.unwrap().is_none());
}

#[tokio::test]
async fn test_citation_tree_isolation() {
    let (s, t1, t2) = setup_two_trees().await;
    let src1 = s
        .create_research_source(t1, "Src1", None, None, None, "BOOK")
        .await
        .unwrap();
    let cit1 = s
        .create_research_citation(src1.id, Some("loc"), None)
        .await
        .unwrap();
    // citation should be accessible only via source's tree; we check source tree mismatch
    let src2 = s
        .create_research_source(t2, "Src2", None, None, None, "BOOK")
        .await
        .unwrap();
    // try to create citation for source in wrong tree? that's not tree isolation but source isolation - but we test that citation's source belongs to t1, not t2
    // Ensure that fetching citation via t2's context would be considered isolation in API layer; storage layer just stores, but we can test that citation's source tree is t1
    let fetched = s.get_research_citation(cit1.id).await.unwrap().unwrap();
    assert_eq!(fetched.source_id, src1.id);
    // ensure citation cannot be created for non-existent source
    let res = s.create_research_citation(99999, Some("x"), None).await;
    assert!(res.is_err());
    // source mismatch not directly tested here, but ensure t2's source not mixed
    let list_t1 = s.list_research_citations(src1.id, 10, 0).await.unwrap();
    assert_eq!(list_t1.1, 1);
    let list_t2 = s.list_research_citations(src2.id, 10, 0).await.unwrap();
    assert_eq!(list_t2.1, 0);
}

// Evidence
#[tokio::test]
async fn test_evidence_crud_validation() {
    let (s, tree) = setup().await;
    let src = s
        .create_research_source(tree, "Src", None, None, None, "REGISTER")
        .await
        .unwrap();
    let cit = s
        .create_research_citation(src.id, Some("folio 42"), None)
        .await
        .unwrap();
    let ev = s
        .create_evidence(tree, src.id, Some(cit.id), "Statement text", Some("notes"))
        .await
        .unwrap();
    assert_eq!(ev.statement, "Statement text");
    assert_eq!(ev.citation_id, Some(cit.id));
    let fetched = s.get_evidence(ev.id).await.unwrap().unwrap();
    assert_eq!(fetched.id, ev.id);
    // without citation
    let ev2 = s
        .create_evidence(tree, src.id, None, "Stmt no citation", None)
        .await
        .unwrap();
    assert!(ev2.citation_id.is_none());
    // list
    let (items, total) = s.list_evidence(tree, 10, 0).await.unwrap();
    assert_eq!(total, 2);
    assert_eq!(items.len(), 2);
    // update
    let upd = s
        .update_evidence(ev.id, Some("Updated statement"), Some("new notes"), None)
        .await
        .unwrap();
    assert_eq!(upd.statement, "Updated statement");
    // validation empty statement
    let res = s.create_evidence(tree, src.id, None, "   ", None).await;
    assert!(res.is_err());
    // delete
    s.delete_evidence(ev.id).await.unwrap();
    assert!(s.get_evidence(ev.id).await.unwrap().is_none());
}

#[tokio::test]
async fn test_evidence_tree_isolation_and_source_validation() {
    let (s, t1, t2) = setup_two_trees().await;
    let src1 = s
        .create_research_source(t1, "Src1", None, None, None, "BOOK")
        .await
        .unwrap();
    let src2 = s
        .create_research_source(t2, "Src2", None, None, None, "BOOK")
        .await
        .unwrap();
    // try to create evidence in t2 with src1 (cross-tree) should fail
    let res = s.create_evidence(t2, src1.id, None, "stmt", None).await;
    assert!(res.is_err());
    // citation must belong to source
    let cit2 = s
        .create_research_citation(src2.id, Some("loc"), None)
        .await
        .unwrap();
    let res2 = s
        .create_evidence(t1, src1.id, Some(cit2.id), "stmt", None)
        .await;
    assert!(res2.is_err());
    // list isolation
    s.create_evidence(t1, src1.id, None, "t1 evidence", None)
        .await
        .unwrap();
    let (t2_items, total2) = s.list_evidence(t2, 10, 0).await.unwrap();
    assert_eq!(total2, 0);
    assert!(t2_items.is_empty());
}

#[tokio::test]
async fn test_evidence_citation_set_null_on_delete() {
    let (s, tree) = setup().await;
    let src = s
        .create_research_source(tree, "Src", None, None, None, "BOOK")
        .await
        .unwrap();
    let cit = s
        .create_research_citation(src.id, Some("loc"), None)
        .await
        .unwrap();
    let ev = s
        .create_evidence(tree, src.id, Some(cit.id), "stmt", None)
        .await
        .unwrap();
    s.delete_research_citation(cit.id).await.unwrap();
    let fetched = s.get_evidence(ev.id).await.unwrap().unwrap();
    assert!(fetched.citation_id.is_none());
}

// OutcomeEvidence
#[tokio::test]
async fn test_outcome_evidence_attach_detach() {
    let (s, tree) = setup().await;
    // need outcome
    let task = s
        .create_research_task(tree, None, None, "Task", None)
        .await
        .unwrap();
    let outcome = s
        .create_research_outcome(tree, task.id, "CONFIRMED", "summary", None)
        .await
        .unwrap();
    let src = s
        .create_research_source(tree, "Src", None, None, None, "BOOK")
        .await
        .unwrap();
    let ev = s
        .create_evidence(tree, src.id, None, "Evidence statement", None)
        .await
        .unwrap();
    let link = s
        .attach_evidence_to_outcome(outcome.id, ev.id, "SUPPORTS")
        .await
        .unwrap();
    assert_eq!(link.relationship, "SUPPORTS");
    // list
    let list = s.list_outcome_evidence(outcome.id).await.unwrap();
    assert_eq!(list.len(), 1);
    // detailed
    let detailed = s.list_outcome_evidence_detailed(outcome.id).await.unwrap();
    assert_eq!(detailed.len(), 1);
    assert_eq!(detailed[0]["relationship"], "SUPPORTS");
    assert_eq!(detailed[0]["statement"], "Evidence statement");
    // duplicate should fail
    let dup = s
        .attach_evidence_to_outcome(outcome.id, ev.id, "SUPPORTS")
        .await;
    assert!(dup.is_err());
    // invalid relationship
    let ev2 = s
        .create_evidence(tree, src.id, None, "Other", None)
        .await
        .unwrap();
    let inv = s
        .attach_evidence_to_outcome(outcome.id, ev2.id, "INVALID")
        .await;
    assert!(inv.is_err());
    // contradicts
    let link2 = s
        .attach_evidence_to_outcome(outcome.id, ev2.id, "CONTRADICTS")
        .await
        .unwrap();
    assert_eq!(link2.relationship, "CONTRADICTS");
    let list2 = s.list_outcome_evidence(outcome.id).await.unwrap();
    assert_eq!(list2.len(), 2);
    // detach
    s.detach_evidence_from_outcome(outcome.id, ev.id)
        .await
        .unwrap();
    let list3 = s.list_outcome_evidence(outcome.id).await.unwrap();
    assert_eq!(list3.len(), 1);
    assert_eq!(list3[0].evidence_id, ev2.id);
}

#[tokio::test]
async fn test_outcome_evidence_cross_tree_rejection() {
    let (s, t1, t2) = setup_two_trees().await;
    let task1 = s
        .create_research_task(t1, None, None, "Task1", None)
        .await
        .unwrap();
    let outcome1 = s
        .create_research_outcome(t1, task1.id, "CONFIRMED", "sum", None)
        .await
        .unwrap();
    let src2 = s
        .create_research_source(t2, "Src2", None, None, None, "BOOK")
        .await
        .unwrap();
    let ev2 = s
        .create_evidence(t2, src2.id, None, "stmt", None)
        .await
        .unwrap();
    let res = s
        .attach_evidence_to_outcome(outcome1.id, ev2.id, "SUPPORTS")
        .await;
    assert!(res.is_err());
}

#[tokio::test]
async fn test_outcome_cascade_and_evidence_reuse() {
    let (s, tree) = setup().await;
    let task = s
        .create_research_task(tree, None, None, "Task", None)
        .await
        .unwrap();
    let outcome = s
        .create_research_outcome(tree, task.id, "CONFIRMED", "sum", None)
        .await
        .unwrap();
    let src = s
        .create_research_source(tree, "Src", None, None, None, "BOOK")
        .await
        .unwrap();
    let ev = s
        .create_evidence(tree, src.id, None, "reuse", None)
        .await
        .unwrap();
    s.attach_evidence_to_outcome(outcome.id, ev.id, "SUPPORTS")
        .await
        .unwrap();
    // delete outcome -> link disappears, evidence remains
    s.delete_research_outcome(outcome.id).await.unwrap();
    assert!(s.get_evidence(ev.id).await.unwrap().is_some());
    let links = s.list_outcome_evidence(outcome.id).await.unwrap();
    assert!(links.is_empty());
    // delete evidence -> link should be gone (tested via cascade)
    let task2 = s
        .create_research_task(tree, None, None, "Task2", None)
        .await
        .unwrap();
    let outcome2 = s
        .create_research_outcome(tree, task2.id, "CONFIRMED", "sum2", None)
        .await
        .unwrap();
    let ev2 = s
        .create_evidence(tree, src.id, None, "ev2", None)
        .await
        .unwrap();
    s.attach_evidence_to_outcome(outcome2.id, ev2.id, "SUPPORTS")
        .await
        .unwrap();
    s.delete_evidence(ev2.id).await.unwrap();
    let links2 = s.list_outcome_evidence(outcome2.id).await.unwrap();
    assert!(links2.is_empty());
    // outcome without evidence detailed should be empty
    let detailed = s.list_outcome_evidence_detailed(outcome2.id).await.unwrap();
    assert!(detailed.is_empty());
}
