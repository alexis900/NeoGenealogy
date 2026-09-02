import { useEffect, useState } from "react";
import { useParams, Link, useNavigate } from "react-router-dom";
import { api } from "../api/client";
import { Loading, ErrorState } from "../components/common";
import { ScoreBadge, PriorityBadge } from "../components/Badges";
import { ScoreBreakdown } from "../components/OpportunityCard";

export default function OpportunityDetail(){
  const {treeId, oppId}=useParams(); const tid=Number(treeId); const navigate=useNavigate();
  const [opp,setOpp]=useState<any>(null); const [err,setErr]=useState<string|null>(null);
  const [taskExists,setTaskExists]=useState<any>(null); const [starting,setStarting]=useState(false);
  useEffect(()=>{
    (async()=>{
      try{
        const r=await api.getOpportunities(tid,{limit:100});
        const found=r.items.find((x:any)=> String(x.id)===String(oppId));
        if(!found) throw new Error("Opportunity not found");
        setOpp(found);
        // check existing task for this opportunity
        try{
          const tasks=await api.getTasks(tid,{opportunity_id:Number(oppId), limit:5});
          if(tasks.items.length>0) setTaskExists(tasks.items[0]);
        }catch{}
      }catch(e:any){setErr(e.message)}
    })();
  },[tid, oppId]);
  const startResearch=async()=>{
    setStarting(true);
    try{
      const task=await api.createTaskFromOpportunity(tid, Number(oppId), {title:`Research opportunity ${oppId} - person ${opp?.person_id}`, description: opp?.why});
      navigate(`/trees/${tid}/research/tasks/${task.id}`);
    }catch(e:any){setErr(e.message)} finally{setStarting(false)}
  };
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
    <div className="flex gap-2">
      {taskExists ? <Link to={`/trees/${tid}/research/tasks/${taskExists.id}`} className="px-4 py-2 bg-emerald-600 text-white rounded">View Research Task</Link>
        : <button onClick={startResearch} disabled={starting} className="px-4 py-2 bg-blue-600 text-white rounded disabled:opacity-50">{starting?"Starting…":"Start Research"}</button>}
      <Link to={`/trees/${tid}/persons/${opp.person_id}`} className="px-4 py-2 border rounded">Go to Person {opp.person_id}</Link>
    </div>
  </div>
}
