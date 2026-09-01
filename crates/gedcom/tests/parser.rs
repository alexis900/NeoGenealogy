use neogenealogy_gedcom::{GedcomParser, LegacyGedcomParser};

#[test]
fn imports_people_families_and_relationships() {
    let input = "0 @I1@ INDI\n1 NAME Ana /Pérez/\n1 BIRT\n2 DATE 1800\n2 PLAC Madrid\n1 FAMS @F1@\n0 @I2@ INDI\n1 NAME Luis /Díaz/\n1 FAMC @F1@\n0 @F1@ FAM\n1 HUSB @I1@\n1 CHIL @I2@\n";
    let tree = LegacyGedcomParser.parse(input).unwrap();
    assert_eq!(tree.persons.len(), 2);
    assert_eq!(tree.families.len(), 1);
    assert_eq!(tree.persons[0].surname, "Pérez");
    assert_eq!(tree.persons[1].family_child.as_deref(), Some("@F1@"));
    assert_eq!(
        tree.persons[0].birth_date.as_ref().unwrap().year,
        Some(1800)
    );
}

#[test]
fn preserves_unknown_tags() {
    let tree = LegacyGedcomParser
        .parse("0 @I1@ INDI\n1 NAME Ana /Pérez/\n1 _CUSTOM preserved\n")
        .unwrap();
    assert!(tree.persons[0]
        .raw
        .iter()
        .any(|tag| tag.tag == "_CUSTOM" && tag.value == "preserved"));
}
