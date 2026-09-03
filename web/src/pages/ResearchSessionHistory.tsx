import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { api } from "../api/client";
import { Loading, ErrorState, Empty } from "../components/common";

function statusClass(s:string){
  if(s==="COMPLETED") return "bg-gray-600 text-white";
  if(s==="ABANDONED") return "bg-red-600 text-white";
  return "bg-gray-400 text-white";
}

export default function ResearchSessionHistory(){
  const {treeId}=useParams(); const id=Number(treeId);
  const [items,setItems]=useState<any[]>([]);
  const [status,setStatus]=useState("");
  const [personId,setPersonId]=useState("");
  const [loading,setLoading]=useState(true);
  const [err,setErr]=useState<string|null>(null);
  const [total,setTotal]=useState(0);
  const [limit] = useState(20);
  const [page,setPage]=useState(1);
  const hasFilters = status!=="" || personId!=="";

  const load=async()=>{
    setLoading(true); setErr(null);
    try{
      const res=await api.getSessionHistory(id,{
        status: status||undefined,
        person_id: personId?Number(personId):undefined,
        limit,
        page
      });
      setItems(res.items);
      setTotal(res.pagination.total);
    }catch(e:any){setErr(e.message)} finally{setLoading(false)}
  };
  useEffect(()=>{load()},[id,status,personId,page]);

  // reset page when filters change
  useEffect(()=>{setPage(1)},[status,personId]);

  const totalPages = Math.max(1, Math.ceil(total/limit));

  return <div className="space-y-4">
    <div className="flex justify-between items-start">
      <div>
        <h1 className="text-2xl font-bold">Research Session History</h1>
        <p className="text-sm text-gray-600">Completed and abandoned sessions — ordered by completion date.</p>
      </div>
      <Link to={`/trees/${id}/research/sessions`} className="text-sm text-blue-600 underline">← Sessions</Link>
    </div>

    <div className="flex gap-2 text-sm border-b pb-2">
      <Link to={`/trees/${id}/research/sessions`} className="px-3 py-1 border rounded hover:bg-gray-50">Active &amp; Planned</Link>
      <span className="px-3 py-1 bg-gray-800 text-white rounded">History</span>
    </div>

    <div className="flex gap-2 flex-wrap items-end bg-gray-50 border rounded p-3">
      <div>
        <label className="text-xs text-gray-600 block">Status</label>
        <select value={status} onChange={e=>setStatus(e.target.value)} className="border rounded px-2 py-1 text-sm">
          <option value="">All</option>
          <option value="COMPLETED">COMPLETED</option>
          <option value="ABANDONED">ABANDONED</option>
        </select>
      </div>
      <div>
        <label className="text-xs text-gray-600 block">Person</label>
        <input value={personId} onChange={e=>setPersonId(e.target.value)} placeholder="person id" className="border rounded px-2 py-1 w-24 text-sm" />
      </div>
    </div>

    {loading ? <Loading msg="Loading history…" /> : err ? <ErrorState msg={err} onRetry={load} /> : items.length===0 ? (
      hasFilters
        ? <Empty msg="No sessions match the selected filters." />
        : <Empty msg="No completed research sessions yet. Completed and abandoned sessions will appear here." />
    ) : (
      <div className="space-y-3">
        <div className="text-xs text-gray-600">{total} sessions</div>
        {items.map((s:any)=>{
          const stats=s.stats;
          const completed = s.completed_at || s.updated_at;
          return <div key={s.id} className="border rounded p-4 bg-white">
            <div className="flex justify-between items-start gap-2">
              <span className={`px-2 py-0.5 rounded text-xs font-semibold ${statusClass(s.status)}`}>{s.status}</span>
              <span className="text-xs text-gray-500">Completed {new Date(completed).toLocaleDateString()}</span>
            </div>
            <div className="font-semibold mt-2">{s.title}</div>
            {s.description && <div className="text-xs text-gray-500 mt-1">{s.description}</div>}
            <div className="text-xs text-gray-600 mt-2 flex gap-3 flex-wrap">
              <span>{stats?.total_tasks ?? 0} tasks</span>
              <span>{stats?.total_outcomes ?? 0} outcomes</span>
              <span>{stats?.total_evidence ?? 0} evidence</span>
              {stats?.supporting_evidence!==undefined && <span>{stats.supporting_evidence} supporting</span>}
              {stats?.contradicting_evidence!==undefined && stats.contradicting_evidence>0 && <span>{stats.contradicting_evidence} contradicting</span>}
            </div>
            <Link to={`/trees/${id}/research/sessions/${s.id}`} className="text-xs text-blue-600 underline mt-2 inline-block">View Session</Link>
          </div>
        })}
        {totalPages>1 && <div className="flex gap-2 justify-center items-center text-sm">
          <button disabled={page<=1} onClick={()=>setPage(p=>Math.max(1,p-1))} className="px-3 py-1 border rounded disabled:opacity-50">Prev</button>
          <span>Page {page} / {totalPages}</span>
          <button disabled={page>=totalPages} onClick={()=>setPage(p=>p+1)} className="px-3 py-1 border rounded disabled:opacity-50">Next</button>
        </div>}
      </div>
    )}
  </div>
}
