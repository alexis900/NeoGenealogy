export function PriorityBadge({p}:{p:string}){
  const c = p==="critical"?"bg-red-700 text-white":p==="high"?"bg-red-500 text-white":p==="medium"?"bg-amber-500 text-white":"bg-gray-200 text-gray-800";
  return <span className={`px-2 py-0.5 rounded text-xs font-semibold uppercase ${c}`}>{p}</span>
}
export function ScoreBadge({score}:{score:number}){
  const c = score>=85?"bg-red-600":score>=65?"bg-orange-500":score>=35?"bg-amber-400":"bg-gray-300";
  return <span className={`px-2 py-1 rounded font-bold text-white ${c}`}>{score}</span>
}
export function ResearchabilityBadge({r}:{r:string}){
  const c = r==="high"?"bg-emerald-600":r==="medium"?"bg-amber-500":"bg-gray-400";
  return <span className={`px-2 py-0.5 rounded text-xs text-white ${c}`}>{r}</span>
}
export function ConfidenceIndicator({c}:{c:number}){
  return <span className="text-xs text-gray-600">{Math.round(c*100)}% confidence</span>
}
export function FindingBadge({severity}:{severity:string}){
  const map:any={critical:"bg-red-800",high:"bg-red-600",warning:"bg-amber-500",medium:"bg-amber-400",low:"bg-gray-300",info:"bg-blue-400"};
  return <span className={`px-2 py-0.5 rounded text-xs text-white ${map[severity]||"bg-gray-400"}`}>{severity}</span>
}
