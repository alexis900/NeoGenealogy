use neogenealogy_storage::assessment::calculate_evidence_assessment;
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
async fn test_no_evidence() {
    let (s, tree) = setup().await;
    let task = s
        .create_research_task(tree, None, None, "Task", None)
        .await
        .unwrap();
    let outcome = s
        .create_research_outcome(tree, task.id, "CONFIRMED", "sum", None)
        .await
        .unwrap();
    let stats = s.get_outcome_evidence_stats(outcome.id).await.unwrap();
    assert_eq!(stats.evidence_total, 0);
    assert_eq!(stats.supporting_count, 0);
    let assessment = s.get_outcome_assessment(outcome.id).await.unwrap();
    assert_eq!(assessment.status, "NO_EVIDENCE");
    assert_eq!(assessment.score, 0);
    // via pure calc
    let calc = calculate_evidence_assessment(&stats);
    assert_eq!(calc.status, "NO_EVIDENCE");
}

#[tokio::test]
async fn test_single_supporting_weak() {
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
        .create_evidence(tree, src.id, None, "stmt", None)
        .await
        .unwrap();
    s.attach_evidence_to_outcome(outcome.id, ev.id, "SUPPORTS")
        .await
        .unwrap();
    let assessment = s.get_outcome_assessment(outcome.id).await.unwrap();
    assert_eq!(assessment.status, "WEAK");
    assert!(assessment.score >= 0 && assessment.score <= 100);
    assert_eq!(assessment.supporting_count, 1);
}

#[tokio::test]
async fn test_multiple_supporting_supported() {
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
    for _ in 0..2 {
        let ev = s
            .create_evidence(tree, src.id, None, "stmt", None)
            .await
            .unwrap();
        s.attach_evidence_to_outcome(outcome.id, ev.id, "SUPPORTS")
            .await
            .unwrap();
    }
    let a = s.get_outcome_assessment(outcome.id).await.unwrap();
    assert_eq!(a.status, "SUPPORTED");
    assert!(a.score > 0);
}

#[tokio::test]
async fn test_citation_additional_points() {
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
    let cit = s
        .create_research_citation(src.id, Some("folio"), None)
        .await
        .unwrap();
    let ev = s
        .create_evidence(tree, src.id, Some(cit.id), "stmt", None)
        .await
        .unwrap();
    s.attach_evidence_to_outcome(outcome.id, ev.id, "SUPPORTS")
        .await
        .unwrap();
    let a_with = s.get_outcome_assessment(outcome.id).await.unwrap();
    // without citation should be lower
    let task2 = s
        .create_research_task(tree, None, None, "Task2", None)
        .await
        .unwrap();
    let outcome2 = s
        .create_research_outcome(tree, task2.id, "CONFIRMED", "sum", None)
        .await
        .unwrap();
    let ev2 = s
        .create_evidence(tree, src.id, None, "stmt2", None)
        .await
        .unwrap();
    s.attach_evidence_to_outcome(outcome2.id, ev2.id, "SUPPORTS")
        .await
        .unwrap();
    let a_without = s.get_outcome_assessment(outcome2.id).await.unwrap();
    assert!(a_with.score > a_without.score);
    assert!(a_with.cited_count >= 1);
}

#[tokio::test]
async fn test_multiple_sources() {
    let (s, tree) = setup().await;
    let task = s
        .create_research_task(tree, None, None, "Task", None)
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
    let ev1 = s
        .create_evidence(tree, src1.id, None, "stmt1", None)
        .await
        .unwrap();
    let ev2 = s
        .create_evidence(tree, src2.id, None, "stmt2", None)
        .await
        .unwrap();
    s.attach_evidence_to_outcome(outcome.id, ev1.id, "SUPPORTS")
        .await
        .unwrap();
    s.attach_evidence_to_outcome(outcome.id, ev2.id, "SUPPORTS")
        .await
        .unwrap();
    let a = s.get_outcome_assessment(outcome.id).await.unwrap();
    assert_eq!(a.sources_count, 2);
    assert!(a.score >= 60); // should have multiple sources bonus
}

#[tokio::test]
async fn test_contradiction_mixed() {
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
    let ev1 = s
        .create_evidence(tree, src.id, None, "s1", None)
        .await
        .unwrap();
    let ev2 = s
        .create_evidence(tree, src.id, None, "s2", None)
        .await
        .unwrap();
    let ev3 = s
        .create_evidence(tree, src.id, None, "c1", None)
        .await
        .unwrap();
    s.attach_evidence_to_outcome(outcome.id, ev1.id, "SUPPORTS")
        .await
        .unwrap();
    s.attach_evidence_to_outcome(outcome.id, ev2.id, "SUPPORTS")
        .await
        .unwrap();
    s.attach_evidence_to_outcome(outcome.id, ev3.id, "CONTRADICTS")
        .await
        .unwrap();
    let a = s.get_outcome_assessment(outcome.id).await.unwrap();
    assert_eq!(a.status, "MIXED");
    assert_eq!(a.supporting_count, 2);
    assert_eq!(a.contradicting_count, 1);
}

#[tokio::test]
async fn test_contradiction_dominant_mixed() {
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
    let ev_s = s
        .create_evidence(tree, src.id, None, "s", None)
        .await
        .unwrap();
    let ev_c1 = s
        .create_evidence(tree, src.id, None, "c1", None)
        .await
        .unwrap();
    let ev_c2 = s
        .create_evidence(tree, src.id, None, "c2", None)
        .await
        .unwrap();
    s.attach_evidence_to_outcome(outcome.id, ev_s.id, "SUPPORTS")
        .await
        .unwrap();
    s.attach_evidence_to_outcome(outcome.id, ev_c1.id, "CONTRADICTS")
        .await
        .unwrap();
    s.attach_evidence_to_outcome(outcome.id, ev_c2.id, "CONTRADICTS")
        .await
        .unwrap();
    let a = s.get_outcome_assessment(outcome.id).await.unwrap();
    assert_eq!(a.status, "MIXED");
    assert!(a.score < 50); // penalty should reduce
}

#[tokio::test]
async fn test_score_bounds() {
    let (s, tree) = setup().await;
    // no evidence => 0
    let task = s
        .create_research_task(tree, None, None, "Task", None)
        .await
        .unwrap();
    let outcome = s
        .create_research_outcome(tree, task.id, "CONFIRMED", "sum", None)
        .await
        .unwrap();
    let a0 = s.get_outcome_assessment(outcome.id).await.unwrap();
    assert!(a0.score >= 0 && a0.score <= 100);
    // many supporting with citations and sources => should clamp to 100 max
    let src1 = s
        .create_research_source(tree, "Src1", None, None, None, "BOOK")
        .await
        .unwrap();
    let _src2 = s
        .create_research_source(tree, "Src2", None, None, None, "CENSUS")
        .await
        .unwrap();
    for _ in 0..5 {
        let cit = s
            .create_research_citation(src1.id, Some("loc"), None)
            .await
            .unwrap();
        let ev = s
            .create_evidence(tree, src1.id, Some(cit.id), "stmt", None)
            .await
            .unwrap();
        s.attach_evidence_to_outcome(outcome.id, ev.id, "SUPPORTS")
            .await
            .unwrap();
    }
    let a = s.get_outcome_assessment(outcome.id).await.unwrap();
    assert!(a.score <= 100 && a.score >= 0);
}

#[tokio::test]
async fn test_tree_isolation() {
    let pool = in_memory_pool().await.unwrap();
    neogenealogy_storage::import_gedcom_content(
        &pool,
        "0 @I1@ INDI\n1 NAME A /A/\n",
        "tree1.ged",
        None,
    )
    .await
    .unwrap();
    neogenealogy_storage::import_gedcom_content(
        &pool,
        "0 @I1@ INDI\n1 NAME B /B/\n",
        "tree2.ged",
        None,
    )
    .await
    .unwrap();
    let s = Storage::new(pool);
    let trees = s.list_trees(None, None).await.unwrap();
    let t1 = trees.iter().find(|t| t.name == "tree1").unwrap().id;
    let t2 = trees.iter().find(|t| t.name == "tree2").unwrap().id;
    let task1 = s
        .create_research_task(t1, None, None, "T1", None)
        .await
        .unwrap();
    let outcome1 = s
        .create_research_outcome(t1, task1.id, "CONFIRMED", "sum", None)
        .await
        .unwrap();
    let src1 = s
        .create_research_source(t1, "Src1", None, None, None, "BOOK")
        .await
        .unwrap();
    let ev1 = s
        .create_evidence(t1, src1.id, None, "stmt", None)
        .await
        .unwrap();
    s.attach_evidence_to_outcome(outcome1.id, ev1.id, "SUPPORTS")
        .await
        .unwrap();
    // t2 should have no evidence, assessment NO_EVIDENCE
    let task2 = s
        .create_research_task(t2, None, None, "T2", None)
        .await
        .unwrap();
    let outcome2 = s
        .create_research_outcome(t2, task2.id, "CONFIRMED", "sum", None)
        .await
        .unwrap();
    let a1 = s.get_outcome_assessment(outcome1.id).await.unwrap();
    let a2 = s.get_outcome_assessment(outcome2.id).await.unwrap();
    assert_eq!(a1.supporting_count, 1);
    assert_eq!(a2.supporting_count, 0);
    assert_eq!(a2.status, "NO_EVIDENCE");
}

#[tokio::test]
async fn test_batch() {
    let (s, tree) = setup().await;
    let mut ids = Vec::new();
    for i in 0..3 {
        let task = s
            .create_research_task(tree, None, None, &format!("Task {i}"), None)
            .await
            .unwrap();
        let outcome = s
            .create_research_outcome(tree, task.id, "CONFIRMED", "sum", None)
            .await
            .unwrap();
        ids.push(outcome.id);
        if i % 2 == 0 {
            let src = s
                .create_research_source(tree, &format!("Src {i}"), None, None, None, "BOOK")
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
    let map = s.get_outcomes_assessments(&ids).await.unwrap();
    assert_eq!(map.len(), 3);
    // batch should equal single
    for id in &ids {
        let single = s.get_outcome_assessment(*id).await.unwrap();
        let batch = map.get(id).unwrap();
        assert_eq!(single.status, batch.status);
        assert_eq!(single.score, batch.score);
    }
}
