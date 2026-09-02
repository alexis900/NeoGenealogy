import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { api } from "../api/client";
import { Loading, ErrorState, Empty } from "../components/common";
import { PriorityBadge, ScoreBadge, ResearchabilityBadge, ConfidenceIndicator } from "../components/Badges";

type PlanItem = {
  opportunity_id: number;
  person_id: number;
  title: string;
  priority: string;
  research_score: number;
  planning_score: number;
  researchability: string;
  confidence: number;
  active_task: boolean;
  task_status?: string | null;
  reasons: { code: string; label: string; description: string }[];
};

export default function ResearchPlanning(){
  const {treeId}=useParams(); const id=Number(treeId);
  const [data,setData]=useState<any>(null);
  const [loading,setLoading]=useState(true);
  const [err,setErr]=useState<string|null>(null);
  const [priority,setPriority]=useState("");
  const [researchability,setResearchability]=useState("");
  const [minScore,setMinScore]=useState("");
  const [limit,setLimit]=useState("10");

  const load=async()=>{
    setLoading(true); setErr(null);
    try{
      const r=await api.getPlan(id,{
        limit: limit?Number(limit):undefined,
        min_score: minScore?Number(minScore):undefined,
        priority: priority||undefined,
        researchability: researchability||undefined,
      });
      setData(r);
    }catch(e:any){ setErr(e.message)} finally{setLoading(false)}
  };
  useEffect(()=>{ load(); },[id, priority, researchability, minScore, limit]);

  const startResearch=async(oppId:number, personId:number)=>{
    try{
      const task=await api.createTaskFromOpportunity(id, oppId, {title:`Research opportunity ${oppId} - person ${personId}`});
      window.location.href=`/trees/${id}/research/tasks/${task.id}`;
    }catch(e:any){ setErr(e.message)}
  };

  if(loading) return <Loading msg="Loading research planning…" />;
  if(err) return <ErrorState msg={err} onRetry={load} />;

  const summary=data?.summary;
  const recommended:PlanItem[]=data?.recommended||[];
  const deferred:PlanItem[]=data?.deferred||[];

  return <div className="space-y-6">
    <h1 className="text-2xl font-bold">Research Planning</h1>
    <p className="text-sm text-gray-600">What should I research next? Deterministic planning based on existing research opportunities, scores and evidence gaps. No AI.</p>

    {summary && <div className="flex gap-4 text-sm bg-gray-50 border rounded p-3">
      <span><strong>{summary.recommended_count}</strong> recommended investigations</span>
      <span><strong>{summary.total_candidates}</strong> total candidates</span>
      {summary.deferred_count!==undefined && <span>{summary.deferred_count} deferred</span>}
      {summary.active_count!==undefined && <span>Active {summary.active_count}</span>}
    </div>}

    <div className="flex gap-2 flex-wrap items-end">
      <div>
        <label className="text-xs text-gray-600 block">Priority</label>
        <select value={priority} onChange={e=>setPriority(e.target.value)} className="border rounded px-2 py-1">
          <option value="">All priorities</option>
          <option value="critical">Critical</option>
          <option value="high">High</option>
          <option value="medium">Medium</option>
          <option value="low">Low</option>
        </select>
      </div>
      <div>
        <label className="text-xs text-gray-600 block">Researchability</label>
        <select value={researchability} onChange={e=>setResearchability(e.target.value)} className="border rounded px-2 py-1">
          <option value="">All</option>
          <option value="high">High</option>
          <option value="medium">Medium</option>
          <option value="low">Low</option>
        </select>
      </div>
      <div>
        <label className="text-xs text-gray-600 block">Min Planning Score</label>
        <input type="number" min={0} max={100} value={minScore} onChange={e=>setMinScore(e.target.value)} placeholder="0..100" className="border rounded px-2 py-1 w-28" />
      </div>
      <div>
        <label className="text-xs text-gray-600 block">Limit</label>
        <select value={limit} onChange={e=>setLimit(e.target.value)} className="border rounded px-2 py-1">
          <option value="10">10</option>
          <option value="20">20</option>
          <option value="50">50</option>
        </select>
      </div>
    </div>

    {recommended.length===0 ? <Empty msg="No recommended investigations. Try adjusting filters." /> :
      <div className="space-y-4">
        <h2 className="text-lg font-semibold">Recommended</h2>
        <p className="text-xs text-gray-600">Research Score = importance/interest · Planning Score = practical priority to decide what to tackle now</p>
        <div className="space-y-3">
          {recommended.map((item, idx)=>
            <div key={item.opportunity_id} className="border rounded p-4 bg-white">
              <div className="flex justify-between items-start gap-2">
                <div>
                  <div className="text-xs text-gray-500">#{idx+1} · Person {item.person_id}</div>
                  <div className="font-semibold">{item.title}</div>
                  <div className="text-xs text-gray-600">Opportunity {item.opportunity_id}</div>
                </div>
                <PriorityBadge p={item.priority.toLowerCase()} />
              </div>
              <div className="flex gap-3 mt-2 items-center text-sm">
                <span className="flex items-center gap-1">Research <ScoreBadge score={item.research_score} /></span>
                <span className="text-xs text-gray-600">Planning <span className="px-2 py-0.5 bg-gray-100 border rounded font-mono">{item.planning_score.toFixed(1)}</span></span>
                <ResearchabilityBadge r={item.researchability.toLowerCase()} />
                <ConfidenceIndicator c={item.confidence} />
                {item.active_task ? <span className="text-xs px-2 py-0.5 bg-blue-100 rounded">Active task{item.task_status? ` · ${item.task_status}`:""}</span> : item.task_status==="INCONCLUSIVE" ? <span className="text-xs px-2 py-0.5 bg-amber-100 rounded">Previously inconclusive</span> : <span className="text-xs px-2 py-0.5 bg-gray-100 rounded">No active task</span>}
              </div>
              {item.reasons && item.reasons.length>0 && <div className="mt-2">
                <div className="text-xs font-semibold">Why is this here?</div>
                <ul className="text-xs text-gray-700 list-disc pl-4">
                  {item.reasons.map(r=> <li key={r.code}><span className="font-medium">{r.label}</span> – {r.description} <span className="text-gray-400">({r.code})</span></li>)}
                </ul>
              </div>}
              <div className="flex gap-2 mt-3">
                <Link to={`/trees/${id}/research/opportunities/${item.opportunity_id}`} className="px-3 py-1 border rounded text-sm">View Opportunity</Link>
                {item.active_task
                  ? <Link to={`/trees/${id}/research/tasks?opportunity_id=${item.opportunity_id}`} className="px-3 py-1 bg-emerald-600 text-white rounded text-sm">View Research Task</Link>
                  : <button onClick={()=>startResearch(item.opportunity_id, item.person_id)} className="px-3 py-1 bg-blue-600 text-white rounded text-sm">Start Research</button>}
                <Link to={`/trees/${id}/persons/${item.person_id}`} className="px-3 py-1 border rounded text-sm">View Person</Link>
              </div>
            </div>
          )}
        </div>
        {deferred.length>0 && <div className="border rounded p-3 bg-gray-50">
          <div className="text-sm">Deferred <strong>{deferred.length}</strong> candidates not in top {limit}</div>
          <details className="mt-2">
            <summary className="text-sm text-blue-600 cursor-pointer">Show deferred ({deferred.length})</summary>
            <div className="mt-2 space-y-2">
              {deferred.map(item=>
                <div key={item.opportunity_id} className="border rounded p-3 bg-white text-sm">
                  <div className="flex justify-between"><span>{item.title} (Opp {item.opportunity_id})</span><span className="font-mono text-xs">{item.planning_score.toFixed(1)}</span></div>
                  <div className="text-xs text-gray-600">Research {item.research_score} · {item.priority} · {item.researchability}</div>
                  <Link to={`/trees/${id}/research/opportunities/${item.opportunity_id}`} className="text-blue-600 underline text-xs">View Opportunity</Link>
                </div>
              )}
            </div>
          </details>
        </div>}
      </div>
    }
  </div>
}
