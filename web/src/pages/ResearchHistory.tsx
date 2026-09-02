import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { api } from "../api/client";
import { Loading, ErrorState, Empty, Pagination } from "../components/common";

function formatOutcomeType(t:string){
  const map:any={"CONFIRMED":"Confirmed","FALSE_LEAD":"False lead","INCONCLUSIVE":"Inconclusive","NEW_LEAD":"New lead","NO_EVIDENCE":"No evidence"};
  return map[t]||t;
}
function formatAssessmentStatus(s:string){
  return s.replace(/_/g," ");
}

export default function ResearchHistory(){
  const {treeId}=useParams(); const id=Number(treeId);
  const [type,setType]=useState("");
  const [personId,setPersonId]=useState("");
  const [assessmentStatus,setAssessmentStatus]=useState("");
  const [gap,setGap]=useState("");
  const [items,setItems]=useState<any[]>([]);
  const [total,setTotal]=useState(0);
  const [offset,setOffset]=useState(0); const limit=20;
  const [err,setErr]=useState<string|null>(null); const [loading,setLoading]=useState(true);

  const load=async(o:number)=>{
    setLoading(true); setErr(null);
    try{
      const r=await api.getOutcomes(id,{
        type: type||undefined,
        person_id: personId?Number(personId):undefined,
        assessment_status: assessmentStatus||undefined,
        gap: gap||undefined,
        limit, offset:o
      });
      setItems(r.items); setTotal(r.pagination.total); setOffset(o);
    }catch(e:any){setErr(e.message)} finally{setLoading(false)}
  };
  useEffect(()=>{load(0)},[id, type, personId, assessmentStatus, gap]);

  return <div className="space-y-4">
    <div className="flex justify-between items-center">
      <h1 className="text-2xl font-bold">Research History</h1>
      <Link to={`/trees/${id}/research`} className="text-sm text-blue-600 underline">← Research Overview</Link>
    </div>
    <p className="text-sm text-gray-600">Historical outcomes — ordered by creation date.</p>
    <div className="flex gap-2 flex-wrap">
      <select value={type} onChange={e=>setType(e.target.value)} className="border rounded px-2 py-1">
        <option value="">All types</option>
        <option value="CONFIRMED">Confirmed</option>
        <option value="FALSE_LEAD">False lead</option>
        <option value="INCONCLUSIVE">Inconclusive</option>
        <option value="NEW_LEAD">New lead</option>
        <option value="NO_EVIDENCE">No evidence</option>
      </select>
      <select value={assessmentStatus} onChange={e=>setAssessmentStatus(e.target.value)} className="border rounded px-2 py-1">
        <option value="">All</option>
        <option value="NO_EVIDENCE">No Evidence</option>
        <option value="WEAK">Weak</option>
        <option value="MIXED">Mixed</option>
        <option value="SUPPORTED">Supported</option>
        <option value="STRONGLY_SUPPORTED">Strongly Supported</option>
      </select>
      <select value={gap} onChange={e=>setGap(e.target.value)} className="border rounded px-2 py-1">
        <option value="">All gaps</option>
        <option value="NO_SUPPORTING_EVIDENCE">No supporting evidence</option>
        <option value="NO_CITATION">No citation</option>
        <option value="SINGLE_SUPPORTING_EVIDENCE">Single supporting</option>
        <option value="CONTRADICTORY_EVIDENCE">Contradictory</option>
        <option value="SINGLE_SOURCE">Single source</option>
        <option value="CONFIRMED_WITHOUT_SUPPORT">Confirmed without support</option>
      </select>
      <input placeholder="Person ID" value={personId} onChange={e=>setPersonId(e.target.value)} className="border rounded px-2 py-1 w-32" type="number" />
    </div>
    {loading ? <Loading msg="Loading history…" /> : err ? <ErrorState msg={err} onRetry={()=>load(offset)} /> : items.length===0 ? <Empty msg="No research history yet. Completed investigations will appear here." /> :
      <div className="space-y-2">
        <div className="hidden md:grid grid-cols-12 text-xs font-semibold text-gray-500 px-3">
          <span className="col-span-2">Date</span>
          <span className="col-span-1">Type</span>
          <span className="col-span-3">Summary</span>
          <span className="col-span-1">Task</span>
          <span className="col-span-2">Evidence</span>
          <span className="col-span-3">Assessment / Gaps</span>
        </div>
        {items.map((o:any)=>{
          const a=o.evidence_assessment;
          const evidenceCount = a ? a.evidence_total : (o.evidence ? o.evidence.length : 0);
          const gaps = o.evidence_gaps || [];
          const gapLabel = gaps.length===0 ? "" : gaps.length===1 ? `Gaps: 1 ${gaps[0].severity.toLowerCase()}` : `Gaps: ${gaps.length}`;
          // alternative: count warnings
          const warningCount = gaps.filter((g:any)=>g.severity==="WARNING").length;
          const gapDisplay = gaps.length>0 ? (warningCount>0 && gaps.length===1 ? `Gaps: 1 warning` : gapLabel) : "";
          return <Link key={o.id} to={`/trees/${id}/research/tasks/${o.task_id}`} className="block border rounded p-3 hover:bg-gray-50">
          <div className="grid md:grid-cols-12 gap-1 text-sm">
            <span className="col-span-2 text-xs text-gray-600">{new Date(o.created_at).toLocaleDateString()}</span>
            <span className="col-span-1"><span className="px-2 py-1 bg-emerald-100 rounded text-xs font-semibold">{formatOutcomeType(o.type)}</span></span>
            <span className="col-span-3 font-medium">{o.summary}</span>
            <span className="col-span-1 text-blue-600 underline">Task {o.task_id}</span>
            <span className="col-span-2 text-gray-600">Evidence: {evidenceCount}</span>
            <span className="col-span-3 text-gray-700">
              {a ? `Assessment: ${formatAssessmentStatus(a.status)} · ${a.score}` : "—"}
              {gapDisplay && <span className="ml-2 text-xs">{gapDisplay}</span>}
            </span>
          </div>
          {o.details && <div className="text-xs text-gray-600 mt-1 whitespace-pre-wrap">{o.details}</div>}
          {gaps.length>0 && <div className="text-xs text-gray-500 mt-1">Gaps: {gaps.map((g:any)=>g.code).join(", ")}</div>}
        </Link>
        })}
        <Pagination limit={limit} offset={offset} total={total} onChange={load} />
      </div>
    }
  </div>
}
