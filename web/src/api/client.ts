const BASE = (import.meta.env.VITE_API_BASE_URL as string) || "http://127.0.0.1:3000";

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
  return res.json() as Promise<T>;
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
};
