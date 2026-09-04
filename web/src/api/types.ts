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
  session_id?: number | null;
  session?: { id:number; title:string; status:ResearchSessionStatus } | null;
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

export type ClosureWarningCode = "RESOLVED_WITHOUT_OUTCOME" | "CONFIRMED_WITHOUT_SUPPORT" | "RESOLVED_WITH_EVIDENCE_GAPS" | "REJECTED_WITH_CONFIRMED_OUTCOME" | "INCONCLUSIVE_WITH_CONFIRMED_OUTCOME";
export type ClosureWarningSeverity = "INFO" | "WARNING" | "CRITICAL";
export interface ResearchCaseClosureWarning { code:ClosureWarningCode; severity:ClosureWarningSeverity; title:string; description:string; }
export type TimelineEventType = "TASK_CREATED" | "TASK_STARTED" | "OUTCOME_CREATED" | "OUTCOME_UPDATED" | "FOLLOWUP_ACTION_CREATED" | "FOLLOWUP_ACTION_COMPLETED" | "TASK_COMPLETED";
export interface ResearchCaseTimelineEvent { event_type:TimelineEventType; timestamp:string; label:string; }
export interface ResearchCaseSummary {
  task: { id:number; title:string; description?:string|null; status:TaskStatus; resolution?:string|null; created_at:string; started_at?:string|null; completed_at?:string|null; updated_at:string; tree_id?:number; };
  person: { person_id:number; person_name:string } | null;
  opportunity: { opportunity_id:number; score?:number|null; priority?:string|null; researchability?:string|null; confidence?:number|null; title?:string|null } | null;
  outcome: { outcome_id:number; type:OutcomeType; summary:string; details?:string|null; created_at:string; updated_at:string } | null;
  evidence_assessment: EvidenceAssessment | null;
  evidence_gaps: EvidenceGap[];
  research_followups: ResearchFollowUp[];
  followup_actions: ResearchFollowupAction[];
  timeline: ResearchCaseTimelineEvent[];
  closure_warnings: ResearchCaseClosureWarning[];
}

export interface ResearchPlanningReason { code:string; label:string; description:string; }
export interface ResearchPlanItem {
  opportunity_id:number; person_id:number; title:string;
  priority:string; research_score:number; planning_score:number;
  researchability:string; confidence:number;
  active_task:boolean; task_status?:string|null;
  reasons: ResearchPlanningReason[];
}
export interface ResearchPlanSummary {
  total_candidates:number; recommended_count:number; deferred_count:number;
  active_count:number; inconclusive_count:number; high_priority_count:number; critical_gap_count:number;
}
export interface ResearchPlan {
  generated_at:string; total_candidates:number; summary:ResearchPlanSummary;
  recommended: ResearchPlanItem[]; deferred: ResearchPlanItem[];
}

export type ResearchSessionStatus = "PLANNED" | "ACTIVE" | "COMPLETED" | "ABANDONED";
export interface ResearchSessionStats {
  total_tasks:number; completed_tasks:number; open_tasks:number; in_progress_tasks:number; inconclusive_tasks:number; rejected_tasks:number;
  total_outcomes:number; confirmed_outcomes:number; false_lead_outcomes:number; inconclusive_outcomes:number; new_lead_outcomes:number; no_evidence_outcomes:number;
  total_evidence:number; supporting_evidence:number; contradicting_evidence:number;
  open_followups:number; completed_followup_actions:number; skipped_followup_actions:number;
}
export interface ResearchSessionTimelineEvent {
  event_type:string; timestamp:string; label:string;
}
export interface ResearchSession {
  id:number; tree_id:number; title:string; description?:string|null; status:ResearchSessionStatus;
  person_id?:number|null; opportunity_id?:number|null;
  created_at:string; updated_at:string; started_at?:string|null; completed_at?:string|null;
  person?: {id:number; name:string; gedcom_id:string} | null;
  opportunity?: {id:number; title:string; priority?:string|null; score?:number|null; person_id:number} | null;
  tasks?: ResearchTask[];
  summary?: ResearchSessionSummary;
  stats?: ResearchSessionStats;
}
export interface ResearchSessionSummary {
  total_tasks:number; open_tasks:number; in_progress_tasks:number; terminal_tasks:number; outcomes_count:number;
}
export interface ResearchSessionDetail {
  session: ResearchSession;
  person: {id:number; name:string; gedcom_id:string} | null;
  opportunity: {id:number; title:string; priority?:string|null; score?:number|null; person_id:number} | null;
  tasks: ResearchTask[];
  summary: ResearchSessionSummary;
  stats: ResearchSessionStats;
  timeline: ResearchSessionTimelineEvent[];
}

export type ResearchQueryStatus = "PENDING"|"RUNNING"|"COMPLETED"|"FAILED";
export interface ResearchQuery {
  id:number; tree_id:number; task_id:number; provider:string; query:string; status:ResearchQueryStatus;
  created_at:string; started_at?:string|null; completed_at?:string|null;
  error_code?:string|null; error_message?:string|null;
  latest_execution?: ResearchQueryExecution | null;
}
export interface ResearchQueryExecution {
  id:number; query_id:number; status:ResearchQueryStatus;
  started_at?:string|null; completed_at?:string|null;
  error_code?:string|null; error_message?:string|null;
  provider_request_id?:string|null; provider_metadata?: unknown | null;
  created_at:string; result_count?: number;
}
export interface ResearchResult {
  id:number; execution_id:number; query_id:number; provider:string;
  external_id?:string|null; title:string; description?:string|null; url?:string|null;
  record_type?:string|null; date?:string|null; place?:string|null;
  metadata?: unknown; position:number; created_at:string;
}
export interface ResearchProviderInfo {
  name:string; display_name:string; configured:boolean; enabled:boolean; status:string; requires_auth:boolean;
}

export interface ResearchSummary {
  opportunities:{high:number;medium:number;low:number};
  tasks:{open:number;in_progress:number;resolved:number;rejected:number;inconclusive:number};
  outcomes:{total:number;confirmed?:number;false_lead?:number;inconclusive?:number;new_lead?:number;no_evidence?:number};
  sources?:{total:number};
  evidence?:{total:number;supporting:number;contradicting:number};
  assessment?:{no_evidence:number;weak:number;mixed:number;supported:number;strongly_supported:number};
  evidence_gaps?:{critical:number;warning:number;info:number};
  research_followups?:{high:number;medium:number;low:number};
  followup_actions?:{open:number;completed:number;skipped:number};
  sessions?:{total:number;active:number;planned:number;completed:number;abandoned:number};
  external_research?:{queries:number;executions:number;successful:number;failed:number;pending:number;results:number};
  research_activity?:{
    tasks:{open:number;in_progress:number;resolved:number;rejected:number;inconclusive:number;total:number};
    outcomes:{total:number;confirmed:number;false_lead:number;inconclusive:number;new_lead:number;no_evidence:number};
    evidence:{total:number;supporting:number;contradicting:number};
    followups:{open:number;completed:number;skipped:number;total:number};
  };
}

export interface ApiErrorBody { error:{code:string; message:string} }
