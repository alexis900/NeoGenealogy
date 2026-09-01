use neogenealogy_analyzer::analyze;
use neogenealogy_gedcom::{GedcomParser, LegacyGedcomParser};
use neogenealogy_scoring::opportunities;

#[test]
fn explains_score_and_orders_opportunity() {
    let tree = LegacyGedcomParser.parse("0 @I1@ INDI\n1 NAME Ana /Pérez/\n1 BIRT\n2 DATE 1800\n2 PLAC Madrid\n").unwrap();
    let findings = analyze(&tree);
    let items = opportunities(&tree, &findings);
    assert_eq!(items.len(), 1);
    assert!(items[0].score >= 50);
    assert!(!items[0].reasons.is_empty());
    assert!(items[0].suggested_sources.iter().any(|s| s == "matrimonios"));
}
