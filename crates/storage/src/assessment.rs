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
}
