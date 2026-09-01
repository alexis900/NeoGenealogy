import { useEffect, useState } from "react";
import { useParams, Link } from "react-router-dom";
import { api } from "../api/client";
import { Loading, ErrorState } from "../components/common";
import { ScoreBadge, PriorityBadge } from "../components/Badges";
import { ScoreBreakdown } from "../components/OpportunityCard";

export default function OpportunityDetail(){
  const {treeId, oppId}=useParams(); const tid=Number(treeId);
  const [opp,setOpp]=useState<any>(null); const [err,setErr]=useState<string|null>(null);
  useEffect(()=>{
    (async()=>{
      try{
        const r=await api.getOpportunities(tid,{limit:100});
        const found=r.items.find((x:any)=> String(x.id)===String(oppId));
        if(!found) throw new Error("Opportunity not found");
        setOpp(found);
      }catch(e:any){setErr(e.message)}
    })();
  },[tid, oppId]);
  if(err) return <ErrorState msg={err} />;
  if(!opp) return <Loading msg="Loading opportunity…" />;
  return <div className="space-y-4">
    <Link to={`/trees/${tid}/research`} className="text-sm text-blue-600 underline">← Research Queue</Link>
    <h1 className="text-2xl font-bold">Opportunity {opp.id}</h1>
    <div className="flex gap-2"><PriorityBadge p={opp.priority} /><ScoreBadge score={opp.score} /></div>
    <div><h3 className="font-semibold">Why</h3><p className="text-sm">{opp.why}</p></div>
    <div><h3 className="font-semibold">What</h3><pre className="text-xs bg-gray-50 p-2 rounded">{JSON.stringify(opp.what,null,2)}</pre></div>
    <div><h3 className="font-semibold">Potential Sources</h3><pre className="text-xs bg-gray-50 p-2 rounded">{JSON.stringify(opp.potential_sources,null,2)}</pre></div>
    {opp.breakdown && <div><h3 className="font-semibold">Score Breakdown</h3><ScoreBreakdown breakdown={opp.breakdown} /></div>}
    <Link to={`/trees/${tid}/persons/${opp.person_id}`} className="text-blue-600 underline">Go to Person {opp.person_id}</Link>
  </div>
}
