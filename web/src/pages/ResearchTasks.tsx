import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { api } from "../api/client";
import { Loading, ErrorState, Empty, Pagination } from "../components/common";
import { TaskStatusBadge } from "../components/Badges";

export default function ResearchTasks(){
  const {treeId}=useParams(); const id=Number(treeId);
  const [status,setStatus]=useState(""); const [items,setItems]=useState<any[]>([]);
  const [total,setTotal]=useState(0); const [offset,setOffset]=useState(0); const limit=20;
  const [err,setErr]=useState<string|null>(null); const [loading,setLoading]=useState(true);
  const load=async(o:number)=>{
    setLoading(true); setErr(null);
    try{
      const r=await api.getTasks(id,{status:status||undefined, limit, offset:o});
      setItems(r.items); setTotal(r.pagination.total); setOffset(o);
    }catch(e:any){setErr(e.message)} finally{setLoading(false)}
  };
  useEffect(()=>{load(0)},[id,status]);
  return <div className="space-y-4">
    <div className="flex justify-between items-center">
      <h1 className="text-2xl font-bold">Research Tasks</h1>
      <Link to={`/trees/${id}/research`} className="text-sm text-blue-600 underline">← Research Queue</Link>
    </div>
    <p className="text-sm text-gray-600">Investigations you have chosen to work on — distinct from automatically detected opportunities.</p>
    <div className="flex gap-2 flex-wrap">
      <select value={status} onChange={e=>setStatus(e.target.value)} className="border rounded px-2 py-1">
        <option value="">All</option><option value="OPEN">Open</option><option value="IN_PROGRESS">In Progress</option><option value="RESOLVED">Resolved</option><option value="REJECTED">Rejected</option><option value="INCONCLUSIVE">Inconclusive</option>
      </select>
    </div>
    {loading ? <Loading msg="Loading research tasks…" /> : err ? <ErrorState msg={err} onRetry={()=>load(offset)} /> : items.length===0 ? <Empty msg="No research tasks yet. Start one from an opportunity." /> :
      <div className="space-y-2">
        {items.map((t:any)=><Link key={t.id} to={`/trees/${id}/research/tasks/${t.id}`} className="block border rounded p-4 hover:bg-gray-50">
          <div className="flex justify-between items-start gap-2"><span className="font-semibold">{t.title}</span><TaskStatusBadge status={t.status} /></div>
          <div className="text-xs text-gray-600">Person {t.person_id||"—"} · {new Date(t.created_at).toLocaleDateString()}</div>
          <div className="text-sm text-gray-700 mt-1 line-clamp-2">{t.description||"—"}</div>
        </Link>)}
        <Pagination limit={limit} offset={offset} total={total} onChange={load} />
      </div>
    }
  </div>
}
