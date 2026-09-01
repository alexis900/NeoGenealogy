use neogenealogy_core::DatePrecision;
use neogenealogy_gedcom::parse_date;

#[test]
fn keeps_partial_and_range_date_semantics() {
    let cases = [
        ("1 JAN 1900", DatePrecision::Exact, Some(1900), None),
        ("JAN 1900", DatePrecision::Exact, Some(1900), None),
        ("1900", DatePrecision::Year, Some(1900), None),
        ("ABT 1900", DatePrecision::About, Some(1900), None),
        ("BEF 1900", DatePrecision::Before, Some(1900), None),
        ("AFT 1900", DatePrecision::After, Some(1900), None),
        (
            "BET 1890 AND 1900",
            DatePrecision::Between,
            Some(1890),
            Some(1900),
        ),
        (
            "FROM 1890 TO 1900",
            DatePrecision::FromTo,
            Some(1890),
            Some(1900),
        ),
    ];
    for (raw, precision, year, end) in cases {
        let date = parse_date(raw);
        assert_eq!(date.precision, precision, "{raw}");
        assert_eq!(date.year, year, "{raw}");
        assert_eq!(date.range.unwrap().end_year, end, "{raw}");
        assert_eq!(date.raw, raw);
    }
}
