use serde::{Deserialize, Serialize};

use crate::assessment::{EvidenceAssessment, EvidenceGap, ResearchFollowUp};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResearchCaseTimelineEvent {
    pub event_type: String,
    pub timestamp: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResearchCaseClosureWarning {
    pub code: String,
    pub severity: String,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseSummaryTask {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub status: String,
    pub resolution: Option<String>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseSummaryPerson {
    pub person_id: i64,
    pub person_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseSummaryOpportunity {
    pub opportunity_id: i64,
    pub score: Option<i64>,
    pub priority: Option<String>,
    pub researchability: Option<String>,
    pub confidence: Option<f64>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseSummaryOutcome {
    pub outcome_id: i64,
    pub r#type: String,
    pub summary: String,
    pub details: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct ResearchCaseSummary {
    pub task: CaseSummaryTask,
    pub person: Option<CaseSummaryPerson>,
    pub opportunity: Option<CaseSummaryOpportunity>,
    pub outcome: Option<CaseSummaryOutcome>,
    pub evidence_assessment: Option<EvidenceAssessment>,
    pub evidence_gaps: Vec<EvidenceGap>,
    pub research_followups: Vec<ResearchFollowUp>,
    pub followup_actions: Vec<crate::models::ResearchFollowupActionRow>,
    pub timeline: Vec<ResearchCaseTimelineEvent>,
    pub closure_warnings: Vec<ResearchCaseClosureWarning>,
}

pub fn calculate_closure_warnings(
    task_status: &str,
    outcome_type: Option<&str>,
    assessment_status: Option<&str>,
    evidence_gaps: &[EvidenceGap],
) -> Vec<ResearchCaseClosureWarning> {
    let mut warnings = Vec::new();

    if task_status == "RESOLVED" && outcome_type.is_none() {
        warnings.push(ResearchCaseClosureWarning {
            code: "RESOLVED_WITHOUT_OUTCOME".into(),
            severity: "WARNING".into(),
            title: "Resolved without outcome".into(),
            description: "This research task is marked as resolved but has no research outcome."
                .into(),
        });
    }

    if let Some(ot) = outcome_type {
        if ot == "CONFIRMED" && assessment_status == Some("NO_EVIDENCE") {
            warnings.push(ResearchCaseClosureWarning {
                code: "CONFIRMED_WITHOUT_SUPPORT".into(),
                severity: "CRITICAL".into(),
                title: "Confirmed without support".into(),
                description: "Confirmed outcome has no supporting evidence.".into(),
            });
        }
    }

    if task_status == "RESOLVED" && outcome_type.is_some() && !evidence_gaps.is_empty() {
        warnings.push(ResearchCaseClosureWarning {
            code: "RESOLVED_WITH_EVIDENCE_GAPS".into(),
            severity: "INFO".into(),
            title: "Resolved with evidence gaps".into(),
            description: "This case was resolved while evidence gaps remain.".into(),
        });
    }

    if let Some(ot) = outcome_type {
        if task_status == "REJECTED" && ot == "CONFIRMED" {
            warnings.push(ResearchCaseClosureWarning {
                code: "REJECTED_WITH_CONFIRMED_OUTCOME".into(),
                severity: "WARNING".into(),
                title: "Rejected with confirmed outcome".into(),
                description: "The task is marked as rejected but its outcome is CONFIRMED.".into(),
            });
        }
        if task_status == "INCONCLUSIVE" && ot == "CONFIRMED" {
            warnings.push(ResearchCaseClosureWarning {
                code: "INCONCLUSIVE_WITH_CONFIRMED_OUTCOME".into(),
                severity: "WARNING".into(),
                title: "Inconclusive with confirmed outcome".into(),
                description: "The task is marked as inconclusive but its outcome is CONFIRMED."
                    .into(),
            });
        }
    }

    warnings
}

pub fn build_timeline(
    task: &crate::models::ResearchTaskRow,
    outcome: Option<&crate::models::ResearchOutcomeRow>,
    followup_actions: &[crate::models::ResearchFollowupActionRow],
) -> Vec<ResearchCaseTimelineEvent> {
    let mut events: Vec<ResearchCaseTimelineEvent> = Vec::new();

    events.push(ResearchCaseTimelineEvent {
        event_type: "TASK_CREATED".into(),
        timestamp: task.created_at.clone(),
        label: "Task created".into(),
    });

    if let Some(started) = &task.started_at {
        events.push(ResearchCaseTimelineEvent {
            event_type: "TASK_STARTED".into(),
            timestamp: started.clone(),
            label: "Research started".into(),
        });
    }

    if let Some(o) = outcome {
        events.push(ResearchCaseTimelineEvent {
            event_type: "OUTCOME_CREATED".into(),
            timestamp: o.created_at.clone(),
            label: "Outcome recorded".into(),
        });
        if o.updated_at != o.created_at {
            events.push(ResearchCaseTimelineEvent {
                event_type: "OUTCOME_UPDATED".into(),
                timestamp: o.updated_at.clone(),
                label: "Outcome updated".into(),
            });
        }
    }

    for fa in followup_actions {
        events.push(ResearchCaseTimelineEvent {
            event_type: "FOLLOWUP_ACTION_CREATED".into(),
            timestamp: fa.created_at.clone(),
            label: format!("Follow-up {} created", fa.followup_code),
        });
        if let Some(completed) = &fa.completed_at {
            events.push(ResearchCaseTimelineEvent {
                event_type: "FOLLOWUP_ACTION_COMPLETED".into(),
                timestamp: completed.clone(),
                label: format!("Follow-up {} completed", fa.followup_code),
            });
        }
    }

    if let Some(completed) = &task.completed_at {
        events.push(ResearchCaseTimelineEvent {
            event_type: "TASK_COMPLETED".into(),
            timestamp: completed.clone(),
            label: "Task completed".into(),
        });
    }

    // Deterministic ordering: timestamp ASC, then rank, then event_type lexical
    fn rank(t: &str) -> u8 {
        match t {
            "TASK_CREATED" => 0,
            "TASK_STARTED" => 1,
            "OUTCOME_CREATED" => 2,
            "FOLLOWUP_ACTION_CREATED" => 3,
            "FOLLOWUP_ACTION_COMPLETED" => 4,
            "OUTCOME_UPDATED" => 5,
            "TASK_COMPLETED" => 6,
            _ => 99,
        }
    }

    events.sort_by(|a, b| {
        let ord = a.timestamp.cmp(&b.timestamp);
        if ord != std::cmp::Ordering::Equal {
            return ord;
        }
        let ra = rank(&a.event_type);
        let rb = rank(&b.event_type);
        if ra != rb {
            return ra.cmp(&rb);
        }
        a.event_type.cmp(&b.event_type)
    });

    events
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gap(code: &str) -> EvidenceGap {
        EvidenceGap {
            code: code.into(),
            severity: "INFO".into(),
            title: "t".into(),
            description: "d".into(),
        }
    }

    #[test]
    fn test_resolved_without_outcome() {
        let w = calculate_closure_warnings("RESOLVED", None, None, &[]);
        assert!(w.iter().any(|x| x.code == "RESOLVED_WITHOUT_OUTCOME"));
        assert_eq!(w[0].severity, "WARNING");
    }

    #[test]
    fn test_no_warning_resolved_with_outcome() {
        let w = calculate_closure_warnings("RESOLVED", Some("CONFIRMED"), Some("SUPPORTED"), &[]);
        assert!(!w.iter().any(|x| x.code == "RESOLVED_WITHOUT_OUTCOME"));
    }

    #[test]
    fn test_confirmed_without_support() {
        let w =
            calculate_closure_warnings("IN_PROGRESS", Some("CONFIRMED"), Some("NO_EVIDENCE"), &[]);
        assert!(w.iter().any(|x| x.code == "CONFIRMED_WITHOUT_SUPPORT"));
        assert_eq!(
            w.iter()
                .find(|x| x.code == "CONFIRMED_WITHOUT_SUPPORT")
                .unwrap()
                .severity,
            "CRITICAL"
        );
    }

    #[test]
    fn test_confirmed_with_support_no_warning() {
        let w = calculate_closure_warnings("RESOLVED", Some("CONFIRMED"), Some("SUPPORTED"), &[]);
        assert!(!w.iter().any(|x| x.code == "CONFIRMED_WITHOUT_SUPPORT"));
    }

    #[test]
    fn test_resolved_with_gaps() {
        let w = calculate_closure_warnings(
            "RESOLVED",
            Some("CONFIRMED"),
            Some("SUPPORTED"),
            &[gap("SINGLE_SOURCE")],
        );
        assert!(w.iter().any(|x| x.code == "RESOLVED_WITH_EVIDENCE_GAPS"));
        assert_eq!(
            w.iter()
                .find(|x| x.code == "RESOLVED_WITH_EVIDENCE_GAPS")
                .unwrap()
                .severity,
            "INFO"
        );
    }

    #[test]
    fn test_resolved_no_gap_no_warning() {
        let w = calculate_closure_warnings("RESOLVED", Some("CONFIRMED"), Some("SUPPORTED"), &[]);
        assert!(!w.iter().any(|x| x.code == "RESOLVED_WITH_EVIDENCE_GAPS"));
    }

    #[test]
    fn test_rejected_with_confirmed() {
        let w = calculate_closure_warnings("REJECTED", Some("CONFIRMED"), Some("SUPPORTED"), &[]);
        assert!(w
            .iter()
            .any(|x| x.code == "REJECTED_WITH_CONFIRMED_OUTCOME"));
    }

    #[test]
    fn test_rejected_not_confirmed_no_warning() {
        let w = calculate_closure_warnings("REJECTED", Some("FALSE_LEAD"), Some("SUPPORTED"), &[]);
        assert!(!w
            .iter()
            .any(|x| x.code == "REJECTED_WITH_CONFIRMED_OUTCOME"));
    }

    #[test]
    fn test_inconclusive_with_confirmed() {
        let w =
            calculate_closure_warnings("INCONCLUSIVE", Some("CONFIRMED"), Some("SUPPORTED"), &[]);
        assert!(w
            .iter()
            .any(|x| x.code == "INCONCLUSIVE_WITH_CONFIRMED_OUTCOME"));
    }

    #[test]
    fn test_clean_resolved_no_warnings() {
        let w = calculate_closure_warnings(
            "RESOLVED",
            Some("CONFIRMED"),
            Some("STRONGLY_SUPPORTED"),
            &[],
        );
        assert!(w.is_empty());
    }

    #[test]
    fn test_multiple_warnings() {
        // RESOLVED + CONFIRMED + NO_EVIDENCE + gaps => 3 warnings
        let w = calculate_closure_warnings(
            "RESOLVED",
            Some("CONFIRMED"),
            Some("NO_EVIDENCE"),
            &[gap("SINGLE_SOURCE")],
        );
        assert!(w.iter().any(|x| x.code == "CONFIRMED_WITHOUT_SUPPORT"));
        assert!(w.iter().any(|x| x.code == "RESOLVED_WITH_EVIDENCE_GAPS"));
        // No RESOLVED_WITHOUT_OUTCOME because outcome exists
        assert!(!w.iter().any(|x| x.code == "RESOLVED_WITHOUT_OUTCOME"));
        assert_eq!(w.len(), 2);
    }

    #[test]
    fn test_open_no_warning() {
        let w = calculate_closure_warnings("OPEN", None, None, &[]);
        assert!(w.is_empty());
    }

    #[test]
    fn test_in_progress_no_warning() {
        let w = calculate_closure_warnings(
            "IN_PROGRESS",
            Some("CONFIRMED"),
            Some("WEAK"),
            &[gap("SINGLE_SOURCE")],
        );
        // Only RESOLVED_WITH_EVIDENCE_GAPS triggers on RESOLVED, so none
        assert!(!w.iter().any(|x| x.code == "RESOLVED_WITH_EVIDENCE_GAPS"));
        assert!(w.is_empty());
    }

    #[test]
    fn test_timeline_ordering() {
        let task = crate::models::ResearchTaskRow {
            id: 1,
            tree_id: 1,
            opportunity_id: None,
            person_id: None,
            title: "t".into(),
            description: None,
            status: "RESOLVED".into(),
            created_at: "2024-01-01T00:00:00Z".into(),
            updated_at: "2024-01-01T00:00:00Z".into(),
            started_at: Some("2024-01-02T00:00:00Z".into()),
            completed_at: Some("2024-01-05T00:00:00Z".into()),
            resolution: None,
        };
        let outcome = crate::models::ResearchOutcomeRow {
            id: 1,
            tree_id: 1,
            task_id: 1,
            r#type: "CONFIRMED".into(),
            summary: "s".into(),
            details: None,
            created_at: "2024-01-03T00:00:00Z".into(),
            updated_at: "2024-01-04T00:00:00Z".into(),
        };
        let fa = crate::models::ResearchFollowupActionRow {
            id: 1,
            tree_id: 1,
            task_id: 1,
            outcome_id: 1,
            followup_code: "ADD_CITATION".into(),
            status: "COMPLETED".into(),
            notes: None,
            created_at: "2024-01-03T12:00:00Z".into(),
            updated_at: "2024-01-03T12:00:00Z".into(),
            completed_at: Some("2024-01-04T12:00:00Z".into()),
        };
        let tl = build_timeline(&task, Some(&outcome), &[fa]);
        // Should be sorted
        assert_eq!(tl[0].event_type, "TASK_CREATED");
        assert_eq!(tl[1].event_type, "TASK_STARTED");
        assert_eq!(tl[2].event_type, "OUTCOME_CREATED");
        // FOLLOWUP_ACTION_CREATED at 03T12:00 comes before OUTCOME_UPDATED at 04T00? Actually 03T12 before 04T00
        assert!(tl.iter().any(|e| e.event_type == "OUTCOME_UPDATED"));
        assert!(tl
            .iter()
            .any(|e| e.event_type == "FOLLOWUP_ACTION_COMPLETED"));
        assert_eq!(tl.last().unwrap().event_type, "TASK_COMPLETED");
        // Deterministic tie
        let times = tl.iter().map(|e| e.timestamp.clone()).collect::<Vec<_>>();
        let mut sorted = times.clone();
        sorted.sort();
        // Already sorted
        assert_eq!(times, sorted);
    }
}
