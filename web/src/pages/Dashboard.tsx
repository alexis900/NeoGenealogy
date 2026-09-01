import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { api } from "../api/client";
import { Loading, ErrorState } from "../components/common";
import { ScoreBadge, PriorityBadge } from "../components/Badges";

export default function Dashboard(){
  const {treeId}=useParams(); const id=Number(treeId);
  const [tree,setTree]=useState<any>(null);
  const [top,setTop]=useState<any[]>([]);
  const [err,setErr]=useState<string|null>(null);
  const [loading,setLoading]=useState(true);
  const load=async()=>{
    setLoading(true); setErr(null);
    try{
      const t=await api.getTree(id);
      const topRes=await api.getTop(id,{limit:5});
      setTree(t); setTop(topRes.items);
    }catch(e:any){setErr(e.message)} finally{setLoading(false)}
  };
  useEffect(()=>{load()},[id]);
  if(loading) return <Loading msg="Loading dashboard…" />;
  if(err) return <ErrorState msg={err} onRetry={load} />;
  if(!tree) return null;
  return <div className="space-y-6">
    <h1 className="text-2xl font-bold">{tree.name}</h1>
    <div className="grid grid-cols-4 gap-4">
      <div className="border rounded p-4"><div className="text-2xl font-bold">{tree.persons}</div><div className="text-sm text-gray-600">Persons</div></div>
      <div className="border rounded p-4"><div className="text-2xl font-bold">{tree.families}</div><div className="text-sm text-gray-600">Families</div></div>
      <div className="border rounded p-4"><div className="text-2xl font-bold">{tree.findings}</div><div className="text-sm text-gray-600">Findings</div></div>
      <div className="border rounded p-4"><div className="text-2xl font-bold">{tree.research_opportunities}</div><div className="text-sm text-gray-600">Opportunities</div></div>
    </div>
    <div>
      <h2 className="text-xl font-semibold mb-2">Top Research Opportunities</h2>
      <div className="space-y-2">
        {top.map((o:any)=><div key={o.id} className="border rounded p-3 flex justify-between items-center">
          <div><div className="font-medium">Person {o.person_id}</div><div className="text-xs text-gray-600">{o.why}</div></div>
          <div className="flex gap-2 items-center"><PriorityBadge p={o.priority} /><ScoreBadge score={o.score} /></div>
        </div>)}
      </div>
      <Link to={`/trees/${id}/research`} className="text-blue-600 underline text-sm mt-2 inline-block">Ver toda la cola de investigación →</Link>
    </div>
  </div>
}
