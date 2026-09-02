const BASE = (import.meta.env.VITE_API_BASE_URL as string) || "";

export class ApiError extends Error {
  code:string; status:number;
  constructor(code:string, message:string, status:number){
    super(message); this.code=code; this.status=status;
  }
}
async function req<T>(path:string, init?:RequestInit): Promise<T> {
  const res = await fetch(`${BASE}${path}`, { headers:{ "Content-Type":"application/json"}, ...init });
  if(!res.ok){
    let body:any={}; try{ body=await res.json(); }catch{}
    const code = body?.error?.code || "HTTP_ERROR";
    const msg = body?.error?.message || res.statusText;
    throw new ApiError(code, msg, res.status);
  }
  if(res.status===204) return undefined as T;
  const text = await res.text();
  if(!text) return undefined as T;
  return JSON.parse(text) as T;
}
export const api = {
  getTrees: (params?:{limit?:number;offset?:number})=>{
    const q=new URLSearchParams();
    if(params?.limit!==undefined) q.set("limit",String(params.limit));
    if(params?.offset!==undefined) q.set("offset",String(params.offset));
    const s=q.toString()?`?${q}`:"";
    return req<import("./types").Paginated<import("./types").TreeSummary>>(`/api/v1/trees${s}`);
  },
  getTree: (id:number)=> req<import("./types").TreeSummary>(`/api/v1/trees/${id}`),
  getPersons: (treeId:number, params?:{limit?:number;offset?:number})=>{
    const q=new URLSearchParams();
    if(params?.limit!==undefined) q.set("limit",String(params.limit));
    if(params?.offset!==undefined) q.set("offset",String(params.offset));
    const s=q.toString()?`?${q}`:"";
    return req<import("./types").Paginated<import("./types").Person>>(`/api/v1/trees/${treeId}/persons${s}`);
  },
  getPerson: (treeId:number, personId:number)=> req<import("./types").Person>(`/api/v1/trees/${treeId}/persons/${personId}`),
  getFamilies: (treeId:number, params?:{limit?:number;offset?:number})=>{
    const q=new URLSearchParams();
    if(params?.limit!==undefined) q.set("limit",String(params.limit));
    if(params?.offset!==undefined) q.set("offset",String(params.offset));
    const s=q.toString()?`?${q}`:"";
    return req<import("./types").Paginated<import("./types").Family>>(`/api/v1/trees/${treeId}/families${s}`);
  },
  getFamily: (treeId:number, fid:number)=> req<import("./types").Family>(`/api/v1/trees/${treeId}/families/${fid}`),
  getFindings: (treeId:number, params?:{severity?:string;type?:string;person_id?:number;limit?:number;offset?:number})=>{
    const q=new URLSearchParams();
    if(params?.severity) q.set("severity",params.severity);
    if(params?.type) q.set("type",params.type);
    if(params?.person_id) q.set("person_id",String(params.person_id));
    if(params?.limit!==undefined) q.set("limit",String(params.limit));
    if(params?.offset!==undefined) q.set("offset",String(params.offset));
    const s=q.toString()?`?${q}`:"";
    return req<import("./types").Paginated<import("./types").Finding>>(`/api/v1/trees/${treeId}/findings${s}`);
  },
  getOpportunities: (treeId:number, params?:{priority?:string;min_score?:number;sort?:string;limit?:number;offset?:number})=>{
    const q=new URLSearchParams();
    if(params?.priority) q.set("priority",params.priority);
    if(params?.min_score!==undefined) q.set("min_score",String(params.min_score));
    if(params?.sort) q.set("sort",params.sort);
    if(params?.limit!==undefined) q.set("limit",String(params.limit));
    if(params?.offset!==undefined) q.set("offset",String(params.offset));
    const s=q.toString()?`?${q}`:"";
    return req<import("./types").Paginated<import("./types").ResearchOpportunity>>(`/api/v1/trees/${treeId}/research-opportunities${s}`);
  },
  getTop: (treeId:number, params?:{priority?:string;limit?:number})=>{
    const q=new URLSearchParams();
    if(params?.priority) q.set("priority",params.priority);
    if(params?.limit!==undefined) q.set("limit",String(params.limit));
    const s=q.toString()?`?${q}`:"";
    return req<{items: import("./types").ResearchOpportunity[]}>(`/api/v1/trees/${treeId}/research-opportunities/top${s}`);
  },
  getBranches: (treeId:number)=> req<{items: import("./types").Branch[]}>(`/api/v1/trees/${treeId}/branches`),
  getCoverage: (treeId:number)=> req<import("./types").SourceCoverage>(`/api/v1/trees/${treeId}/source-coverage`),
  getRuns: (treeId:number)=> req<{items: import("./types").AnalysisRun[]}>(`/api/v1/trees/${treeId}/analysis-runs`),
  // Research Tasks
  getTasks: (treeId:number, params?:{status?:string;person_id?:number;opportunity_id?:number;has_outcome?:boolean;limit?:number;offset?:number})=>{
    const q=new URLSearchParams();
    if(params?.status) q.set("status",params.status);
    if(params?.person_id) q.set("person_id",String(params.person_id));
    if(params?.opportunity_id) q.set("opportunity_id",String(params.opportunity_id));
    if(params?.has_outcome!==undefined) q.set("has_outcome",String(params.has_outcome));
    if(params?.limit!==undefined) q.set("limit",String(params.limit));
    if(params?.offset!==undefined) q.set("offset",String(params.offset));
    const s=q.toString()?`?${q}`:"";
    return req<import("./types").Paginated<import("./types").ResearchTask & {has_outcome?:boolean; opportunity?:{id:number;score?:number;priority?:string;why?:string}}>>(`/api/v1/trees/${treeId}/research-tasks${s}`);
  },
  getTask: (treeId:number, taskId:number)=> req<import("./types").ResearchTask>(`/api/v1/trees/${treeId}/research-tasks/${taskId}`),
  createTask: (treeId:number, body:{title:string;description?:string;person_id?:number;opportunity_id?:number})=> req<import("./types").ResearchTask>(`/api/v1/trees/${treeId}/research-tasks`,{method:"POST", body:JSON.stringify(body)}),
  createTaskFromOpportunity: (treeId:number, oppId:number, body?:{title?:string;description?:string})=> req<import("./types").ResearchTask>(`/api/v1/trees/${treeId}/research-opportunities/${oppId}/tasks`,{method:"POST", body:JSON.stringify(body||{})}),
  updateTask: (treeId:number, taskId:number, body:{title?:string;description?:string;status?:string;resolution?:string})=> req<import("./types").ResearchTask>(`/api/v1/trees/${treeId}/research-tasks/${taskId}`,{method:"PATCH", body:JSON.stringify(body)}),
  deleteTask: (treeId:number, taskId:number)=> req<void>(`/api/v1/trees/${treeId}/research-tasks/${taskId}`,{method:"DELETE"}),
  // Research Outcomes
  getOutcomes: (treeId:number, params?:{type?:string;task_id?:number;person_id?:number;assessment_status?:string;limit?:number;offset?:number})=>{
    const q=new URLSearchParams();
    if(params?.type) q.set("type",params.type);
    if(params?.task_id) q.set("task_id",String(params.task_id));
    if(params?.person_id) q.set("person_id",String(params.person_id));
    if(params?.assessment_status) q.set("assessment_status",params.assessment_status);
    if(params?.limit!==undefined) q.set("limit",String(params.limit));
    if(params?.offset!==undefined) q.set("offset",String(params.offset));
    const s=q.toString()?`?${q}`:"";
    return req<import("./types").Paginated<import("./types").ResearchOutcome>>(`/api/v1/trees/${treeId}/research-outcomes${s}`);
  },
  getOutcome: (treeId:number, outcomeId:number)=> req<import("./types").ResearchOutcome>(`/api/v1/trees/${treeId}/research-outcomes/${outcomeId}`),
  createOutcome: (treeId:number, taskId:number, body:{type:string;summary:string;details?:string})=> req<import("./types").ResearchOutcome>(`/api/v1/trees/${treeId}/research-tasks/${taskId}/outcome`,{method:"POST", body:JSON.stringify(body)}),
  updateOutcome: (treeId:number, outcomeId:number, body:{type?:string;summary?:string;details?:string})=> req<import("./types").ResearchOutcome>(`/api/v1/trees/${treeId}/research-outcomes/${outcomeId}`,{method:"PATCH", body:JSON.stringify(body)}),
  deleteOutcome: (treeId:number, outcomeId:number)=> req<void>(`/api/v1/trees/${treeId}/research-outcomes/${outcomeId}`,{method:"DELETE"}),
  getResearchSummary: (treeId:number)=> req<{opportunities:{high:number;medium:number;low:number}; tasks:{open:number;in_progress:number;resolved:number;rejected:number;inconclusive:number}; outcomes:{total:number}; sources?:{total:number}; evidence?:{total:number;supporting:number;contradicting:number}; assessment?:{no_evidence:number;weak:number;mixed:number;supported:number;strongly_supported:number}}>(`/api/v1/trees/${treeId}/research/summary`),
  // Sources
  getSources: (treeId:number, params?:{type?:string;limit?:number;offset?:number})=>{
    const q=new URLSearchParams();
    if(params?.type) q.set("type",params.type);
    if(params?.limit!==undefined) q.set("limit",String(params.limit));
    if(params?.offset!==undefined) q.set("offset",String(params.offset));
    const s=q.toString()?`?${q}`:"";
    return req<import("./types").Paginated<import("./types").ResearchSource>>(`/api/v1/trees/${treeId}/sources${s}`);
  },
  getSource: (treeId:number, sourceId:number)=> req<import("./types").ResearchSource>(`/api/v1/trees/${treeId}/sources/${sourceId}`),
  createSource: (treeId:number, body:{title:string;author?:string;publication?:string;date?:string;type:string})=> req<import("./types").ResearchSource>(`/api/v1/trees/${treeId}/sources`,{method:"POST", body:JSON.stringify(body)}),
  updateSource: (treeId:number, sourceId:number, body:{title?:string;author?:string;publication?:string;date?:string;type?:string})=> req<import("./types").ResearchSource>(`/api/v1/trees/${treeId}/sources/${sourceId}`,{method:"PATCH", body:JSON.stringify(body)}),
  deleteSource: (treeId:number, sourceId:number)=> req<void>(`/api/v1/trees/${treeId}/sources/${sourceId}`,{method:"DELETE"}),
  // Citations
  getCitations: (treeId:number, sourceId:number, params?:{limit?:number;offset?:number})=>{
    const q=new URLSearchParams();
    if(params?.limit!==undefined) q.set("limit",String(params.limit));
    if(params?.offset!==undefined) q.set("offset",String(params.offset));
    const s=q.toString()?`?${q}`:"";
    return req<import("./types").Paginated<import("./types").ResearchCitation>>(`/api/v1/trees/${treeId}/sources/${sourceId}/citations${s}`);
  },
  getCitation: (treeId:number, citationId:number)=> req<import("./types").ResearchCitation>(`/api/v1/trees/${treeId}/citations/${citationId}`),
  createCitation: (treeId:number, sourceId:number, body:{locator?:string;text?:string})=> req<import("./types").ResearchCitation>(`/api/v1/trees/${treeId}/sources/${sourceId}/citations`,{method:"POST", body:JSON.stringify(body)}),
  updateCitation: (treeId:number, citationId:number, body:{locator?:string;text?:string})=> req<import("./types").ResearchCitation>(`/api/v1/trees/${treeId}/citations/${citationId}`,{method:"PATCH", body:JSON.stringify(body)}),
  deleteCitation: (treeId:number, citationId:number)=> req<void>(`/api/v1/trees/${treeId}/citations/${citationId}`,{method:"DELETE"}),
  // Evidence
  getEvidenceList: (treeId:number, params?:{limit?:number;offset?:number})=>{
    const q=new URLSearchParams();
    if(params?.limit!==undefined) q.set("limit",String(params.limit));
    if(params?.offset!==undefined) q.set("offset",String(params.offset));
    const s=q.toString()?`?${q}`:"";
    return req<import("./types").Paginated<import("./types").Evidence>>(`/api/v1/trees/${treeId}/evidence${s}`);
  },
  getEvidence: (treeId:number, evidenceId:number)=> req<import("./types").Evidence>(`/api/v1/trees/${treeId}/evidence/${evidenceId}`),
  createEvidence: (treeId:number, body:{source_id:number;citation_id?:number|null;statement:string;notes?:string})=> req<import("./types").Evidence>(`/api/v1/trees/${treeId}/evidence`,{method:"POST", body:JSON.stringify(body)}),
  updateEvidence: (treeId:number, evidenceId:number, body:{statement?:string;notes?:string;citation_id?:number})=> req<import("./types").Evidence>(`/api/v1/trees/${treeId}/evidence/${evidenceId}`,{method:"PATCH", body:JSON.stringify(body)}),
  deleteEvidence: (treeId:number, evidenceId:number)=> req<void>(`/api/v1/trees/${treeId}/evidence/${evidenceId}`,{method:"DELETE"}),
  // Outcome Evidence
  getOutcomeEvidence: (treeId:number, outcomeId:number)=> req<{items: import("./types").EvidenceWithRelationship[]}>(`/api/v1/trees/${treeId}/research-outcomes/${outcomeId}/evidence`),
  attachEvidence: (treeId:number, outcomeId:number, evidenceId:number, body:{relationship:string})=> req<{outcome_id:number;evidence_id:number;relationship:string}>(`/api/v1/trees/${treeId}/research-outcomes/${outcomeId}/evidence/${evidenceId}`,{method:"POST", body:JSON.stringify(body)}),
  detachEvidence: (treeId:number, outcomeId:number, evidenceId:number)=> req<void>(`/api/v1/trees/${treeId}/research-outcomes/${outcomeId}/evidence/${evidenceId}`,{method:"DELETE"}),
};
