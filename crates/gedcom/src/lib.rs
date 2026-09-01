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
            let tokens: Vec<&str> = s.split_whitespace().collect();
            let level = tokens
                .first()
                .copied()
                .and_then(|x| x.parse().ok())
                .ok_or(GedcomError::InvalidLine(n + 1))?;
            let second = tokens.get(1).ok_or(GedcomError::InvalidLine(n + 1))?;
            let (xref, tag, value_start) = if second.starts_with('@') {
                (
                    Some((*second).to_string()),
                    tokens
                        .get(2)
                        .ok_or(GedcomError::InvalidLine(n + 1))?
                        .to_string(),
                    3,
                )
            } else {
                (None, (*second).to_string(), 2)
            };
            Ok(Line {
                level,
                xref,
                tag,
                value: tokens.get(value_start..).unwrap_or(&[]).join(" "),
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

const KNOWN_EVENT_TAGS: &[&str] = &[
    "BIRT", "DEAT", "BAPM", "CHR", "BURI", "MARR", "DIV", "RESI", "OCCU", "EDUC", "RETI", "EVEN",
    "BARM", "BASM", "BLES", "ADOP", "CENS", "GRAD", "EMIG", "IMMI", "NATU", "PROB", "WILL",
];

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
                        "SOUR" => {
                            p.sources.push(x.value.clone());
                            // capture citation sub-tags (PAGE, DATA, etc.) as raw already
                            // look ahead for PAGE/DATA at level+1
                            let mut j = i + 1;
                            while j < ls.len() && ls[j].level > x.level {
                                p.raw.push(RawTag {
                                    level: ls[j].level,
                                    tag: ls[j].tag.clone(),
                                    value: ls[j].value.clone(),
                                });
                                j += 1;
                            }
                        }
                        "BIRT" => {
                            let event_level = x.level;
                            let mut j = i + 1;
                            while j < ls.len() && ls[j].level > event_level {
                                let sub = &ls[j];
                                p.raw.push(RawTag {
                                    level: sub.level,
                                    tag: sub.tag.clone(),
                                    value: sub.value.clone(),
                                });
                                match sub.tag.as_str() {
                                    "DATE" => p.birth_date = Some(parse_date(&sub.value)),
                                    "PLAC" => p.birth_place = Some(sub.value.clone()),
                                    "SOUR" => p.sources.push(sub.value.clone()),
                                    _ => {}
                                }
                                j += 1;
                            }
                            // events for birth
                            let ev = Event {
                                id: t.events.len() + 1,
                                person_id: Some(p.gedcom_id.clone()),
                                event_type: "BIRT".into(),
                                date: p.birth_date.clone(),
                                place: p.birth_place.clone(),
                                ..Default::default()
                            };
                            // only push if had date or place; still useful for coverage?
                            if p.birth_date.is_some() || p.birth_place.is_some() {
                                t.events.push(ev);
                            }
                            i = j - 1;
                        }
                        "DEAT" => {
                            let event_level = x.level;
                            let mut j = i + 1;
                            while j < ls.len() && ls[j].level > event_level {
                                let sub = &ls[j];
                                p.raw.push(RawTag {
                                    level: sub.level,
                                    tag: sub.tag.clone(),
                                    value: sub.value.clone(),
                                });
                                match sub.tag.as_str() {
                                    "DATE" => p.death_date = Some(parse_date(&sub.value)),
                                    "PLAC" => p.death_place = Some(sub.value.clone()),
                                    "SOUR" => p.sources.push(sub.value.clone()),
                                    _ => {}
                                }
                                j += 1;
                            }
                            let ev = Event {
                                id: t.events.len() + 1,
                                person_id: Some(p.gedcom_id.clone()),
                                event_type: "DEAT".into(),
                                date: p.death_date.clone(),
                                place: p.death_place.clone(),
                                ..Default::default()
                            };
                            if p.death_date.is_some() || p.death_place.is_some() {
                                t.events.push(ev);
                            }
                            i = j - 1;
                        }
                        _ if KNOWN_EVENT_TAGS.contains(&x.tag.as_str()) => {
                            // generic event handling
                            let event_level = x.level;
                            let mut date: Option<DateValue> = None;
                            let mut place: Option<String> = None;
                            let mut j = i + 1;
                            while j < ls.len() && ls[j].level > event_level {
                                let sub = &ls[j];
                                p.raw.push(RawTag {
                                    level: sub.level,
                                    tag: sub.tag.clone(),
                                    value: sub.value.clone(),
                                });
                                match sub.tag.as_str() {
                                    "DATE" => date = Some(parse_date(&sub.value)),
                                    "PLAC" => place = Some(sub.value.clone()),
                                    "SOUR" => p.sources.push(sub.value.clone()),
                                    _ => {}
                                }
                                j += 1;
                            }
                            let ev = Event {
                                id: t.events.len() + 1,
                                person_id: Some(p.gedcom_id.clone()),
                                event_type: x.tag.clone(),
                                date,
                                place,
                                ..Default::default()
                            };
                            t.events.push(ev);
                            i = j - 1;
                        }
                        _ => {}
                    }
                    i += 1;
                }
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
                            let lvl = x.level;
                            let mut j = i + 1;
                            while j < ls.len() && ls[j].level > lvl {
                                let sub = &ls[j];
                                f.raw.push(RawTag {
                                    level: sub.level,
                                    tag: sub.tag.clone(),
                                    value: sub.value.clone(),
                                });
                                match sub.tag.as_str() {
                                    "DATE" => f.marriage_date = Some(parse_date(&sub.value)),
                                    "PLAC" => f.marriage_place = Some(sub.value.clone()),
                                    "SOUR" => f.sources.push(sub.value.clone()),
                                    _ => {}
                                }
                                j += 1;
                            }
                            // also push family marriage event
                            let ev = Event {
                                id: t.events.len() + 1,
                                family_id: Some(f.gedcom_id.clone()),
                                event_type: "MARR".into(),
                                date: f.marriage_date.clone(),
                                place: f.marriage_place.clone(),
                                ..Default::default()
                            };
                            if f.marriage_date.is_some() || f.marriage_place.is_some() {
                                t.events.push(ev);
                            }
                            i = j - 1;
                        }
                        _ => {}
                    }
                    i += 1;
                }
                t.families.push(f);
                continue;
            }
            if l.level == 0 && l.tag == "SOUR" {
                let mut s = Source {
                    id: t.sources.len() + 1,
                    gedcom_id: l.xref.clone().unwrap_or_default(),
                    title: l.value.clone(),
                    ..Default::default()
                };
                let lvl = l.level;
                let mut j = i + 1;
                while j < ls.len() && ls[j].level > lvl {
                    let sub = &ls[j];
                    match sub.tag.as_str() {
                        "TITL"
                            if (s.title.is_empty() || sub.value.len() > s.title.len()) => {
                                s.title = sub.value.clone();
                            }
                        "AUTH" => s.author = Some(sub.value.clone()),
                        "REPO" => s.repository = Some(sub.value.clone()),
                        "PUBL" => {
                            // treat PUBL as citation detail
                            s.citation = Some(match &s.citation {
                                Some(c) => format!("{c}; {}", sub.value),
                                None => sub.value.clone(),
                            });
                        }
                        "TEXT" => {
                            s.citation = Some(match &s.citation {
                                Some(c) => format!("{c} {}", sub.value),
                                None => sub.value.clone(),
                            });
                        }
                        "PAGE" => {
                            s.citation = Some(match &s.citation {
                                Some(c) => format!("{c} Page {}", sub.value),
                                None => format!("Page {}", sub.value),
                            });
                        }
                        "ABBR" | "DATA"
                            // keep as citation if no other
                            if s.citation.is_none() => {
                                s.citation = Some(sub.value.clone());
                            }
                        _ => {}
                    }
                    j += 1;
                }
                // fallback title if empty
                if s.title.is_empty() {
                    s.title = format!("Source {}", s.gedcom_id);
                }
                t.sources.push(s);
            }
            i += 1;
        }
        for f in &t.families {
            for child in &f.children {
                if let Some(p) = t.persons.iter_mut().find(|p| &p.gedcom_id == child) {
                    if p.family_child.is_none() {
                        p.family_child = Some(f.gedcom_id.clone());
                    }
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
