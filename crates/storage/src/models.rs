use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TreeRow {
    pub id: i64,
    pub name: String,
    pub source_filename: Option<String>,
    pub gedcom_version: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PersonRow {
    pub id: i64,
    pub tree_id: i64,
    pub gedcom_id: String,
    pub given_name: Option<String>,
    pub surname: Option<String>,
    pub display_name: Option<String>,
    pub sex: Option<String>,
    pub raw_name: Option<String>,
    pub birth_date_original: Option<String>,
    pub birth_date_precision: Option<String>,
    pub birth_date_year: Option<i32>,
    pub birth_date_start: Option<i32>,
    pub birth_date_end: Option<i32>,
    pub birth_place: Option<String>,
    pub death_date_original: Option<String>,
    pub death_date_precision: Option<String>,
    pub death_date_year: Option<i32>,
    pub death_place: Option<String>,
    pub occupation: Option<String>,
    pub raw_tags: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FamilyRow {
    pub id: i64,
    pub tree_id: i64,
    pub gedcom_id: String,
    pub raw_tags: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FamilyMemberRow {
    pub id: i64,
    pub family_id: i64,
    pub person_id: i64,
    pub role: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PlaceRow {
    pub id: i64,
    pub tree_id: i64,
    pub raw_name: String,
    pub normalized_name: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EventRow {
    pub id: i64,
    pub tree_id: i64,
    pub person_id: Option<i64>,
    pub family_id: Option<i64>,
    pub event_type: String,
    pub date_original: Option<String>,
    pub date_precision: Option<String>,
    pub date_start: Option<i32>,
    pub date_end: Option<i32>,
    pub date_year: Option<i32>,
    pub place_id: Option<i64>,
    pub place_raw: Option<String>,
    pub raw_value: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SourceRow {
    pub id: i64,
    pub tree_id: i64,
    pub gedcom_id: String,
    pub title: Option<String>,
    pub author: Option<String>,
    pub publication: Option<String>,
    pub text: Option<String>,
    pub repository: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CitationRow {
    pub id: i64,
    pub tree_id: i64,
    pub source_id: i64,
    pub person_id: Option<i64>,
    pub family_id: Option<i64>,
    pub event_id: Option<i64>,
    pub page: Option<String>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AnalysisRunRow {
    pub id: i64,
    pub tree_id: i64,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub engine_version: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FindingRow {
    pub id: i64,
    pub tree_id: i64,
    pub analysis_run_id: Option<i64>,
    pub person_id: Option<i64>,
    pub family_id: Option<i64>,
    pub related_person_id: Option<i64>,
    pub finding_type: String,
    pub severity: String,
    pub confidence: Option<f64>,
    pub message: Option<String>,
    pub evidence: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ResearchOpportunityRow {
    pub id: i64,
    pub tree_id: i64,
    pub analysis_run_id: Option<i64>,
    pub person_id: i64,
    pub finding_id: Option<i64>,
    pub priority: Option<String>,
    pub score: Option<i64>,
    pub confidence: Option<f64>,
    pub researchability: Option<String>,
    pub why: Option<String>,
    pub what: Option<String>,
    pub potential_sources: Option<String>,
    pub breakdown: Option<String>,
    pub missing_information: Option<String>,
    pub reasons: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct BranchAnalysisRow {
    pub id: i64,
    pub tree_id: i64,
    pub analysis_run_id: i64,
    pub name: String,
    pub score: Option<i64>,
    pub opportunity_count: Option<i64>,
    pub high_priority_count: Option<i64>,
    pub deepest_generation: Option<i64>,
    pub source_coverage: Option<f64>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SourceCoverageRow {
    pub id: i64,
    pub tree_id: i64,
    pub analysis_run_id: i64,
    pub birth: Option<f64>,
    pub marriage: Option<f64>,
    pub death: Option<f64>,
    pub other_events: Option<f64>,
    pub overall: Option<f64>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ResearchTaskRow {
    pub id: i64,
    pub tree_id: i64,
    pub opportunity_id: Option<i64>,
    pub person_id: Option<i64>,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub resolution: Option<String>,
    pub session_id: Option<i64>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ResearchOutcomeRow {
    pub id: i64,
    pub tree_id: i64,
    pub task_id: i64,
    pub r#type: String,
    pub summary: String,
    pub details: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ResearchSourceRow {
    pub id: i64,
    pub tree_id: i64,
    pub title: String,
    pub author: Option<String>,
    pub publication: Option<String>,
    pub date: Option<String>,
    pub r#type: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ResearchCitationRow {
    pub id: i64,
    pub source_id: i64,
    pub locator: Option<String>,
    pub text: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EvidenceRow {
    pub id: i64,
    pub tree_id: i64,
    pub source_id: i64,
    pub citation_id: Option<i64>,
    pub statement: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct OutcomeEvidenceRow {
    pub outcome_id: i64,
    pub evidence_id: i64,
    pub relationship: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ResearchFollowupActionRow {
    pub id: i64,
    pub tree_id: i64,
    pub task_id: i64,
    pub outcome_id: i64,
    pub followup_code: String,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub completed_at: Option<String>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ResearchSessionRow {
    pub id: i64,
    pub tree_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub person_id: Option<i64>,
    pub opportunity_id: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
}

// Helper for counts
#[derive(Debug, sqlx::FromRow)]
pub struct CountRow {
    pub cnt: i64,
}

pub fn now_iso() -> String {
    chrono::Utc::now().to_rfc3339()
}

pub fn normalize_opt(s: Option<&str>) -> Option<String> {
    s.map(neogenealogy_core::normalize_text)
}
