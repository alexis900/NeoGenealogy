use neogenealogy_core::*;

pub trait AnalysisRule {
    fn id(&self) -> &'static str;
    fn analyze(&self, tree: &GenealogyTree) -> Vec<Finding>;
}
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    pub minimum_parent_age: i32,
    pub unusual_parent_age: i32,
    pub maximum_parent_age: i32,
}
impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            minimum_parent_age: 14,
            unusual_parent_age: 55,
            maximum_parent_age: 75,
        }
    }
}
#[allow(clippy::too_many_arguments)]
fn finding(
    id: String,
    kind: &str,
    severity: Severity,
    person: Option<String>,
    related: Option<String>,
    description: String,
    evidence: Vec<String>,
    confidence: f32,
) -> Finding {
    Finding {
        id,
        kind: kind.into(),
        severity,
        person_id: person,
        related_person_id: related,
        description,
        evidence,
        confidence,
    }
}
pub struct MissingDataRule;
pub struct ChronologyRule {
    pub config: AnalysisConfig,
}
pub struct ResearchGapRule;
pub struct DuplicateRule;
pub struct SourceGapRule;
impl AnalysisRule for MissingDataRule {
    fn id(&self) -> &'static str {
        "missing-data"
    }
    fn analyze(&self, t: &GenealogyTree) -> Vec<Finding> {
        let mut o = Vec::new();
        for p in &t.persons {
            for (m, l) in [
                (p.birth_date.is_none(), "fecha de nacimiento"),
                (p.birth_place.is_none(), "lugar de nacimiento"),
                (p.death_date.is_none(), "fecha de defunción"),
                (p.death_place.is_none(), "lugar de defunción"),
            ] {
                if m {
                    o.push(finding(
                        format!("missing:{}:{}", p.gedcom_id, l),
                        "missing-data",
                        Severity::Info,
                        Some(p.gedcom_id.clone()),
                        None,
                        format!("Falta {} en {}.", l, p.given_name),
                        vec![],
                        0.99,
                    ));
                }
            }
        }
        for f in &t.families {
            if f.marriage_date.is_none() {
                o.push(finding(
                    format!("marriage-date:{}", f.gedcom_id),
                    "missing-data",
                    Severity::Info,
                    None,
                    None,
                    "Matrimonio sin fecha.".into(),
                    vec![],
                    0.99,
                ));
            }
            if f.marriage_place.is_none() {
                o.push(finding(
                    format!("marriage-place:{}", f.gedcom_id),
                    "missing-data",
                    Severity::Info,
                    None,
                    None,
                    "Matrimonio sin lugar.".into(),
                    vec![],
                    0.99,
                ));
            }
        }
        o
    }
}
impl AnalysisRule for ChronologyRule {
    fn id(&self) -> &'static str {
        "chronology"
    }
    fn analyze(&self, t: &GenealogyTree) -> Vec<Finding> {
        let mut o = Vec::new();
        for p in &t.persons {
            if let (Some(b), Some(d)) = (&p.birth_date, &p.death_date) {
                if let (Some(by), Some(dy)) = (b.year, d.year) {
                    if by > dy {
                        o.push(finding(format!("birth-after-death:{}",p.gedcom_id),"chronology",Severity::High,Some(p.gedcom_id.clone()),None,"Posible inconsistencia: nacimiento posterior a defunción; requiere verificación.".into(),vec![format!("birth year: {by}"),format!("death year: {dy}")],0.98));
                    }
                }
            }
        }
        for f in &t.families {
            let my = f.marriage_date.as_ref().and_then(|d| d.year);
            for c in &f.children {
                let cy = t
                    .person(c)
                    .and_then(|p| p.birth_date.as_ref())
                    .and_then(|d| d.year);
                if let (Some(my), Some(cy)) = (my, cy) {
                    if cy < my {
                        o.push(finding(format!("child-before-marriage:{c}"),"RELATIONSHIP_ANOMALY",Severity::Warning,Some(c.clone()),None,"El nacimiento precede al matrimonio registrado; podría reflejar otra unión o una fecha incompleta.".into(),vec![format!("marriage year: {my}"),format!("birth year: {cy}")],0.76));
                    }
                }
                for parent in f.husband_id.iter().chain(f.wife_id.iter()) {
                    if let (Some(py), Some(cy)) = (
                        t.person(parent)
                            .and_then(|p| p.birth_date.as_ref())
                            .and_then(|d| d.year),
                        cy,
                    ) {
                        let age = cy - py;
                        if age < self.config.minimum_parent_age
                            || age > self.config.unusual_parent_age
                        {
                            let severe = if age < self.config.minimum_parent_age {
                                Severity::High
                            } else {
                                Severity::Warning
                            };
                            o.push(finding(format!("parent-age:{parent}:{c}"),"AGE_ANOMALY",severe,Some(parent.clone()),Some(c.clone()),format!("Edad parental {age} años (UNUSUAL/IMPOSSIBLE según umbrales); no es una afirmación de error."),vec![format!("parent birth: {py}"),format!("child birth: {cy}"),format!("calculated age: {age}")],if age<self.config.minimum_parent_age{0.91}else{0.73}));
                        }
                    }
                }
            }
        }
        o
    }
}
impl AnalysisRule for ResearchGapRule {
    fn id(&self) -> &'static str {
        "research-gaps"
    }
    fn analyze(&self, t: &GenealogyTree) -> Vec<Finding> {
        t.persons.iter().filter(|p|p.family_child.is_none()).map(|p|finding(format!("gap:{}",p.gedcom_id),"research-gap",Severity::High,Some(p.gedcom_id.clone()),None,"Hueco contextualizado: faltan progenitores y existe contexto para buscar en bautismos o matrimonios.".into(),vec!["missing: parents".into(),"potential sources: parish registers".into()],0.82)).collect()
    }
}
impl AnalysisRule for DuplicateRule {
    fn id(&self) -> &'static str {
        "duplicates"
    }
    fn analyze(&self, t: &GenealogyTree) -> Vec<Finding> {
        let mut o = Vec::new();
        for (i, a) in t.persons.iter().enumerate() {
            for b in t.persons.iter().skip(i + 1) {
                let n = normalize_text(&format!("{} {}", a.given_name, a.surname))
                    == normalize_text(&format!("{} {}", b.given_name, b.surname));
                let d = a
                    .birth_date
                    .as_ref()
                    .and_then(|x| x.year)
                    .zip(b.birth_date.as_ref().and_then(|x| x.year))
                    .map(|(x, y)| (x - y).abs() <= 2)
                    .unwrap_or(false);
                let p = a
                    .birth_place
                    .as_ref()
                    .zip(b.birth_place.as_ref())
                    .map(|(x, y)| normalize_text(x) == normalize_text(y))
                    .unwrap_or(false);
                let m = n as u8 + d as u8 + p as u8;
                if m >= 2 {
                    o.push(finding(format!("duplicate:{}:{}",a.gedcom_id,b.gedcom_id),"POSSIBLE_DUPLICATE",Severity::Warning,Some(a.gedcom_id.clone()),Some(b.gedcom_id.clone()),format!("Posibles duplicados; probabilidad aproximada {} %. No fusionar automáticamente.",55+m*12),vec![format!("same name: {n}"),format!("near birth year: {d}"),format!("near birth place: {p}")],0.55+m as f32*0.12));
                }
            }
        }
        o
    }
}
impl AnalysisRule for SourceGapRule {
    fn id(&self) -> &'static str {
        "source-gaps"
    }
    fn analyze(&self, t: &GenealogyTree) -> Vec<Finding> {
        t.persons
            .iter()
            .filter(|p| p.sources.is_empty())
            .map(|p| {
                finding(
                    format!("source-gap:{}", p.gedcom_id),
                    "missing-source",
                    Severity::Medium,
                    Some(p.gedcom_id.clone()),
                    None,
                    "La persona no tiene fuentes asociadas.".into(),
                    vec!["no person source citations".into()],
                    0.99,
                )
            })
            .collect()
    }
}
pub fn analyze_with_config(t: &GenealogyTree, c: AnalysisConfig) -> Vec<Finding> {
    let r: [&dyn AnalysisRule; 5] = [
        &MissingDataRule,
        &ChronologyRule { config: c },
        &ResearchGapRule,
        &DuplicateRule,
        &SourceGapRule,
    ];
    r.iter().flat_map(|x| x.analyze(t)).collect()
}
pub fn analyze(t: &GenealogyTree) -> Vec<Finding> {
    analyze_with_config(t, AnalysisConfig::default())
}
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct AncestryStats {
    pub maximum_depth: usize,
    pub average_depth: f32,
    pub missing_generations: Vec<usize>,
}
pub fn ancestry_stats(t: &GenealogyTree, root: &str) -> AncestryStats {
    let n = t.ancestors(root);
    let d: Vec<usize> = n.iter().map(|x| x.0).collect();
    let max = *d.iter().max().unwrap_or(&0);
    AncestryStats {
        maximum_depth: max,
        average_depth: if d.is_empty() {
            0.0
        } else {
            d.iter().sum::<usize>() as f32 / d.len() as f32
        },
        missing_generations: (1..=max).filter(|g| !d.contains(g)).collect(),
    }
}
