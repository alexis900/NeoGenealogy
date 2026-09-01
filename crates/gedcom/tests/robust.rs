use neogenealogy_gedcom::{GedcomParser, LegacyGedcomParser};

#[test]
fn parse_source_level_zero_with_details() {
    let ged = "0 @S1@ SOUR\n1 TITL Parish register\n1 AUTH Archivo\n1 PUBL Book 4 Page 127\n0 @I1@ INDI\n1 NAME A /B/\n1 SOUR @S1@\n";
    let tree = LegacyGedcomParser.parse(ged).unwrap();
    assert_eq!(tree.sources.len(), 1);
    assert_eq!(tree.sources[0].title, "Parish register");
    assert!(tree.sources[0].citation.is_some());
}

#[test]
fn whitespace_and_values_robust() {
    let ged = "0 @I1@ INDI\n1 NAME   Juan  /García/  \n1 BIRT\n2 DATE    ABT   1760  \n2 PLAC   Alcalá   la   Real  \n";
    let tree = LegacyGedcomParser.parse(ged).unwrap();
    assert_eq!(tree.persons[0].given_name, "Juan");
    assert_eq!(tree.persons[0].surname, "García");
    assert!(tree.persons[0].birth_place.is_some());
}

#[test]
fn broken_references_do_not_crash() {
    let ged = "0 @I1@ INDI\n1 NAME A /B/\n1 FAMC @F999@\n1 FAMS @F888@\n0 @F1@ FAM\n1 HUSB @I1@\n1 CHIL @I999@\n";
    let tree = LegacyGedcomParser.parse(ged).unwrap();
    assert_eq!(tree.persons.len(), 1);
    // ancestors should handle missing families gracefully
    let anc = tree.ancestors("@I1@");
    assert!(!anc.is_empty());
    let findings = neogenealogy_analyzer::analyze(&tree);
    assert!(!findings.is_empty());
}

#[test]
fn preserves_unknown_tags_and_dates_invalid() {
    let ged = "0 @I1@ INDI\n1 NAME A /B/\n1 _CUSTOM foo\n1 BIRT\n2 DATE INVALID\n2 PLAC X\n";
    let tree = LegacyGedcomParser.parse(ged).unwrap();
    assert!(tree.persons[0].raw.iter().any(|r| r.tag == "_CUSTOM"));
    assert!(tree.persons[0].birth_date.is_some());
    assert_eq!(
        tree.persons[0].birth_date.as_ref().unwrap().precision,
        neogenealogy_core::DatePrecision::Unknown
    );
}
