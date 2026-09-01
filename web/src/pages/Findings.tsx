import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { api } from "../api/client";
import { Loading, ErrorState, Empty, Pagination } from "../components/common";
import { FindingBadge } from "../components/Badges";

export default function Findings(){
  const {treeId}=useParams(); const id=Number(treeId);
  const [severity,setSeverity]=useState(""); const [type,setType]=useState("");
  const [items,setItems]=useState<any[]>([]); const [total,setTotal]=useState(0); const [offset,setOffset]=useState(0); const limit=20;
  const [err,setErr]=useState<string|null>(null); const [loading,setLoading]=useState(true);
  const load=async(o:number)=>{ setLoading(true); setErr(null); try{ const r=await api.getFindings(id,{severity:severity||undefined, type:type||undefined, limit, offset:o}); setItems(r.items); setTotal(r.pagination.total); setOffset(o);}catch(e:any){setErr(e.message)} finally{setLoading(false)}}
  useEffect(()=>{load(0)},[id, severity, type]);
  return <div className="space-y-4">
    <h1 className="text-2xl font-bold">Findings</h1>
    <div className="flex gap-2">
      <select value={severity} onChange={e=>setSeverity(e.target.value)} className="border rounded px-2 py-1"><option value="">All severities</option><option value="critical">Critical</option><option value="high">High</option><option value="warning">Warning</option><option value="medium">Medium</option><option value="info">Info</option><option value="low">Low</option></select>
      <input placeholder="Type" value={type} onChange={e=>setType(e.target.value)} className="border rounded px-2 py-1" />
    </div>
    {loading ? <Loading msg="Loading findings…" /> : err ? <ErrorState msg={err} onRetry={()=>load(offset)} /> : items.length===0 ? <Empty msg="No findings match the selected filters." /> :
      <><div className="space-y-2">{items.map((f:any)=><div key={f.id} className="border rounded p-3"><FindingBadge severity={f.severity} /> <span className="font-semibold text-sm ml-2">{f.finding_type}</span><div className="text-sm mt-1">{f.message}</div><div className="text-xs text-gray-500">Person {f.person_id||"—"}</div></div>)}</div><Pagination limit={limit} offset={offset} total={total} onChange={load} /></>
    }
  </div>
}
