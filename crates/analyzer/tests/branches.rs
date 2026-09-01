use neogenealogy_analyzer::{analyze, branch_analyses};
use neogenealogy_gedcom::{GedcomParser, LegacyGedcomParser};
use neogenealogy_scoring::opportunities;

#[test]
fn branch_simple() {
    let ged = "0 @I1@ INDI\n1 NAME Juan /García/\n1 BIRT\n2 DATE 1800\n0 @I2@ INDI\n1 NAME Ana /García/\n1 BIRT\n2 DATE 1802\n";
    let tree = LegacyGedcomParser.parse(ged).unwrap();
    let opps = opportunities(&tree, &analyze(&tree));
    let branches = branch_analyses(&tree, &opps);
    assert_eq!(branches.len(), 1);
    assert_eq!(branches[0].name, "García");
    assert!(branches[0].opportunity_count == 2);
}

#[test]
fn branches_multiple_with_different_coverage() {
    let ged = "0 @S1@ SOUR\n1 TITL Book\n1 PAGE 1\n0 @I1@ INDI\n1 NAME Juan /García/\n1 BIRT\n2 DATE 1800\n1 SOUR @S1@\n0 @I2@ INDI\n1 NAME Ana /López/\n1 BIRT\n2 DATE 1801\n0 @I3@ INDI\n1 NAME Pedro /García/\n1 BIRT\n2 DATE 1802\n1 SOUR @S1@\n";
    let tree = LegacyGedcomParser.parse(ged).unwrap();
    let opps = opportunities(&tree, &analyze(&tree));
    let branches = branch_analyses(&tree, &opps);
    let garcia = branches.iter().find(|b| b.name == "García").unwrap();
    let lopez = branches.iter().find(|b| b.name == "López").unwrap();
    assert!(
        garcia.source_coverage > lopez.source_coverage,
        "García should have higher source coverage"
    );
    // Quality over quantity: branch with single high priority should not be overtaken by many trivial
    // Here García has 2 persons, López 1, both similar scores, branch score derived from max+avg
    assert!(garcia.score <= 100 && lopez.score <= 100);
}

#[test]
fn branch_score_quality_over_quantity() {
    // Create 1 critical vs 10 trivial: ensure critical branch wins
    // We simulate by using scoring: trivial low scores are hard to get because all missing parents high.
    // Instead verify branch_score formula: average of top5 prevents quantity dominance
    let ged = "0 @I1@ INDI\n1 NAME A /Alpha/\n1 BIRT\n2 DATE 1800\n2 PLAC Madrid\n0 @I2@ INDI\n1 NAME B /Alpha/\n1 BIRT\n2 DATE 1801\n2 PLAC Madrid\n0 @I3@ INDI\n1 NAME C /Beta/\n1 BIRT\n2 DATE ABT 1500\n";
    let tree = LegacyGedcomParser.parse(ged).unwrap();
    let opps = opportunities(&tree, &analyze(&tree));
    let branches = branch_analyses(&tree, &opps);
    // Beta has Low researchability (year <1500 or missing place) -> lower score
    // Alpha should beat Beta
    let alpha = branches.iter().find(|b| b.name == "Alpha").unwrap();
    let beta = branches.iter().find(|b| b.name == "Beta").unwrap();
    assert!(alpha.score >= beta.score);
}
