import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { api } from "../api/client";
import { Loading, ErrorState, Empty, Pagination } from "../components/common";
import { TaskStatusBadge } from "../components/Badges";



export default function ResearchTasks(){
  const {treeId}=useParams(); const id=Number(treeId);
  const [status,setStatus]=useState("");
  const [hasOutcome,setHasOutcome]=useState("");
  const [personId,setPersonId]=useState("");
  const [opportunityId,setOpportunityId]=useState("");
  const [items,setItems]=useState<any[]>([]);
  const [total,setTotal]=useState(0); const [offset,setOffset]=useState(0); const limit=20;
  const [err,setErr]=useState<string|null>(null); const [loading,setLoading]=useState(true);
  const load=async(o:number)=>{
    setLoading(true); setErr(null);
    try{
      const r=await api.getTasks(id,{
        status:status||undefined,
        has_outcome: hasOutcome===""?undefined:hasOutcome==="yes",
        person_id: personId?Number(personId):undefined,
        opportunity_id: opportunityId?Number(opportunityId):undefined,
        limit, offset:o
      });
      setItems(r.items); setTotal(r.pagination.total); setOffset(o);
    }catch(e:any){setErr(e.message)} finally{setLoading(false)}
  };
  useEffect(()=>{load(0)},[id,status,hasOutcome,personId,opportunityId]);
  return <div className="space-y-4">
    <div className="flex justify-between items-center">
      <h1 className="text-2xl font-bold">Research Tasks</h1>
      <Link to={`/trees/${id}/research`} className="text-sm text-blue-600 underline">← Research Overview</Link>
    </div>
    <p className="text-sm text-gray-600">Investigations you have chosen to work on — distinct from automatically detected opportunities.</p>
    <div className="flex gap-2 flex-wrap">
      <select value={status} onChange={e=>setStatus(e.target.value)} className="border rounded px-2 py-1">
        <option value="">All status</option><option value="OPEN">Open</option><option value="IN_PROGRESS">In Progress</option><option value="RESOLVED">Resolved</option><option value="REJECTED">Rejected</option><option value="INCONCLUSIVE">Inconclusive</option>
      </select>
      <select value={hasOutcome} onChange={e=>setHasOutcome(e.target.value)} className="border rounded px-2 py-1">
        <option value="">All outcomes</option><option value="yes">Has Outcome</option><option value="no">No Outcome</option>
      </select>
      <input placeholder="Person ID" value={personId} onChange={e=>setPersonId(e.target.value)} className="border rounded px-2 py-1 w-28" type="number" />
      <input placeholder="Opportunity ID" value={opportunityId} onChange={e=>setOpportunityId(e.target.value)} className="border rounded px-2 py-1 w-32" type="number" />
    </div>
    {loading ? <Loading msg="Loading research tasks…" /> : err ? <ErrorState msg={err} onRetry={()=>load(offset)} /> : items.length===0 ? <Empty msg="No research tasks yet. Start one from an opportunity." /> :
      <div className="space-y-2">
        {items.map((t:any)=><Link key={t.id} to={`/trees/${id}/research/tasks/${t.id}`} className="block border rounded p-4 hover:bg-gray-50">
          <div className="flex justify-between items-start gap-2"><span className="font-semibold">Research Task: {t.title}</span><TaskStatusBadge status={t.status} /></div>
          <div className="text-xs text-gray-600 mt-1 space-y-1">
            <div>Person: {t.person_id ? <span className="font-medium">{t.person_id}</span> : "—"}</div>
            <div>Updated: {new Date(t.updated_at).toLocaleDateString()}</div>
            <div>Outcome: {t.has_outcome ? <span className="px-1 bg-emerald-100 rounded font-semibold">Recorded</span> : "Not recorded"}</div>
            {t.opportunity && <div>From Opportunity Score: {t.opportunity.score} Priority: {t.opportunity.priority}</div>}
          </div>
          <div className="text-xs text-gray-700 mt-2 line-clamp-2">{t.description||"—"}</div>
          <div className="text-xs mt-2 text-blue-600">
            {t.has_outcome ? "View Result →" : t.status==="IN_PROGRESS" ? "Continue Research →" : t.status==="OPEN" ? "Start Research →" : "View →"}
          </div>
        </Link>)}
        <Pagination limit={limit} offset={offset} total={total} onChange={load} />
      </div>
    }
  </div>
}
