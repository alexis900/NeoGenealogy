use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ResearchSessionStats {
    pub total_tasks: i64,
    pub completed_tasks: i64,
    pub open_tasks: i64,
    pub in_progress_tasks: i64,
    pub inconclusive_tasks: i64,
    pub rejected_tasks: i64,
    pub total_outcomes: i64,
    pub confirmed_outcomes: i64,
    pub false_lead_outcomes: i64,
    pub inconclusive_outcomes: i64,
    pub new_lead_outcomes: i64,
    pub no_evidence_outcomes: i64,
    pub total_evidence: i64,
    pub supporting_evidence: i64,
    pub contradicting_evidence: i64,
    pub open_followups: i64,
    pub completed_followup_actions: i64,
    pub skipped_followup_actions: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResearchSessionTimelineEvent {
    pub event_type: String,
    pub timestamp: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchActivitySummary {
    pub tasks: serde_json::Value,
    pub outcomes: serde_json::Value,
    pub evidence: serde_json::Value,
    pub followups: serde_json::Value,
}
