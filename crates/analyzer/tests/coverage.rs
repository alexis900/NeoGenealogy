use neogenealogy_analyzer::source_coverage;
use neogenealogy_gedcom::{GedcomParser, LegacyGedcomParser};

#[test]
fn coverage_zero() {
    let tree = LegacyGedcomParser
        .parse("0 @I1@ INDI\n1 NAME A /B/\n0 @I2@ INDI\n1 NAME C /D/\n")
        .unwrap();
    let sc = source_coverage(&tree);
    assert_eq!(sc.birth, 0.0);
    assert_eq!(sc.marriage, 0.0);
    assert_eq!(sc.death, 0.0);
    assert_eq!(sc.overall, 0.0);
}

#[test]
fn coverage_partial_and_full() {
    let ged = "0 @S1@ SOUR\n1 TITL T\n0 @I1@ INDI\n1 NAME A /B/\n1 BIRT\n2 DATE 1800\n2 PLAC X\n1 SOUR @S1@\n0 @F1@ FAM\n1 HUSB @I1@\n1 MARR\n2 DATE 1820\n2 PLAC Y\n0 @I2@ INDI\n1 NAME C /D/\n1 BIRT\n2 DATE 1802\n0 @I3@ INDI\n1 NAME E /F/\n1 DEAT\n2 DATE 1880\n";
    let tree = LegacyGedcomParser.parse(ged).unwrap();
    let sc = source_coverage(&tree);
    assert!(sc.birth > 0.0 && sc.birth < 100.0);
    assert!(sc.marriage > 0.0);
    assert!(sc.death > 0.0);
    assert!(sc.overall > 0.0 && sc.overall <= 100.0);
}

#[test]
fn sources_without_citation_vs_with() {
    let ged_no_cit = "0 @S1@ SOUR\n1 TITL Generic\n0 @I1@ INDI\n1 NAME A /B/\n1 SOUR @S1@\n";
    let ged_cit =
        "0 @S1@ SOUR\n1 TITL Book\n1 PUBL Page 127\n0 @I1@ INDI\n1 NAME A /B/\n1 SOUR @S1@\n";
    let tree_no = LegacyGedcomParser.parse(ged_no_cit).unwrap();
    let tree_cit = LegacyGedcomParser.parse(ged_cit).unwrap();
    assert!(tree_no.sources[0].citation.is_none());
    assert!(tree_cit.sources[0].citation.is_some());
    // coverage persons_with_source same, but confidence differs (tested in scoring)
    let sc_no = source_coverage(&tree_no);
    let sc_cit = source_coverage(&tree_cit);
    assert_eq!(sc_no.overall, sc_cit.overall); // coverage counts linkage, not citation quality
}

#[test]
fn events_without_sources() {
    let tree = LegacyGedcomParser
        .parse("0 @I1@ INDI\n1 NAME A /B/\n1 BIRT\n2 DATE 1800\n")
        .unwrap();
    let sc = source_coverage(&tree);
    assert_eq!(sc.other_events, 0.0);
    // add BAPM without source
    let tree2 = LegacyGedcomParser
        .parse("0 @I1@ INDI\n1 NAME A /B/\n1 BAPM\n2 DATE 1801\n2 PLAC X\n")
        .unwrap();
    let sc2 = source_coverage(&tree2);
    assert!(sc2.other_events > 0.0);
}
