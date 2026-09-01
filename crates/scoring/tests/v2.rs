use neogenealogy_analyzer::analyze;
use neogenealogy_gedcom::{GedcomParser, LegacyGedcomParser};
use neogenealogy_scoring::opportunities;

#[test]
fn score_limits_0_100() {
    let tree = LegacyGedcomParser
        .parse("0 @I1@ INDI\n1 NAME Ana /Pérez/\n1 BIRT\n2 DATE 1800\n2 PLAC Madrid\n0 @I2@ INDI\n1 NAME Luis /García/\n1 BIRT\n2 DATE ABT 1750\n2 PLAC Alcalá\n")
        .unwrap();
    let f = analyze(&tree);
    let opps = opportunities(&tree, &f);
    for o in opps {
        assert!(o.score <= 100, "score >100: {}", o.score);
        assert!(o.breakdown.total == o.score);
        let sum: i32 = o.breakdown.components.iter().map(|c| c.points).sum();
        let clamped = sum.clamp(0, 100) as u8;
        assert_eq!(o.breakdown.total, clamped, "breakdown sum mismatch");
        assert!((0.1..=0.98).contains(&o.confidence));
    }
}

#[test]
fn score_min_and_max() {
    // Person with everything missing but no parents -> high score due to missing parent + researchability
    let tree = LegacyGedcomParser
        .parse("0 @I1@ INDI\n1 NAME X /Y/\n")
        .unwrap();
    let opps = opportunities(&tree, &analyze(&tree));
    assert!(!opps.is_empty());
    let score = opps[0].score;
    assert!(score >= 35, "expected at least medium, got {score}");

    // Person with full data and citation should not exceed 100
    let ged = "0 @S1@ SOUR\n1 TITL Book\n1 PUBL Page 1\n0 @I1@ INDI\n1 NAME Ana /Pérez/\n1 BIRT\n2 DATE 12 JAN 1800\n2 PLAC Madrid\n1 FAMC @F1@\n1 SOUR @S1@\n0 @F1@ FAM\n1 HUSB @I2@\n1 WIFE @I3@\n1 CHIL @I1@\n0 @I2@ INDI\n1 NAME A /B/\n0 @I3@ INDI\n1 NAME C /D/\n";
    let tree = LegacyGedcomParser.parse(ged).unwrap();
    let opps = opportunities(&tree, &analyze(&tree));
    for o in opps {
        assert!(o.score <= 100);
    }
}

#[test]
fn breakdown_consistent_and_explainable() {
    let tree = LegacyGedcomParser
        .parse("0 @I1@ INDI\n1 NAME Juan /García/\n1 BIRT\n2 DATE ABT 1760\n2 PLAC Alcalá\n1 SOUR @S1@\n0 @S1@ SOUR\n1 TITL Generic\n")
        .unwrap();
    let opps = opportunities(&tree, &analyze(&tree));
    assert_eq!(opps.len(), 1);
    let opp = &opps[0];
    assert!(!opp.breakdown.components.is_empty());
    assert!(opp
        .breakdown
        .components
        .iter()
        .any(|c| c.name.contains("Direct ancestor") || c.name.contains("researchability")));
    assert!(!opp.why_it_matters.is_empty());
    assert!(!opp.potential_sources.is_empty());
    assert!(matches!(
        opp.researchability,
        neogenealogy_core::Researchability::High
            | neogenealogy_core::Researchability::Medium
            | neogenealogy_core::Researchability::Low
    ));
}

#[test]
fn researchability_high_vs_low() {
    let high = LegacyGedcomParser
        .parse("0 @I1@ INDI\n1 NAME A /B/\n1 BIRT\n2 DATE 1800\n2 PLAC Madrid\n")
        .unwrap();
    let low = LegacyGedcomParser
        .parse("0 @I1@ INDI\n1 NAME A /B/\n")
        .unwrap();
    let opp_high = &opportunities(&high, &analyze(&high))[0];
    let opp_low = &opportunities(&low, &analyze(&low))[0];
    assert!(
        opp_high.score > opp_low.score,
        "high researchability should rank higher"
    );
    assert_eq!(
        opp_high.researchability,
        neogenealogy_core::Researchability::High
    );
    assert_eq!(
        opp_low.researchability,
        neogenealogy_core::Researchability::Low
    );
}
