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

export interface ApiErrorBody { error:{code:string; message:string} }
