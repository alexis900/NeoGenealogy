use neogenealogy_gedcom::{GedcomParser, LegacyGedcomParser};

#[test]
fn generation_1() {
    let ged = "0 @I1@ INDI\n1 NAME Child /X/\n1 FAMC @F1@\n0 @I2@ INDI\n1 NAME Parent /X/\n1 FAMS @F1@\n0 @F1@ FAM\n1 HUSB @I2@\n1 CHIL @I1@\n";
    let tree = LegacyGedcomParser.parse(ged).unwrap();
    let anc = tree.ancestors("@I1@");
    assert!(anc.iter().any(|(d, id)| *d == 1 && id == "@I2@"));
    assert_eq!(tree.generation_distance("@I1@"), Some(1));
}

#[test]
fn deep_generations_and_multiple_paths() {
    let ged = "0 @I1@ INDI\n1 NAME A /A/\n1 FAMC @F1@\n0 @I2@ INDI\n1 NAME B /B/\n1 FAMC @F2@\n1 FAMS @F1@\n0 @I3@ INDI\n1 NAME C /C/\n1 FAMS @F1@\n0 @I4@ INDI\n1 NAME D /D/\n1 FAMC @F2@\n1 SEX F\n0 @F1@ FAM\n1 HUSB @I2@\n1 WIFE @I3@\n1 CHIL @I1@\n0 @F2@ FAM\n1 HUSB @I4@\n1 CHIL @I2@\n";
    let tree = LegacyGedcomParser.parse(ged).unwrap();
    // I1 -> I2 -> I4 (2 generations)
    assert_eq!(tree.generation_distance("@I1@"), Some(2));
    let paths = tree.ancestor_paths("@I1@");
    assert!(!paths.is_empty());
    // cycle safety: should not infinite loop
}

#[test]
fn cycle_does_not_break_analysis() {
    let ged = "0 @I1@ INDI\n1 NAME A /A/\n1 FAMC @F2@\n1 FAMS @F1@\n0 @I2@ INDI\n1 NAME B /B/\n1 FAMC @F1@\n1 FAMS @F2@\n0 @F1@ FAM\n1 HUSB @I1@\n1 CHIL @I2@\n0 @F2@ FAM\n1 HUSB @I2@\n1 CHIL @I1@\n";
    let tree = LegacyGedcomParser.parse(ged).unwrap();
    let anc = tree.ancestors("@I1@");
    // should terminate and contain cycle nodes
    assert!(anc.len() >= 2);
    let findings = neogenealogy_analyzer::analyze(&tree);
    assert!(findings
        .iter()
        .any(|f| f.severity == neogenealogy_core::Severity::Critical
            && f.kind == "RELATIONSHIP_ANOMALY"));
    assert!(findings.iter().any(|f| f.description.contains("Ciclo")));
}

#[test]
fn is_direct_ancestor() {
    let ged = "0 @I1@ INDI\n1 NAME Child /X/\n1 FAMC @F1@\n0 @I2@ INDI\n1 NAME Parent /X/\n0 @F1@ FAM\n1 HUSB @I2@\n1 CHIL @I1@\n";
    let tree = LegacyGedcomParser.parse(ged).unwrap();
    assert!(tree.is_direct_ancestor_of("@I2@", "@I1@"));
    assert!(!tree.is_direct_ancestor_of("@I1@", "@I2@"));
}
