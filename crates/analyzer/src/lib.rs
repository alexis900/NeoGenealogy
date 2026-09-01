use neogenealogy_core::*;
use std::collections::{HashMap, HashSet};

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
pub struct CycleRule;
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
impl AnalysisRule for CycleRule {
    fn id(&self) -> &'static str {
        "cycles"
    }
    fn analyze(&self, t: &GenealogyTree) -> Vec<Finding> {
        detect_cycles(t)
    }
}

pub fn detect_cycles(tree: &GenealogyTree) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut visited_global: HashSet<String> = HashSet::new();

    for person in &tree.persons {
        let start = &person.gedcom_id;
        if visited_global.contains(start) {
            continue;
        }
        let mut stack: Vec<String> = Vec::new();
        let mut in_stack: HashSet<String> = HashSet::new();
        dfs_cycle(
            tree,
            start,
            &mut stack,
            &mut in_stack,
            &mut visited_global,
            &mut findings,
        );
    }
    findings
}

fn dfs_cycle(
    tree: &GenealogyTree,
    current: &str,
    stack: &mut Vec<String>,
    in_stack: &mut HashSet<String>,
    visited_global: &mut HashSet<String>,
    findings: &mut Vec<Finding>,
) {
    if in_stack.contains(current) {
        // found cycle: current already in stack
        let pos = stack.iter().position(|x| x == current).unwrap_or(0);
        let cycle: Vec<String> = stack[pos..].to_vec();
        let mut cycle_with_current = cycle.clone();
        cycle_with_current.push(current.to_string());
        let evidence = vec![
            format!("cycle: {}", cycle_with_current.join(" -> ")),
            format!("length: {}", cycle_with_current.len() - 1),
        ];
        findings.push(finding(
            format!("cycle:{}", cycle_with_current.join("-")),
            "RELATIONSHIP_ANOMALY",
            Severity::Critical,
            Some(current.to_string()),
            None,
            format!(
                "Ciclo genealógico detectado: {} es ancestro de sí mismo. Revisar relaciones familiares.",
                current
            ),
            evidence,
            0.99,
        ));
        return;
    }
    if visited_global.contains(current) {
        return;
    }
    visited_global.insert(current.to_string());
    stack.push(current.to_string());
    in_stack.insert(current.to_string());

    if let Some(p) = tree.person(current) {
        if let Some(fid) = &p.family_child {
            if let Some(f) = tree.family(fid) {
                for parent in [&f.husband_id, &f.wife_id].into_iter().flatten() {
                    dfs_cycle(tree, parent, stack, in_stack, visited_global, findings);
                }
            }
        }
    }

    stack.pop();
    in_stack.remove(current);
}

pub fn source_coverage(tree: &GenealogyTree) -> SourceCoverage {
    let total_persons = tree.persons.len() as f32;
    let total_families = tree.families.len() as f32;

    let birth_covered = if total_persons > 0.0 {
        tree.persons
            .iter()
            .filter(|p| p.birth_date.is_some() || p.birth_place.is_some())
            .count() as f32
            / total_persons
            * 100.0
    } else {
        0.0
    };
    let death_covered = if total_persons > 0.0 {
        tree.persons
            .iter()
            .filter(|p| p.death_date.is_some() || p.death_place.is_some())
            .count() as f32
            / total_persons
            * 100.0
    } else {
        0.0
    };
    let marriage_covered = if total_families > 0.0 {
        tree.families
            .iter()
            .filter(|f| f.marriage_date.is_some() || f.marriage_place.is_some())
            .count() as f32
            / total_families
            * 100.0
    } else {
        0.0
    };
    // other events: count persons with at least one event not birth/death/marriage
    let other_covered = if total_persons > 0.0 {
        let persons_with_other: HashSet<String> = tree
            .events
            .iter()
            .filter(|e| !matches!(e.event_type.as_str(), "BIRT" | "DEAT" | "MARR"))
            .filter_map(|e| e.person_id.clone())
            .collect();
        persons_with_other.len() as f32 / total_persons * 100.0
    } else {
        0.0
    };
    let overall = if total_persons > 0.0 {
        let persons_with_source = tree
            .persons
            .iter()
            .filter(|p| !p.sources.is_empty())
            .count() as f32
            / total_persons
            * 100.0;
        // weighted average: birth, marriage, death, other, persons_with_source
        // but spec expects: birth, marriage, death, events, overall => overall is combination
        // We compute overall as average of birth,marriage,death + source linkage
        let avg = (birth_covered + marriage_covered + death_covered + other_covered) / 4.0;
        (avg + persons_with_source) / 2.0
    } else {
        0.0
    };

    SourceCoverage {
        birth: birth_covered,
        marriage: marriage_covered,
        death: death_covered,
        other_events: other_covered,
        overall,
    }
}

pub fn branch_analyses(
    tree: &GenealogyTree,
    opportunities: &[ResearchOpportunity],
) -> Vec<BranchAnalysis> {
    // Group persons by surname (normalized)
    let mut branches: HashMap<String, Vec<&Person>> = HashMap::new();
    for p in &tree.persons {
        let key = p.surname.trim().to_string();
        if key.is_empty() {
            continue;
        }
        branches.entry(key).or_default().push(p);
    }

    let opp_map: HashMap<String, &ResearchOpportunity> = opportunities
        .iter()
        .map(|o| (o.person_id.clone(), o))
        .collect();

    let mut result: Vec<BranchAnalysis> = Vec::new();
    for (surname, persons) in branches {
        let person_ids: Vec<String> = persons.iter().map(|p| p.gedcom_id.clone()).collect();
        let branch_opps: Vec<&ResearchOpportunity> = person_ids
            .iter()
            .filter_map(|id| opp_map.get(id).copied())
            .collect();

        let opportunity_count = branch_opps.len();
        let high_priority_count = branch_opps
            .iter()
            .filter(|o| matches!(o.priority, Severity::High | Severity::Critical))
            .count();

        // deepest generation among persons in branch
        let deepest_generation = persons
            .iter()
            .filter_map(|p| tree.generation_distance(&p.gedcom_id))
            .max()
            .unwrap_or(0);

        // source coverage for branch
        let with_sources = persons.iter().filter(|p| !p.sources.is_empty()).count() as f32;
        let source_coverage = if persons.is_empty() {
            0.0
        } else {
            with_sources / persons.len() as f32 * 100.0
        };

        // branch score derived from opportunities: avoid quantity over quality
        let score = if branch_opps.is_empty() {
            0
        } else {
            let mut scores: Vec<u8> = branch_opps.iter().map(|o| o.score).collect();
            scores.sort_by(|a, b| b.cmp(a));
            let max = *scores.iter().max().unwrap_or(&0) as f32;
            // average of top 5 to avoid dilution
            let top_n = scores
                .iter()
                .take(5)
                .copied()
                .map(|v| v as f32)
                .collect::<Vec<_>>();
            let avg_top = if top_n.is_empty() {
                0.0
            } else {
                top_n.iter().sum::<f32>() / top_n.len() as f32
            };
            // weighted: 60% max, 40% avg_top
            let combined = 0.6 * max + 0.4 * avg_top;
            combined.round().clamp(0.0, 100.0) as u8
        };

        result.push(BranchAnalysis {
            name: surname,
            score,
            opportunity_count,
            high_priority_count,
            deepest_generation,
            source_coverage,
            person_ids,
        });
    }
    // sort by score descending
    result.sort_by_key(|b| std::cmp::Reverse(b.score));
    result
}

pub fn ancestor_infos(tree: &GenealogyTree, root: &str) -> Vec<AncestorInfo> {
    let mut infos = Vec::new();
    let ancestors_set: HashSet<String> =
        tree.ancestors(root).into_iter().map(|(_, id)| id).collect();
    for p in &tree.persons {
        let is_direct = ancestors_set.contains(&p.gedcom_id);
        let generation_distance = if is_direct {
            tree.ancestors(root)
                .iter()
                .find(|(_, id)| id == &p.gedcom_id)
                .map(|(d, _)| *d)
        } else {
            None
        };
        let ancestor_paths = if is_direct {
            // paths from root to this ancestor? Actually from root upwards, we want ancestor paths from root that include person
            // We provide all paths from person upwards? For simplicity provide path from person via ancestors
            tree.ancestor_paths(&p.gedcom_id)
        } else {
            vec![]
        };
        infos.push(AncestorInfo {
            person_id: p.gedcom_id.clone(),
            is_direct_ancestor: is_direct,
            generation_distance,
            ancestor_paths,
        });
    }
    infos
}

pub fn analyze_with_config(t: &GenealogyTree, c: AnalysisConfig) -> Vec<Finding> {
    let missing = MissingDataRule;
    let chrono = ChronologyRule { config: c };
    let gap = ResearchGapRule;
    let dup = DuplicateRule;
    let src = SourceGapRule;
    let cycle = CycleRule;
    let r: Vec<&dyn AnalysisRule> = vec![&missing, &chrono, &gap, &dup, &src, &cycle];
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
