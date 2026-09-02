export type Severity = "low"|"info"|"medium"|"warning"|"high"|"critical";
export type Priority = Severity;
export type Researchability = "low"|"medium"|"high";

export interface Paginated<T> {
  items: T[];
  pagination: { limit:number; offset:number; total:number };
}

export interface TreeSummary {
  id:number; name:string; source_filename?:string; gedcom_version?:string;
  created_at:string; updated_at:string;
  persons:number; families:number; findings:number; research_opportunities:number;
}

export interface Person {
  id:number; tree_id:number; gedcom_id:string;
  given_name?:string; surname?:string; display_name?:string; sex?:string; raw_name?:string;
  birth_date_original?:string; birth_date_precision?:string; birth_date_year?:number;
  birth_place?:string; death_date_original?:string; death_place?:string; occupation?:string;
  raw_tags?: unknown;
}

export interface Family {
  id:number; tree_id:number; gedcom_id:string;
  members: { husband:number[]; wife:number[]; children:number[] };
  raw_tags?: unknown;
}

export interface Finding {
  id:number; tree_id:number; person_id?:number; related_person_id?:number;
  finding_type:string; severity:Severity; confidence?:number; message?:string; evidence?:unknown; created_at:string;
}

export interface ScoreComponent { name:string; points:number; reason:string; }
export interface ScoreBreakdown { total:number; components: ScoreComponent[]; }

export interface ResearchOpportunity {
  id:number; tree_id:number; person_id:number;
  priority:Priority; score:number; confidence:number; researchability:Researchability;
  why?:string; what?: unknown; potential_sources?: unknown; breakdown?: ScoreBreakdown;
  missing_information?: unknown; reasons?: unknown;
}

export interface Branch {
  id:number; tree_id:number; name:string; branch?:string;
  score:number; branch_score?:number; opportunity_count:number; high_priority_count:number;
  deepest_generation:number; source_coverage:number;
}

export interface SourceCoverage {
  tree_id:number; birth:number; marriage:number; death:number; other_events:number; overall:number;
}

export interface AnalysisRun {
  id:number; tree_id:number; started_at:string; completed_at?:string; engine_version?:string; status:string;
}

export type TaskStatus = "OPEN" | "IN_PROGRESS" | "RESOLVED" | "REJECTED" | "INCONCLUSIVE";

export interface ResearchTask {
  id:number; tree_id:number; opportunity_id?:number|null; person_id?:number|null;
  title:string; description?:string|null; status:TaskStatus;
  created_at:string; updated_at:string; started_at?:string|null; completed_at?:string|null; resolution?:string|null;
  outcome?: ResearchOutcome | null;
  has_outcome?: boolean;
  opportunity?: { id:number; score?:number; priority?:string; why?:string } | null;
}

export type OutcomeType = "CONFIRMED" | "FALSE_LEAD" | "INCONCLUSIVE" | "NEW_LEAD" | "NO_EVIDENCE";

export type AssessmentStatus = "NO_EVIDENCE" | "WEAK" | "MIXED" | "SUPPORTED" | "STRONGLY_SUPPORTED";

export interface EvidenceAssessmentReason {
  code:string; points:number; message:string;
}

export interface EvidenceAssessment {
  score:number; status:AssessmentStatus;
  evidence_total:number; supporting_count:number; contradicting_count:number;
  sources_count:number; cited_count:number; uncited_count:number;
  reasons: EvidenceAssessmentReason[];
}

export interface EvidenceStats {
  evidence_total:number; supporting_count:number; contradicting_count:number;
  sources_count:number; cited_count:number; uncited_count:number; cited_supporting_count:number;
}

export type GapCode = "NO_SUPPORTING_EVIDENCE" | "NO_CITATION" | "SINGLE_SUPPORTING_EVIDENCE" | "CONTRADICTORY_EVIDENCE" | "SINGLE_SOURCE" | "CONFIRMED_WITHOUT_SUPPORT";
export type GapSeverity = "INFO" | "WARNING" | "CRITICAL";
export interface EvidenceGap {
  code: GapCode; severity: GapSeverity; title: string; description: string;
}

export type FollowUpCode = "ADD_SUPPORTING_EVIDENCE" | "ADD_CITATION" | "REVIEW_CONTRADICTION" | "ADD_SECOND_SUPPORTING_EVIDENCE" | "REVIEW_SOURCE_COVERAGE";
export type FollowUpPriority = "HIGH" | "MEDIUM" | "LOW";
export interface ResearchFollowUp {
  code: FollowUpCode; priority: FollowUpPriority; title: string; description: string; gap_code: GapCode;
}

export interface ResearchOutcome {
  id:number; tree_id:number; task_id:number; type:OutcomeType; summary:string; details?:string|null;
  created_at:string; updated_at:string;
  evidence?: EvidenceWithRelationship[];
  evidence_assessment?: EvidenceAssessment | null;
  evidence_gaps?: EvidenceGap[];
  research_followups?: ResearchFollowUp[];
  followup_actions?: ResearchFollowupAction[];
  followup_actions_count?: number;
}

export type FollowupActionStatus = "OPEN" | "COMPLETED" | "SKIPPED";
export interface ResearchFollowupAction {
  id:number; tree_id:number; task_id:number; outcome_id:number; followup_code:FollowUpCode; status:FollowupActionStatus; notes?:string|null;
  created_at:string; updated_at:string; completed_at?:string|null;
}

export type SourceType = "BOOK"|"REGISTER"|"CENSUS"|"CIVIL_RECORD"|"PARISH_RECORD"|"NEWSPAPER"|"WEBSITE"|"OTHER";
export interface ResearchSource {
  id:number; tree_id:number; title:string; author?:string|null; publication?:string|null; date?:string|null; type:SourceType;
  created_at:string; updated_at:string;
}
export interface ResearchCitation {
  id:number; source_id:number; locator?:string|null; text?:string|null;
  created_at:string; updated_at:string;
}
export type EvidenceRelationship = "SUPPORTS"|"CONTRADICTS";
export interface Evidence {
  id:number; tree_id:number; source_id:number; citation_id?:number|null; statement:string; notes?:string|null;
  created_at:string; updated_at:string;
  source?: {id:number; title:string; type:SourceType};
  citation?: {id:number; locator?:string|null; text?:string|null} | null;
}
export interface EvidenceWithRelationship {
  id:number; relationship:EvidenceRelationship; statement:string; notes?:string|null;
  source:{id:number; title:string; type:SourceType; author?:string|null; publication?:string|null; date?:string|null};
  citation?:{id:number; locator?:string|null; text?:string|null} | null;
  created_at:string; updated_at:string;
}

export interface ApiErrorBody { error:{code:string; message:string} }
