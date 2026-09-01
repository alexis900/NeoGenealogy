import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { api } from "../api/client";
import { Loading, ErrorState, Empty } from "../components/common";

export default function Branches(){
  const {treeId}=useParams(); const id=Number(treeId);
  const [items,setItems]=useState<any[]>([]); const [err,setErr]=useState<string|null>(null); const [loading,setLoading]=useState(true);
  useEffect(()=>{(async()=>{ setLoading(true); setErr(null); try{ const r=await api.getBranches(id); setItems(r.items);}catch(e:any){setErr(e.message)} finally{setLoading(false)}})()},[id]);
  if(loading) return <Loading msg="Loading branches…" />;
  if(err) return <ErrorState msg={err} />;
  if(items.length===0) return <Empty msg="No branches found." />;
  return <div className="space-y-4">
    <h1 className="text-2xl font-bold">Branches</h1>
    <p className="text-sm text-gray-600">¿Qué rama merece más atención?</p>
    <div className="space-y-2">
      {items.map((b:any)=><div key={b.id} className="border rounded p-4">
        <div className="flex justify-between"><span className="font-semibold">{b.branch||b.name}</span><span className="font-bold">{b.branch_score||b.score}</span></div>
        <div className="text-xs text-gray-600">{b.opportunity_count} opportunities · {b.high_priority_count} high · gen {b.deepest_generation} · coverage {Math.round(b.source_coverage)}%</div>
      </div>)}
    </div>
  </div>
}
