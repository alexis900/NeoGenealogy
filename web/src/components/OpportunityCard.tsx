import { PriorityBadge, ScoreBadge, ResearchabilityBadge, ConfidenceIndicator } from "./Badges";
import type { ResearchOpportunity } from "../api/types";
import { Link } from "react-router-dom";
export function OpportunityCard({opp, treeId}:{opp:ResearchOpportunity; treeId:number}){
  return <div className="border rounded p-4 bg-white shadow-sm">
    <div className="flex justify-between items-start gap-2">
      <PriorityBadge p={opp.priority} />
      <ScoreBadge score={opp.score} />
    </div>
    <div className="mt-2 font-semibold">Person {opp.person_id} </div>
    <div className="text-xs text-gray-600 flex gap-2 items-center"><ResearchabilityBadge r={opp.researchability} /> <ConfidenceIndicator c={opp.confidence} /></div>
    {opp.why && <p className="text-sm mt-2 text-gray-700">{opp.why}</p>}
    <Link to={`/trees/${treeId}/research/${opp.id}`} className="text-sm text-blue-600 underline mt-2 inline-block">Ver oportunidad</Link>
  </div>
}
export function ScoreBreakdown({breakdown}:{breakdown:{total:number; components:{name:string;points:number;reason:string}[]}}){
  if(!breakdown) return null;
  return <div className="border rounded p-3 bg-gray-50">
    {breakdown.components?.map((c,i)=><div key={i} className="flex gap-2 py-1 border-b last:border-0">
      <span className="font-mono w-12 text-right">+{c.points}</span>
      <div><div className="font-semibold text-sm">{c.name}</div><div className="text-xs text-gray-600">{c.reason}</div></div>
    </div>)}
    <div className="font-bold text-right pt-2">Total {breakdown.total}</div>
  </div>
}
