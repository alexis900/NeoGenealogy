import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { api } from "../api/client";
import { Loading, ErrorState, Empty } from "../components/common";

function statusClass(s:string){
  if(s==="ACTIVE") return "bg-emerald-600 text-white";
  if(s==="PLANNED") return "bg-blue-600 text-white";
  if(s==="COMPLETED") return "bg-gray-600 text-white";
  if(s==="ABANDONED") return "bg-red-600 text-white";
  return "bg-gray-400 text-white";
}

export default function ResearchSessions(){
  const {treeId}=useParams(); const id=Number(treeId);
  const [items,setItems]=useState<any[]>([]);
  const [status,setStatus]=useState("");
  const [personId,setPersonId]=useState("");
  const [oppId,setOppId]=useState("");
  const [loading,setLoading]=useState(true);
  const [err,setErr]=useState<string|null>(null);
  const [total,setTotal]=useState(0);

  const load=async()=>{
    setLoading(true); setErr(null);
    try{
      const res=await api.getSessions(id,{
        status: status||undefined,
        person_id: personId?Number(personId):undefined,
        opportunity_id: oppId?Number(oppId):undefined,
        limit:50
      });
      setItems(res.items);
      setTotal(res.pagination.total);
    }catch(e:any){setErr(e.message)} finally{setLoading(false)}
  };
  useEffect(()=>{load()},[id,status,personId,oppId]);

  return <div className="space-y-4">
    <h1 className="text-2xl font-bold">Research Sessions</h1>
    <p className="text-sm text-gray-600">Investigations you are working on — planned, active, completed or abandoned.</p>
    <div className="flex gap-2 flex-wrap items-end bg-gray-50 border rounded p-3">
      <div>
        <label className="text-xs text-gray-600 block">Status</label>
        <select value={status} onChange={e=>setStatus(e.target.value)} className="border rounded px-2 py-1 text-sm">
          <option value="">All</option>
          <option value="ACTIVE">ACTIVE</option>
          <option value="PLANNED">PLANNED</option>
          <option value="COMPLETED">COMPLETED</option>
          <option value="ABANDONED">ABANDONED</option>
        </select>
      </div>
      <div>
        <label className="text-xs text-gray-600 block">Person</label>
        <input value={personId} onChange={e=>setPersonId(e.target.value)} placeholder="person id" className="border rounded px-2 py-1 w-24 text-sm" />
      </div>
      <div>
        <label className="text-xs text-gray-600 block">Opportunity</label>
        <input value={oppId} onChange={e=>setOppId(e.target.value)} placeholder="opp id" className="border rounded px-2 py-1 w-24 text-sm" />
      </div>
    </div>
    {loading ? <Loading msg="Loading research sessions…" /> : err ? <ErrorState msg={err} onRetry={load} /> : items.length===0 ? <Empty msg="No research sessions yet. Start from planning to create your first session." /> :
      <div className="space-y-3">
        <div className="text-xs text-gray-600">{total} sessions</div>
        {items.map((s:any)=><Link key={s.id} to={`/trees/${id}/research/sessions/${s.id}`} className="block border rounded p-4 bg-white hover:bg-gray-50">
          <div className="flex justify-between items-start gap-2">
            <span className={`px-2 py-0.5 rounded text-xs font-semibold ${statusClass(s.status)}`}>{s.status}</span>
            <span className="text-xs text-gray-500">{new Date(s.updated_at).toLocaleDateString()}</span>
          </div>
          <div className="font-semibold mt-2">{s.title}</div>
          <div className="text-xs text-gray-600">Person {s.person_id ?? "—"} · Opportunity {s.opportunity_id ?? "—"}</div>
          <div className="text-xs text-gray-500 mt-1">{s.description || "No description"}</div>
        </Link>)}
      </div>
    }
    <Link to={`/trees/${id}/research/planning`} className="text-sm text-blue-600 underline">← Planning</Link>
  </div>
}
