use neogenealogy_core::*;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GedcomError {
    #[error("línea GEDCOM inválida en la línea {0}")]
    InvalidLine(usize),
    #[error("no se pudo leer el archivo: {0}")]
    Io(#[from] std::io::Error),
}
pub trait GedcomParser {
    fn parse(&self, input: &str) -> Result<GenealogyTree, GedcomError>;
}
#[derive(Default)]
pub struct LegacyGedcomParser;
#[derive(Clone)]
struct Line {
    level: u8,
    xref: Option<String>,
    tag: String,
    value: String,
}
fn lines(input: &str) -> Result<Vec<Line>, GedcomError> {
    input
        .lines()
        .enumerate()
        .filter(|(_, s)| !s.trim().is_empty())
        .map(|(n, s)| {
            let mut p = s.splitn(3, ' ');
            let level = p
                .next()
                .and_then(|x| x.parse().ok())
                .ok_or(GedcomError::InvalidLine(n + 1))?;
            let second = p.next().ok_or(GedcomError::InvalidLine(n + 1))?;
            let (xref, tag) = if second.starts_with('@') {
                (
                    Some(second.to_string()),
                    p.next().ok_or(GedcomError::InvalidLine(n + 1))?.to_string(),
                )
            } else {
                (None, second.to_string())
            };
            Ok(Line {
                level,
                xref,
                tag,
                value: p.next().unwrap_or("").trim().to_string(),
            })
        })
        .collect()
}
pub fn parse_date(raw: &str) -> DateValue {
    let raw = raw.trim().to_string();
    let parts: Vec<&str> = raw.split_whitespace().collect();
    let year_of = |value: &str| value.parse::<i32>().ok();
    let year = parts
        .iter()
        .find_map(|p| if p.len() == 4 { year_of(p) } else { None });
    let precision = match parts.first().copied() {
        Some("ABT") => DatePrecision::About,
        Some("BEF") => DatePrecision::Before,
        Some("AFT") => DatePrecision::After,
        Some("BET") => DatePrecision::Between,
        Some("FROM") => DatePrecision::FromTo,
        _ if parts.len() == 1 && year.is_some() => DatePrecision::Year,
        _ if parts.len() >= 2 && year.is_some() => DatePrecision::Exact,
        _ => DatePrecision::Unknown,
    };
    let start_year = parts.iter().find_map(|p| year_of(p));
    let end_year = match precision {
        DatePrecision::Between | DatePrecision::FromTo => {
            parts.iter().rev().find_map(|p| year_of(p))
        }
        _ => None,
    };
    let date = if precision == DatePrecision::Exact {
        chrono::NaiveDate::parse_from_str(&raw, "%d %b %Y").ok()
    } else {
        None
    };
    DateValue {
        raw,
        year,
        date,
        approximate: !matches!(
            precision,
            DatePrecision::Exact | DatePrecision::Year | DatePrecision::Unknown
        ),
        precision,
        range: Some(DateRange {
            start_year,
            end_year,
        }),
    }
}
impl GedcomParser for LegacyGedcomParser {
    fn parse(&self, input: &str) -> Result<GenealogyTree, GedcomError> {
        let ls = lines(input)?;
        let mut t = GenealogyTree::default();
        let mut i = 0;
        while i < ls.len() {
            let l = &ls[i];
            if l.level == 0 && l.tag == "INDI" {
                let mut p = Person {
                    gedcom_id: l.xref.clone().unwrap_or_default(),
                    id: t.persons.len() + 1,
                    ..Default::default()
                };
                let start = i;
                i += 1;
                while i < ls.len() && ls[i].level > 0 {
                    let x = &ls[i];
                    p.raw.push(RawTag {
                        level: x.level,
                        tag: x.tag.clone(),
                        value: x.value.clone(),
                    });
                    match x.tag.as_str() {
                        "NAME" => {
                            let v = x.value.trim_end_matches('/');
                            let parts: Vec<_> = v.split('/').collect();
                            p.name_original = Some(x.value.clone());
                            p.name_normalized = Some(normalize_text(v));
                            p.given_name = parts.first().unwrap_or(&"").trim().to_string();
                            p.surname = parts.get(1).unwrap_or(&"").trim().to_string();
                        }
                        "SEX" => p.sex = Some(x.value.clone()),
                        "NOTE" => p.notes.push(x.value.clone()),
                        "OCCU" => p.occupation = Some(x.value.clone()),
                        "FAMS" => p.family_spouse.push(x.value.clone()),
                        "FAMC" => p.family_child = Some(x.value.clone()),
                        "SOUR" => p.sources.push(x.value.clone()),
                        "BIRT" => {
                            if i + 1 < ls.len() && ls[i + 1].tag == "DATE" {
                                i += 1;
                                p.birth_date = Some(parse_date(&ls[i].value));
                            }
                            if i + 1 < ls.len() && ls[i + 1].tag == "PLAC" {
                                i += 1;
                                p.birth_place = Some(ls[i].value.clone());
                            }
                        }
                        "DEAT" => {
                            if i + 1 < ls.len() && ls[i + 1].tag == "DATE" {
                                i += 1;
                                p.death_date = Some(parse_date(&ls[i].value));
                            }
                            if i + 1 < ls.len() && ls[i + 1].tag == "PLAC" {
                                i += 1;
                                p.death_place = Some(ls[i].value.clone());
                            }
                        }
                        _ => {}
                    }
                    i += 1;
                }
                let _ = start;
                t.persons.push(p);
                continue;
            }
            if l.level == 0 && l.tag == "FAM" {
                let mut f = Family {
                    gedcom_id: l.xref.clone().unwrap_or_default(),
                    id: t.families.len() + 1,
                    ..Default::default()
                };
                i += 1;
                while i < ls.len() && ls[i].level > 0 {
                    let x = &ls[i];
                    f.raw.push(RawTag {
                        level: x.level,
                        tag: x.tag.clone(),
                        value: x.value.clone(),
                    });
                    match x.tag.as_str() {
                        "HUSB" => f.husband_id = Some(x.value.clone()),
                        "WIFE" => f.wife_id = Some(x.value.clone()),
                        "CHIL" => f.children.push(x.value.clone()),
                        "SOUR" => f.sources.push(x.value.clone()),
                        "MARR" => {
                            if i + 1 < ls.len() && ls[i + 1].tag == "DATE" {
                                i += 1;
                                f.marriage_date = Some(parse_date(&ls[i].value));
                            }
                            if i + 1 < ls.len() && ls[i + 1].tag == "PLAC" {
                                i += 1;
                                f.marriage_place = Some(ls[i].value.clone());
                            }
                        }
                        _ => {}
                    }
                    i += 1;
                }
                t.families.push(f);
                continue;
            }
            if l.level == 0 && l.tag == "SOUR" {
                t.sources.push(Source {
                    id: t.sources.len() + 1,
                    gedcom_id: l.xref.clone().unwrap_or_default(),
                    title: l.value.clone(),
                    ..Default::default()
                });
            }
            i += 1;
        }
        for f in &t.families {
            for child in &f.children {
                if let Some(p) = t.persons.iter_mut().find(|p| &p.gedcom_id == child) {
                    p.family_child = Some(f.gedcom_id.clone());
                }
            }
        }
        Ok(t)
    }
}
pub fn parse_file(path: &std::path::Path) -> Result<GenealogyTree, GedcomError> {
    let s = std::fs::read_to_string(path)?;
    LegacyGedcomParser.parse(&s)
}
