use neogenealogy_storage::db::in_memory_pool;
use neogenealogy_storage::Storage;

async fn setup() -> (Storage, i64) {
    let pool = in_memory_pool().await.unwrap();
    neogenealogy_storage::import_gedcom_content(
        &pool,
        "0 @I1@ INDI\n1 NAME A /A/\n",
        "test.ged",
        None,
    )
    .await
    .unwrap();
    let s = Storage::new(pool);
    let tree = s.list_trees(None, None).await.unwrap()[0].id;
    (s, tree)
}

#[tokio::test]
async fn test_gap_no_supporting() {
    let (s, tree) = setup().await;
    let task = s
        .create_research_task(tree, None, None, "T", None)
        .await
        .unwrap();
    let outcome = s
        .create_research_outcome(tree, task.id, "INCONCLUSIVE", "sum", None)
        .await
        .unwrap();
    let gaps = s.get_outcome_gaps(outcome.id).await.unwrap();
    assert!(gaps
        .iter()
        .any(|g| g.code == "NO_SUPPORTING_EVIDENCE" && g.severity == "CRITICAL"));
}

#[tokio::test]
async fn test_gap_confirmed_without_support() {
    let (s, tree) = setup().await;
    let task = s
        .create_research_task(tree, None, None, "T", None)
        .await
        .unwrap();
    let outcome = s
        .create_research_outcome(tree, task.id, "CONFIRMED", "sum", None)
        .await
        .unwrap();
    let gaps = s.get_outcome_gaps(outcome.id).await.unwrap();
    assert!(gaps.iter().any(|g| g.code == "CONFIRMED_WITHOUT_SUPPORT"));
    assert!(!gaps.iter().any(|g| g.code == "NO_SUPPORTING_EVIDENCE"));
}

#[tokio::test]
async fn test_gap_single_support() {
    let (s, tree) = setup().await;
    let task = s
        .create_research_task(tree, None, None, "T", None)
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
    let cit = s
        .create_research_citation(src.id, Some("loc"), None)
        .await
        .unwrap();
    let ev = s
        .create_evidence(tree, src.id, Some(cit.id), "stmt", None)
        .await
        .unwrap();
    s.attach_evidence_to_outcome(outcome.id, ev.id, "SUPPORTS")
        .await
        .unwrap();
    let gaps = s.get_outcome_gaps(outcome.id).await.unwrap();
    assert!(gaps.iter().any(|g| g.code == "SINGLE_SUPPORTING_EVIDENCE"));
    assert!(gaps.iter().any(|g| g.code == "SINGLE_SOURCE"));
    assert!(!gaps.iter().any(|g| g.code == "NO_CITATION"));
}

#[tokio::test]
async fn test_gap_no_citation() {
    let (s, tree) = setup().await;
    let task = s
        .create_research_task(tree, None, None, "T", None)
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
        .create_evidence(tree, src.id, None, "stmt", None)
        .await
        .unwrap();
    s.attach_evidence_to_outcome(outcome.id, ev.id, "SUPPORTS")
        .await
        .unwrap();
    let gaps = s.get_outcome_gaps(outcome.id).await.unwrap();
    assert!(gaps.iter().any(|g| g.code == "NO_CITATION"));
}

#[tokio::test]
async fn test_gap_contradiction() {
    let (s, tree) = setup().await;
    let task = s
        .create_research_task(tree, None, None, "T", None)
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
    let ev1 = s
        .create_evidence(tree, src.id, None, "s1", None)
        .await
        .unwrap();
    let ev2 = s
        .create_evidence(tree, src.id, None, "c1", None)
        .await
        .unwrap();
    s.attach_evidence_to_outcome(outcome.id, ev1.id, "SUPPORTS")
        .await
        .unwrap();
    s.attach_evidence_to_outcome(outcome.id, ev2.id, "CONTRADICTS")
        .await
        .unwrap();
    let gaps = s.get_outcome_gaps(outcome.id).await.unwrap();
    assert!(gaps.iter().any(|g| g.code == "CONTRADICTORY_EVIDENCE"));
}

#[tokio::test]
async fn test_gap_single_source_multiple() {
    let (s, tree) = setup().await;
    let task = s
        .create_research_task(tree, None, None, "T", None)
        .await
        .unwrap();
    let outcome = s
        .create_research_outcome(tree, task.id, "CONFIRMED", "sum", None)
        .await
        .unwrap();
    let src1 = s
        .create_research_source(tree, "Src1", None, None, None, "BOOK")
        .await
        .unwrap();
    let src2 = s
        .create_research_source(tree, "Src2", None, None, None, "CENSUS")
        .await
        .unwrap();
    let cit = s
        .create_research_citation(src1.id, Some("loc"), None)
        .await
        .unwrap();
    let ev1 = s
        .create_evidence(tree, src1.id, Some(cit.id), "s1", None)
        .await
        .unwrap();
    let ev2 = s
        .create_evidence(tree, src2.id, None, "s2", None)
        .await
        .unwrap();
    s.attach_evidence_to_outcome(outcome.id, ev1.id, "SUPPORTS")
        .await
        .unwrap();
    s.attach_evidence_to_outcome(outcome.id, ev2.id, "SUPPORTS")
        .await
        .unwrap();
    let gaps = s.get_outcome_gaps(outcome.id).await.unwrap();
    assert!(!gaps.iter().any(|g| g.code == "SINGLE_SOURCE"));
    assert!(gaps.is_empty());
}

#[tokio::test]
async fn test_gap_combined() {
    let (s, tree) = setup().await;
    let task = s
        .create_research_task(tree, None, None, "T", None)
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
        .create_evidence(tree, src.id, None, "stmt", None)
        .await
        .unwrap();
    s.attach_evidence_to_outcome(outcome.id, ev.id, "SUPPORTS")
        .await
        .unwrap();
    let gaps = s.get_outcome_gaps(outcome.id).await.unwrap();
    assert!(gaps.iter().any(|g| g.code == "SINGLE_SUPPORTING_EVIDENCE"));
    assert!(gaps.iter().any(|g| g.code == "NO_CITATION"));
    assert!(gaps.iter().any(|g| g.code == "SINGLE_SOURCE"));
    assert_eq!(gaps.len(), 3);
}

#[tokio::test]
async fn test_gap_batch() {
    let (s, tree) = setup().await;
    let mut ids = Vec::new();
    for i in 0..3 {
        let task = s
            .create_research_task(tree, None, None, &format!("T{i}"), None)
            .await
            .unwrap();
        let outcome = s
            .create_research_outcome(tree, task.id, "CONFIRMED", "sum", None)
            .await
            .unwrap();
        ids.push(outcome.id);
        if i == 1 {
            let src = s
                .create_research_source(tree, &format!("Src{i}"), None, None, None, "BOOK")
                .await
                .unwrap();
            let ev = s
                .create_evidence(tree, src.id, None, "stmt", None)
                .await
                .unwrap();
            s.attach_evidence_to_outcome(outcome.id, ev.id, "SUPPORTS")
                .await
                .unwrap();
        }
    }
    let map = s.get_outcomes_gaps(&ids).await.unwrap();
    assert_eq!(map.len(), 3);
    for id in &ids {
        let single = s.get_outcome_gaps(*id).await.unwrap();
        let batch = map.get(id).unwrap();
        assert_eq!(single.len(), batch.len());
        for g in single {
            assert!(batch.iter().any(|b| b.code == g.code));
        }
    }
}
