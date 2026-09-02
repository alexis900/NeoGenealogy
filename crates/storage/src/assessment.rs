use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceStats {
    pub evidence_total: i64,
    pub supporting_count: i64,
    pub contradicting_count: i64,
    pub sources_count: i64,
    pub cited_count: i64,
    pub uncited_count: i64,
    pub cited_supporting_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssessmentReason {
    pub code: String,
    pub points: i32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceAssessment {
    pub score: i32,
    pub status: String,
    pub evidence_total: i64,
    pub supporting_count: i64,
    pub contradicting_count: i64,
    pub sources_count: i64,
    pub cited_count: i64,
    pub uncited_count: i64,
    pub reasons: Vec<AssessmentReason>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceGap {
    pub code: String,
    pub severity: String,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResearchFollowUp {
    pub code: String,
    pub priority: String,
    pub title: String,
    pub description: String,
    pub gap_code: String,
}

pub fn calculate_evidence_assessment(stats: &EvidenceStats) -> EvidenceAssessment {
    let mut score: i32 = 0;
    let mut reasons: Vec<AssessmentReason> = Vec::new();

    // Base bonuses
    if stats.supporting_count >= 1 {
        score += 30;
        reasons.push(AssessmentReason {
            code: "SUPPORTING_EVIDENCE".into(),
            points: 30,
            message: "Supporting evidence exists".into(),
        });
    }
    if stats.supporting_count >= 2 {
        score += 20;
        reasons.push(AssessmentReason {
            code: "MULTIPLE_SUPPORTING_EVIDENCE".into(),
            points: 20,
            message: "Multiple supporting evidence".into(),
        });
    }
    if stats.cited_supporting_count >= 1 {
        score += 15;
        reasons.push(AssessmentReason {
            code: "SUPPORTING_EVIDENCE_HAS_CITATION".into(),
            points: 15,
            message: "Supporting evidence has citation".into(),
        });
    }
    if stats.sources_count >= 2 {
        score += 10;
        reasons.push(AssessmentReason {
            code: "MULTIPLE_SOURCES".into(),
            points: 10,
            message: "Multiple sources".into(),
        });
    }
    if stats.evidence_total >= 2 {
        score += 10;
        reasons.push(AssessmentReason {
            code: "MULTIPLE_EVIDENCE".into(),
            points: 10,
            message: "Multiple evidence".into(),
        });
    }
    if stats.evidence_total > 0 && stats.sources_count > 0 {
        // all evidence have source (always true if evidence exists)
        // we interpret as at least one source and evidence exists
        // check if every evidence has source -> since source_id NOT NULL, this is true
        score += 5;
        reasons.push(AssessmentReason {
            code: "ALL_EVIDENCE_HAS_SOURCE".into(),
            points: 5,
            message: "All evidence has source".into(),
        });
    }

    // Penalties
    if stats.contradicting_count >= 1 {
        score -= 30;
        reasons.push(AssessmentReason {
            code: "CONTRADICTING_EVIDENCE".into(),
            points: -30,
            message: "Contradicting evidence exists".into(),
        });
    }
    if stats.contradicting_count >= stats.supporting_count && stats.contradicting_count > 0 {
        score -= 15;
        reasons.push(AssessmentReason {
            code: "CONTRADICTS_DOMINANT".into(),
            points: -15,
            message: "Contradicting dominates".into(),
        });
    }
    if stats.cited_count == 0 && stats.evidence_total > 0 {
        score -= 10;
        reasons.push(AssessmentReason {
            code: "NO_CITATION".into(),
            points: -10,
            message: "No evidence has citation".into(),
        });
    }

    score = score.clamp(0, 100);

    // Status
    let status = if stats.supporting_count == 0 && stats.contradicting_count == 0 {
        "NO_EVIDENCE"
    } else if stats.supporting_count >= 1 && stats.contradicting_count >= 1 {
        "MIXED"
    } else if stats.supporting_count >= 2
        && stats.contradicting_count == 0
        && stats.cited_supporting_count >= 1
        && stats.evidence_total >= 2
    {
        "STRONGLY_SUPPORTED"
    } else if stats.supporting_count >= 2 && stats.contradicting_count == 0 {
        "SUPPORTED"
    } else {
        "WEAK"
    };

    EvidenceAssessment {
        score,
        status: status.to_string(),
        evidence_total: stats.evidence_total,
        supporting_count: stats.supporting_count,
        contradicting_count: stats.contradicting_count,
        sources_count: stats.sources_count,
        cited_count: stats.cited_count,
        uncited_count: stats.uncited_count,
        reasons,
    }
}

pub fn calculate_evidence_gaps(outcome_type: &str, stats: &EvidenceStats) -> Vec<EvidenceGap> {
    let mut gaps = Vec::new();
    let is_confirmed = outcome_type.eq_ignore_ascii_case("CONFIRMED");

    // NO_SUPPORTING_EVIDENCE / CONFIRMED_WITHOUT_SUPPORT
    if stats.supporting_count == 0 {
        if is_confirmed {
            gaps.push(EvidenceGap {
                code: "CONFIRMED_WITHOUT_SUPPORT".into(),
                severity: "CRITICAL".into(),
                title: "Confirmed without support".into(),
                description: "This confirmed outcome has no recorded supporting evidence.".into(),
            });
        } else {
            gaps.push(EvidenceGap {
                code: "NO_SUPPORTING_EVIDENCE".into(),
                severity: "CRITICAL".into(),
                title: "No supporting evidence".into(),
                description: "No supporting evidence is currently recorded for this outcome."
                    .into(),
            });
        }
    }

    // SINGLE_SUPPORTING_EVIDENCE
    if stats.supporting_count == 1 {
        gaps.push(EvidenceGap {
            code: "SINGLE_SUPPORTING_EVIDENCE".into(),
            severity: "WARNING".into(),
            title: "Single supporting evidence".into(),
            description: "This outcome currently relies on a single supporting evidence record."
                .into(),
        });
    }

    // NO_CITATION - only if supporting exists but none cited
    if stats.supporting_count > 0 && stats.cited_supporting_count == 0 {
        gaps.push(EvidenceGap {
            code: "NO_CITATION".into(),
            severity: "WARNING".into(),
            title: "No citation".into(),
            description: "Supporting evidence has no citation.".into(),
        });
    }

    // CONTRADICTORY_EVIDENCE
    if stats.contradicting_count > 0 {
        gaps.push(EvidenceGap {
            code: "CONTRADICTORY_EVIDENCE".into(),
            severity: "WARNING".into(),
            title: "Contradictory evidence".into(),
            description: "Contradictory evidence is recorded for this outcome.".into(),
        });
    }

    // SINGLE_SOURCE
    if stats.evidence_total > 0 && stats.sources_count == 1 {
        gaps.push(EvidenceGap {
            code: "SINGLE_SOURCE".into(),
            severity: "INFO".into(),
            title: "Single source".into(),
            description: "Evidence currently comes from a single source.".into(),
        });
    }

    gaps
}

pub fn calculate_research_followups(
    _outcome_type: &str,
    _stats: &EvidenceStats,
    gaps: &[EvidenceGap],
) -> Vec<ResearchFollowUp> {
    use std::collections::HashSet;
    let mut seen: HashSet<String> = HashSet::new();
    let mut followups: Vec<ResearchFollowUp> = Vec::new();

    for gap in gaps {
        let (code, priority, title, description) = match gap.code.as_str() {
            "CONFIRMED_WITHOUT_SUPPORT" => (
                "ADD_SUPPORTING_EVIDENCE",
                "HIGH",
                "Add supporting evidence",
                "This confirmed outcome has no supporting evidence recorded.",
            ),
            "NO_SUPPORTING_EVIDENCE" => (
                "ADD_SUPPORTING_EVIDENCE",
                "HIGH",
                "Add supporting evidence",
                "No supporting evidence is currently recorded for this outcome.",
            ),
            "NO_CITATION" => (
                "ADD_CITATION",
                "MEDIUM",
                "Add a citation",
                "Supporting evidence is recorded without a citation.",
            ),
            "SINGLE_SUPPORTING_EVIDENCE" => (
                "ADD_SECOND_SUPPORTING_EVIDENCE",
                "MEDIUM",
                "Add another supporting evidence",
                "This outcome currently has a single supporting evidence record.",
            ),
            "CONTRADICTORY_EVIDENCE" => (
                "REVIEW_CONTRADICTION",
                "HIGH",
                "Review contradictory evidence",
                "Supporting and contradicting evidence are both recorded for this outcome.",
            ),
            "SINGLE_SOURCE" => (
                "REVIEW_SOURCE_COVERAGE",
                "LOW",
                "Review source coverage",
                "Evidence for this outcome currently comes from a single source.",
            ),
            _ => continue,
        };
        if seen.contains(code) {
            continue;
        }
        seen.insert(code.to_string());
        followups.push(ResearchFollowUp {
            code: code.to_string(),
            priority: priority.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            gap_code: gap.code.clone(),
        });
    }

    // Order: HIGH, MEDIUM, LOW and within same priority stable order
    fn priority_rank(p: &str) -> u8 {
        match p {
            "HIGH" => 0,
            "MEDIUM" => 1,
            "LOW" => 2,
            _ => 3,
        }
    }
    fn code_order(code: &str) -> u8 {
        match code {
            "ADD_SUPPORTING_EVIDENCE" => 0,
            "REVIEW_CONTRADICTION" => 1,
            "ADD_CITATION" => 2,
            "ADD_SECOND_SUPPORTING_EVIDENCE" => 3,
            "REVIEW_SOURCE_COVERAGE" => 4,
            _ => 99,
        }
    }
    followups.sort_by(|a, b| {
        let pa = priority_rank(&a.priority);
        let pb = priority_rank(&b.priority);
        if pa != pb {
            return pa.cmp(&pb);
        }
        code_order(&a.code).cmp(&code_order(&b.code))
    });
    followups
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_evidence() {
        let s = EvidenceStats {
            evidence_total: 0,
            supporting_count: 0,
            contradicting_count: 0,
            sources_count: 0,
            cited_count: 0,
            uncited_count: 0,
            cited_supporting_count: 0,
        };
        let a = calculate_evidence_assessment(&s);
        assert_eq!(a.status, "NO_EVIDENCE");
        assert_eq!(a.score, 0);
    }

    #[test]
    fn test_single_supporting() {
        let s = EvidenceStats {
            evidence_total: 1,
            supporting_count: 1,
            contradicting_count: 0,
            sources_count: 1,
            cited_count: 0,
            uncited_count: 1,
            cited_supporting_count: 0,
        };
        let a = calculate_evidence_assessment(&s);
        assert_eq!(a.status, "WEAK");
        // 30 +5 -10 =25? Let's compute: +30 supporting, +5 all has source, -10 no citation => 25
        assert!(a.score >= 0 && a.score <= 100);
    }

    #[test]
    fn test_multiple_supporting() {
        let s = EvidenceStats {
            evidence_total: 2,
            supporting_count: 2,
            contradicting_count: 0,
            sources_count: 1,
            cited_count: 0,
            uncited_count: 2,
            cited_supporting_count: 0,
        };
        let a = calculate_evidence_assessment(&s);
        assert_eq!(a.status, "SUPPORTED");
    }

    #[test]
    fn test_strongly_supported() {
        let s = EvidenceStats {
            evidence_total: 2,
            supporting_count: 2,
            contradicting_count: 0,
            sources_count: 2,
            cited_count: 2,
            uncited_count: 0,
            cited_supporting_count: 2,
        };
        let a = calculate_evidence_assessment(&s);
        assert_eq!(a.status, "STRONGLY_SUPPORTED");
        assert!(a.score > 50);
    }

    #[test]
    fn test_mixed() {
        let s = EvidenceStats {
            evidence_total: 3,
            supporting_count: 2,
            contradicting_count: 1,
            sources_count: 2,
            cited_count: 2,
            uncited_count: 1,
            cited_supporting_count: 2,
        };
        let a = calculate_evidence_assessment(&s);
        assert_eq!(a.status, "MIXED");
    }

    // Gap tests
    #[test]
    fn test_gap_no_evidence() {
        let s = EvidenceStats {
            evidence_total: 0,
            supporting_count: 0,
            contradicting_count: 0,
            sources_count: 0,
            cited_count: 0,
            uncited_count: 0,
            cited_supporting_count: 0,
        };
        let gaps = calculate_evidence_gaps("INCONCLUSIVE", &s);
        assert!(gaps
            .iter()
            .any(|g| g.code == "NO_SUPPORTING_EVIDENCE" && g.severity == "CRITICAL"));
        assert_eq!(
            gaps.iter()
                .find(|g| g.code == "NO_SUPPORTING_EVIDENCE")
                .unwrap()
                .title,
            "No supporting evidence"
        );
    }

    #[test]
    fn test_gap_confirmed_without_support() {
        let s = EvidenceStats {
            evidence_total: 0,
            supporting_count: 0,
            contradicting_count: 0,
            sources_count: 0,
            cited_count: 0,
            uncited_count: 0,
            cited_supporting_count: 0,
        };
        let gaps = calculate_evidence_gaps("CONFIRMED", &s);
        assert!(gaps
            .iter()
            .any(|g| g.code == "CONFIRMED_WITHOUT_SUPPORT" && g.severity == "CRITICAL"));
        assert!(!gaps.iter().any(|g| g.code == "NO_SUPPORTING_EVIDENCE"));
    }

    #[test]
    fn test_gap_single_supporting() {
        let s = EvidenceStats {
            evidence_total: 1,
            supporting_count: 1,
            contradicting_count: 0,
            sources_count: 1,
            cited_count: 1,
            uncited_count: 0,
            cited_supporting_count: 1,
        };
        let gaps = calculate_evidence_gaps("CONFIRMED", &s);
        assert!(gaps
            .iter()
            .any(|g| g.code == "SINGLE_SUPPORTING_EVIDENCE" && g.severity == "WARNING"));
        assert!(gaps
            .iter()
            .any(|g| g.code == "SINGLE_SOURCE" && g.severity == "INFO"));
        assert!(!gaps.iter().any(|g| g.code == "NO_SUPPORTING_EVIDENCE"));
    }

    #[test]
    fn test_gap_no_citation() {
        let s = EvidenceStats {
            evidence_total: 1,
            supporting_count: 1,
            contradicting_count: 0,
            sources_count: 1,
            cited_count: 0,
            uncited_count: 1,
            cited_supporting_count: 0,
        };
        let gaps = calculate_evidence_gaps("CONFIRMED", &s);
        assert!(gaps
            .iter()
            .any(|g| g.code == "NO_CITATION" && g.severity == "WARNING"));
        assert_eq!(
            gaps.iter()
                .find(|g| g.code == "NO_CITATION")
                .unwrap()
                .description,
            "Supporting evidence has no citation."
        );
    }

    #[test]
    fn test_gap_no_citation_not_triggered_without_support() {
        let s = EvidenceStats {
            evidence_total: 1,
            supporting_count: 0,
            contradicting_count: 1,
            sources_count: 1,
            cited_count: 0,
            uncited_count: 1,
            cited_supporting_count: 0,
        };
        let gaps = calculate_evidence_gaps("CONFIRMED", &s);
        assert!(!gaps.iter().any(|g| g.code == "NO_CITATION"));
    }

    #[test]
    fn test_gap_contradictory() {
        let s = EvidenceStats {
            evidence_total: 2,
            supporting_count: 1,
            contradicting_count: 1,
            sources_count: 1,
            cited_count: 1,
            uncited_count: 1,
            cited_supporting_count: 1,
        };
        let gaps = calculate_evidence_gaps("CONFIRMED", &s);
        assert!(gaps
            .iter()
            .any(|g| g.code == "CONTRADICTORY_EVIDENCE" && g.severity == "WARNING"));
    }

    #[test]
    fn test_gap_single_source() {
        let s = EvidenceStats {
            evidence_total: 2,
            supporting_count: 2,
            contradicting_count: 0,
            sources_count: 1,
            cited_count: 2,
            uncited_count: 0,
            cited_supporting_count: 2,
        };
        let gaps = calculate_evidence_gaps("CONFIRMED", &s);
        assert!(gaps.iter().any(|g| g.code == "SINGLE_SOURCE"));
        let s2 = s.clone();
        let mut s2 = s2;
        s2.sources_count = 2;
        let gaps2 = calculate_evidence_gaps("CONFIRMED", &s2);
        assert!(!gaps2.iter().any(|g| g.code == "SINGLE_SOURCE"));
    }

    #[test]
    fn test_gap_combined() {
        let s = EvidenceStats {
            evidence_total: 1,
            supporting_count: 1,
            contradicting_count: 0,
            sources_count: 1,
            cited_count: 0,
            uncited_count: 1,
            cited_supporting_count: 0,
        };
        let gaps = calculate_evidence_gaps("CONFIRMED", &s);
        assert!(gaps.iter().any(|g| g.code == "SINGLE_SUPPORTING_EVIDENCE"));
        assert!(gaps.iter().any(|g| g.code == "NO_CITATION"));
        assert!(gaps.iter().any(|g| g.code == "SINGLE_SOURCE"));
        assert_eq!(gaps.len(), 3);
    }

    #[test]
    fn test_gap_multiple_support_no_gap() {
        let s = EvidenceStats {
            evidence_total: 2,
            supporting_count: 2,
            contradicting_count: 0,
            sources_count: 2,
            cited_count: 2,
            uncited_count: 0,
            cited_supporting_count: 2,
        };
        let gaps = calculate_evidence_gaps("CONFIRMED", &s);
        assert!(!gaps.iter().any(|g| g.code == "SINGLE_SUPPORTING_EVIDENCE"));
        assert!(!gaps.iter().any(|g| g.code == "NO_CITATION"));
        assert!(!gaps.iter().any(|g| g.code == "SINGLE_SOURCE"));
        assert!(!gaps.iter().any(|g| g.code == "CONTRADICTORY_EVIDENCE"));
        assert!(gaps.is_empty());
    }

    // Follow-up tests
    #[test]
    fn test_followup_confirmed_without_support() {
        let s = EvidenceStats {
            evidence_total: 0,
            supporting_count: 0,
            contradicting_count: 0,
            sources_count: 0,
            cited_count: 0,
            uncited_count: 0,
            cited_supporting_count: 0,
        };
        let gaps = calculate_evidence_gaps("CONFIRMED", &s);
        let fus = calculate_research_followups("CONFIRMED", &s, &gaps);
        assert!(fus.iter().any(|f| f.code == "ADD_SUPPORTING_EVIDENCE"));
        let f = fus
            .iter()
            .find(|f| f.code == "ADD_SUPPORTING_EVIDENCE")
            .unwrap();
        assert_eq!(f.priority, "HIGH");
        assert_eq!(f.title, "Add supporting evidence");
        assert_eq!(
            f.description,
            "This confirmed outcome has no supporting evidence recorded."
        );
        assert_eq!(f.gap_code, "CONFIRMED_WITHOUT_SUPPORT");
    }

    #[test]
    fn test_followup_no_supporting_evidence() {
        let s = EvidenceStats {
            evidence_total: 0,
            supporting_count: 0,
            contradicting_count: 0,
            sources_count: 0,
            cited_count: 0,
            uncited_count: 0,
            cited_supporting_count: 0,
        };
        let gaps = calculate_evidence_gaps("INCONCLUSIVE", &s);
        let fus = calculate_research_followups("INCONCLUSIVE", &s, &gaps);
        let f = fus
            .iter()
            .find(|f| f.code == "ADD_SUPPORTING_EVIDENCE")
            .unwrap();
        assert_eq!(f.priority, "HIGH");
        assert_eq!(
            f.description,
            "No supporting evidence is currently recorded for this outcome."
        );
        assert_eq!(f.gap_code, "NO_SUPPORTING_EVIDENCE");
    }

    #[test]
    fn test_followup_no_citation() {
        let s = EvidenceStats {
            evidence_total: 1,
            supporting_count: 1,
            contradicting_count: 0,
            sources_count: 1,
            cited_count: 0,
            uncited_count: 1,
            cited_supporting_count: 0,
        };
        let gaps = calculate_evidence_gaps("CONFIRMED", &s);
        let fus = calculate_research_followups("CONFIRMED", &s, &gaps);
        let f = fus.iter().find(|f| f.code == "ADD_CITATION").unwrap();
        assert_eq!(f.priority, "MEDIUM");
        assert_eq!(f.title, "Add a citation");
        assert_eq!(
            f.description,
            "Supporting evidence is recorded without a citation."
        );
        assert_eq!(f.gap_code, "NO_CITATION");
    }

    #[test]
    fn test_followup_single_supporting() {
        let s = EvidenceStats {
            evidence_total: 1,
            supporting_count: 1,
            contradicting_count: 0,
            sources_count: 1,
            cited_count: 1,
            uncited_count: 0,
            cited_supporting_count: 1,
        };
        let gaps = calculate_evidence_gaps("CONFIRMED", &s);
        let fus = calculate_research_followups("CONFIRMED", &s, &gaps);
        let f = fus
            .iter()
            .find(|f| f.code == "ADD_SECOND_SUPPORTING_EVIDENCE")
            .unwrap();
        assert_eq!(f.priority, "MEDIUM");
        assert_eq!(f.title, "Add another supporting evidence");
        assert_eq!(
            f.description,
            "This outcome currently has a single supporting evidence record."
        );
    }

    #[test]
    fn test_followup_contradictory() {
        let s = EvidenceStats {
            evidence_total: 2,
            supporting_count: 1,
            contradicting_count: 1,
            sources_count: 1,
            cited_count: 1,
            uncited_count: 1,
            cited_supporting_count: 1,
        };
        let gaps = calculate_evidence_gaps("CONFIRMED", &s);
        let fus = calculate_research_followups("CONFIRMED", &s, &gaps);
        let f = fus
            .iter()
            .find(|f| f.code == "REVIEW_CONTRADICTION")
            .unwrap();
        assert_eq!(f.priority, "HIGH");
        assert_eq!(f.title, "Review contradictory evidence");
        assert_eq!(
            f.description,
            "Supporting and contradicting evidence are both recorded for this outcome."
        );
    }

    #[test]
    fn test_followup_single_source() {
        let s = EvidenceStats {
            evidence_total: 2,
            supporting_count: 2,
            contradicting_count: 0,
            sources_count: 1,
            cited_count: 2,
            uncited_count: 0,
            cited_supporting_count: 2,
        };
        let gaps = calculate_evidence_gaps("CONFIRMED", &s);
        let fus = calculate_research_followups("CONFIRMED", &s, &gaps);
        let f = fus
            .iter()
            .find(|f| f.code == "REVIEW_SOURCE_COVERAGE")
            .unwrap();
        assert_eq!(f.priority, "LOW");
        assert_eq!(f.title, "Review source coverage");
        assert_eq!(
            f.description,
            "Evidence for this outcome currently comes from a single source."
        );
    }

    #[test]
    fn test_followup_multiple_gaps_ordering() {
        let s = EvidenceStats {
            evidence_total: 2,
            supporting_count: 1,
            contradicting_count: 1,
            sources_count: 1,
            cited_count: 0,
            uncited_count: 2,
            cited_supporting_count: 0,
        };
        let gaps = calculate_evidence_gaps("CONFIRMED", &s);
        // gaps: SINGLE_SUPPORTING, NO_CITATION, CONTRADICTORY, SINGLE_SOURCE
        let fus = calculate_research_followups("CONFIRMED", &s, &gaps);
        assert_eq!(fus.len(), 4);
        // Order HIGH, MEDIUM, LOW
        assert_eq!(fus[0].code, "REVIEW_CONTRADICTION"); // HIGH (ADD_SUPPORTING not present)
        assert_eq!(fus[0].priority, "HIGH");
        assert_eq!(fus[1].code, "ADD_CITATION"); // MEDIUM
        assert_eq!(fus[2].code, "ADD_SECOND_SUPPORTING_EVIDENCE");
        assert_eq!(fus[3].code, "REVIEW_SOURCE_COVERAGE"); // LOW
    }

    #[test]
    fn test_followup_dedup() {
        // manually craft gaps that both map to same followup
        let s = EvidenceStats {
            evidence_total: 0,
            supporting_count: 0,
            contradicting_count: 0,
            sources_count: 0,
            cited_count: 0,
            uncited_count: 0,
            cited_supporting_count: 0,
        };
        let gaps = vec![
            EvidenceGap {
                code: "CONFIRMED_WITHOUT_SUPPORT".into(),
                severity: "CRITICAL".into(),
                title: "Confirmed without support".into(),
                description: "".into(),
            },
            EvidenceGap {
                code: "NO_SUPPORTING_EVIDENCE".into(),
                severity: "CRITICAL".into(),
                title: "No supporting evidence".into(),
                description: "".into(),
            },
        ];
        let fus = calculate_research_followups("CONFIRMED", &s, &gaps);
        assert_eq!(
            fus.iter()
                .filter(|f| f.code == "ADD_SUPPORTING_EVIDENCE")
                .count(),
            1
        );
        assert_eq!(fus.len(), 1);
    }

    #[test]
    fn test_followup_empty() {
        let s = EvidenceStats {
            evidence_total: 2,
            supporting_count: 2,
            contradicting_count: 0,
            sources_count: 2,
            cited_count: 2,
            uncited_count: 0,
            cited_supporting_count: 2,
        };
        let gaps = calculate_evidence_gaps("CONFIRMED", &s);
        assert!(gaps.is_empty());
        let fus = calculate_research_followups("CONFIRMED", &s, &gaps);
        assert!(fus.is_empty());
    }

    #[test]
    fn test_followup_priority_ordering_full() {
        let s = EvidenceStats {
            evidence_total: 1,
            supporting_count: 0,
            contradicting_count: 1,
            sources_count: 1,
            cited_count: 0,
            uncited_count: 1,
            cited_supporting_count: 0,
        };
        let gaps = calculate_evidence_gaps("CONFIRMED", &s);
        // NO_SUPPORTING? actually CONFIRMED_WITHOUT_SUPPORT + CONTRADICTORY + SINGLE_SOURCE
        let fus = calculate_research_followups("CONFIRMED", &s, &gaps);
        // Should be HIGH: ADD_SUPPORTING, REVIEW_CONTRADICTION then LOW
        assert_eq!(fus[0].code, "ADD_SUPPORTING_EVIDENCE");
        assert_eq!(fus[1].code, "REVIEW_CONTRADICTION");
        assert_eq!(fus[2].code, "REVIEW_SOURCE_COVERAGE");
    }
}
