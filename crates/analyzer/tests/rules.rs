use neogenealogy_analyzer::analyze;
use neogenealogy_core::Severity;
use neogenealogy_gedcom::{GedcomParser, LegacyGedcomParser};

#[test]
fn detects_missing_data_and_research_gap() {
    let tree = LegacyGedcomParser
        .parse("0 @I1@ INDI\n1 NAME Ana /Pérez/\n1 BIRT\n2 DATE 1800\n")
        .unwrap();
    let findings = analyze(&tree);
    assert!(findings.iter().any(|f| f.kind == "research-gap"));
    assert!(findings
        .iter()
        .any(|f| f.description.contains("lugar de nacimiento")));
    assert!(findings
        .iter()
        .any(|f| f.severity == Severity::Medium && f.kind == "missing-source"));
}

#[test]
fn chronology_is_anomaly_not_fact() {
    let tree = LegacyGedcomParser
        .parse("0 @I1@ INDI\n1 NAME Ana /Pérez/\n1 BIRT\n2 DATE 1900\n1 DEAT\n2 DATE 1800\n")
        .unwrap();
    let findings = analyze(&tree);
    let item = findings.iter().find(|f| f.kind == "chronology").unwrap();
    assert_eq!(item.severity, Severity::High);
    assert!(item.description.contains("Posible inconsistencia"));
}

#[test]
fn duplicate_detection_is_probabilistic() {
    let tree = neogenealogy_gedcom::LegacyGedcomParser
        .parse("0 @I1@ INDI\n1 NAME Juan /García/\n1 BIRT\n2 DATE 1872\n2 PLAC Madrid\n0 @I2@ INDI\n1 NAME Juan /García/\n1 BIRT\n2 DATE 1871\n2 PLAC Madrid\n")
        .unwrap();
    let finding = analyze(&tree).into_iter().find(|f| f.kind == "POSSIBLE_DUPLICATE").unwrap();
    assert!(finding.confidence < 1.0);
    assert!(finding.description.contains("No fusionar"));
}
