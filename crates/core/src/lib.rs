use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum DatePrecision {
    Exact,
    Year,
    About,
    Before,
    After,
    Between,
    FromTo,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DateRange {
    pub start_year: Option<i32>,
    pub end_year: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DateValue {
    pub raw: String,
    pub year: Option<i32>,
    pub date: Option<NaiveDate>,
    pub approximate: bool,
    pub precision: DatePrecision,
    pub range: Option<DateRange>,
}

impl DateValue {
    pub fn year(year: i32) -> Self {
        Self {
            raw: year.to_string(),
            year: Some(year),
            date: None,
            approximate: false,
            precision: DatePrecision::Year,
            range: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawTag {
    pub level: u8,
    pub tag: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Person {
    pub id: usize,
    pub gedcom_id: String,
    pub given_name: String,
    pub surname: String,
    pub sex: Option<String>,
    pub birth_date: Option<DateValue>,
    pub birth_place: Option<String>,
    pub death_date: Option<DateValue>,
    pub death_place: Option<String>,
    pub occupation: Option<String>,
    pub notes: Vec<String>,
    pub confidence: Option<f32>,
    pub family_child: Option<String>,
    pub family_spouse: Vec<String>,
    pub sources: Vec<String>,
    pub raw: Vec<RawTag>,
    pub name_original: Option<String>,
    pub name_normalized: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Family {
    pub id: usize,
    pub gedcom_id: String,
    pub husband_id: Option<String>,
    pub wife_id: Option<String>,
    pub children: Vec<String>,
    pub marriage_date: Option<DateValue>,
    pub marriage_place: Option<String>,
    pub sources: Vec<String>,
    pub raw: Vec<RawTag>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Event {
    pub id: usize,
    pub person_id: Option<String>,
    pub family_id: Option<String>,
    pub event_type: String,
    pub date: Option<DateValue>,
    pub place: Option<String>,
    pub description: Option<String>,
    pub sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Source {
    pub id: usize,
    pub gedcom_id: String,
    pub title: String,
    pub author: Option<String>,
    pub repository: Option<String>,
    pub url: Option<String>,
    pub citation: Option<String>,
    pub confidence: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Finding {
    pub id: String,
    pub kind: String,
    pub severity: Severity,
    pub person_id: Option<String>,
    pub related_person_id: Option<String>,
    pub description: String,
    pub evidence: Vec<String>,
    pub confidence: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Severity {
    Info,
    Warning,
    High,
    Critical,
    Medium,
    #[default]
    Low,
}

impl Severity {
    pub fn rank(&self) -> u8 {
        match self {
            Severity::Low => 0,
            Severity::Info => 1,
            Severity::Medium => 2,
            Severity::Warning => 3,
            Severity::High => 4,
            Severity::Critical => 5,
        }
    }
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "low" => Some(Self::Low),
            "info" => Some(Self::Info),
            "medium" => Some(Self::Medium),
            "warning" => Some(Self::Warning),
            "high" => Some(Self::High),
            "critical" => Some(Self::Critical),
            _ => None,
        }
    }
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Info => "info",
            Self::Medium => "medium",
            Self::Warning => "warning",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

// --- New types for Research Engine v2 ---

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum Researchability {
    #[default]
    Low,
    Medium,
    High,
}

impl Researchability {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScoreComponent {
    pub name: String,
    pub points: i32,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScoreBreakdown {
    pub total: u8,
    pub components: Vec<ScoreComponent>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResearchOpportunity {
    pub person_id: String,
    pub score: u8,
    pub confidence: f32,
    pub priority: Severity,
    pub researchability: Researchability,
    pub breakdown: ScoreBreakdown,
    pub reasons: Vec<String>,
    pub suggested_sources: Vec<String>,
    pub missing_information: Vec<String>,
    pub why_it_matters: String,
    pub what_is_known: Vec<String>,
    pub potential_sources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SourceCoverage {
    pub birth: f32,
    pub marriage: f32,
    pub death: f32,
    pub other_events: f32,
    pub overall: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BranchAnalysis {
    pub name: String,
    pub score: u8,
    pub opportunity_count: usize,
    pub high_priority_count: usize,
    pub deepest_generation: usize,
    pub source_coverage: f32,
    pub person_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AncestorInfo {
    pub person_id: String,
    pub is_direct_ancestor: bool,
    pub generation_distance: Option<usize>,
    pub ancestor_paths: Vec<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GenealogyTree {
    pub persons: Vec<Person>,
    pub families: Vec<Family>,
    pub events: Vec<Event>,
    pub sources: Vec<Source>,
    pub places: Vec<Place>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Place {
    pub id: usize,
    pub name: String,
    pub original: String,
    pub normalized_name: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub historical_names: Vec<String>,
}

pub fn normalize_text(value: &str) -> String {
    use unicode_normalization::UnicodeNormalization;
    value
        .nfkd()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

impl GenealogyTree {
    pub fn person(&self, id: &str) -> Option<&Person> {
        self.persons.iter().find(|p| p.gedcom_id == id)
    }
    pub fn family(&self, id: &str) -> Option<&Family> {
        self.families.iter().find(|f| f.gedcom_id == id)
    }
    pub fn ancestors(&self, id: &str) -> Vec<(usize, String)> {
        let mut out = Vec::new();
        let mut visited = std::collections::HashSet::new();
        self.ancestor_walk(id, 0, &mut out, &mut visited);
        out
    }
    fn ancestor_walk(
        &self,
        id: &str,
        depth: usize,
        out: &mut Vec<(usize, String)>,
        visited: &mut std::collections::HashSet<String>,
    ) {
        if !visited.insert(id.to_string()) {
            return;
        }
        if let Some(p) = self.person(id) {
            out.push((depth, id.to_string()));
            if let Some(fid) = &p.family_child {
                if let Some(f) = self.family(fid) {
                    for parent in [&f.husband_id, &f.wife_id].into_iter().flatten() {
                        self.ancestor_walk(parent, depth + 1, out, visited);
                    }
                }
            }
        }
    }

    /// Returns all ancestor paths from `id` to root ancestors.
    /// Each path is a Vec<String> of gedcom_ids starting at `id`.
    pub fn ancestor_paths(&self, id: &str) -> Vec<Vec<String>> {
        let mut paths = Vec::new();
        let mut current = vec![id.to_string()];
        let mut visited = std::collections::HashSet::new();
        self.collect_paths(id, &mut current, &mut paths, &mut visited);
        if paths.is_empty() {
            vec![vec![id.to_string()]]
        } else {
            paths
        }
    }

    fn collect_paths(
        &self,
        id: &str,
        current: &mut Vec<String>,
        paths: &mut Vec<Vec<String>>,
        visited: &mut std::collections::HashSet<String>,
    ) {
        if !visited.insert(id.to_string()) {
            // cycle detected: record current path and stop
            paths.push(current.clone());
            visited.remove(id);
            return;
        }
        if let Some(p) = self.person(id) {
            if let Some(fid) = &p.family_child {
                if let Some(f) = self.family(fid) {
                    let parents: Vec<&String> =
                        [&f.husband_id, &f.wife_id].into_iter().flatten().collect();
                    if parents.is_empty() {
                        paths.push(current.clone());
                    } else {
                        for parent in parents {
                            current.push(parent.clone());
                            self.collect_paths(parent, current, paths, visited);
                            current.pop();
                        }
                    }
                    visited.remove(id);
                    return;
                }
            }
        }
        // leaf ancestor (no parents known)
        paths.push(current.clone());
        visited.remove(id);
    }

    /// Generation distance to the deepest root reachable.
    pub fn generation_distance(&self, id: &str) -> Option<usize> {
        let anc = self.ancestors(id);
        anc.iter().map(|(d, _)| *d).max()
    }

    /// Determine if `candidate` is a direct ancestor of `root` (or of any leaf if root not specified).
    /// We consider direct ancestor as someone who lies on at least one ancestor path of `root`.
    pub fn is_direct_ancestor_of(&self, candidate: &str, root: &str) -> bool {
        if candidate == root {
            return true;
        }
        self.ancestors(root).iter().any(|(_, pid)| pid == candidate)
    }

    /// All persons who are ancestors of at least one other person (have descendants).
    pub fn persons_with_descendants(&self) -> std::collections::HashSet<String> {
        let mut has_desc = std::collections::HashSet::new();
        for f in &self.families {
            for parent in [&f.husband_id, &f.wife_id].into_iter().flatten() {
                has_desc.insert(parent.clone());
            }
        }
        has_desc
    }
}
