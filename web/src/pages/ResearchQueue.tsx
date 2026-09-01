import { useEffect, useState } from "react";
import { useParams, Link } from "react-router-dom";
import { api } from "../api/client";
import { Loading, ErrorState, Empty } from "../components/common";
import { PriorityBadge, ScoreBadge, ResearchabilityBadge, ConfidenceIndicator } from "../components/Badges";

export default function ResearchQueue(){
  const {treeId}=useParams(); const id=Number(treeId);
  const [items,setItems]=useState<any[]>([]);
  const [priority,setPriority]=useState(""); const [sort,setSort]=useState("score"); const [minScore,setMinScore]=useState("");
  const [err,setErr]=useState<string|null>(null); const [loading,setLoading]=useState(true);
  const load=async()=>{
    setLoading(true); setErr(null);
    try{
      const r=await api.getOpportunities(id, {
        priority: priority||undefined,
        sort: sort||undefined,
        min_score: minScore?Number(minScore):undefined,
        limit:50
      });
      setItems(r.items);
    }catch(e:any){setErr(e.message)} finally{setLoading(false)}
  };
  useEffect(()=>{load()},[id, priority, sort, minScore]);
  return <div className="space-y-4">
    <h1 className="text-2xl font-bold">Research Queue</h1>
    <div className="flex gap-2 flex-wrap">
      <select value={priority} onChange={e=>setPriority(e.target.value)} className="border rounded px-2 py-1">
        <option value="">All priorities</option><option value="critical">Critical</option><option value="high">High</option><option value="medium">Medium</option><option value="low">Low</option>
      </select>
      <select value={sort} onChange={e=>setSort(e.target.value)} className="border rounded px-2 py-1">
        <option value="score">Sort: Score</option><option value="priority">Priority</option><option value="confidence">Confidence</option>
      </select>
      <input placeholder="Min score" value={minScore} onChange={e=>setMinScore(e.target.value)} className="border rounded px-2 py-1 w-24" type="number" />
    </div>
    {loading ? <Loading msg="Loading research queue…" /> : err ? <ErrorState msg={err} onRetry={load} /> : items.length===0 ? <Empty msg="No research opportunities found. Your current filters may be too restrictive." /> :
      <div className="space-y-3">
        {items.map((o:any)=><div key={o.id} className="border rounded p-4 bg-white">
          <div className="flex justify-between"><PriorityBadge p={o.priority} /><ScoreBadge score={o.score} /></div>
          <div className="font-semibold mt-2">Person {o.person_id}</div>
          <div className="text-xs flex gap-2"><ResearchabilityBadge r={o.researchability} /> <ConfidenceIndicator c={o.confidence} /></div>
          {o.why && <p className="text-sm mt-2">{o.why}</p>}
          <Link to={`/trees/${id}/research/${o.id}`} className="text-blue-600 text-sm underline mt-2 inline-block">Ver oportunidad</Link>
        </div>)}
      </div>
    }
  </div>
}
