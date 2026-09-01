import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { api } from "../api/client";
import { Loading, ErrorState } from "../components/common";
import { FindingBadge } from "../components/Badges";

export default function PersonDetail(){
  const {treeId, personId}=useParams(); const tid=Number(treeId); const pid=Number(personId);
  const [person,setPerson]=useState<any>(null); const [findings,setFindings]=useState<any[]>([]); const [opps,setOpps]=useState<any[]>([]); const [err,setErr]=useState<string|null>(null);
  useEffect(()=>{
    (async()=>{
      try{
        const p=await api.getPerson(tid,pid);
        const f=await api.getFindings(tid,{person_id:pid,limit:50});
        const o=await api.getOpportunities(tid,{limit:100});
        setPerson(p); setFindings(f.items); setOpps(o.items.filter((x:any)=> x.person_id===pid));
      }catch(e:any){setErr(e.message)}
    })();
  },[tid,pid]);
  if(err) return <ErrorState msg={err} />;
  if(!person) return <Loading msg="Loading person…" />;
  return <div className="space-y-6">
    <h1 className="text-2xl font-bold">{person.display_name || `${person.given_name||""} ${person.surname||""}`}</h1>
    <div className="text-sm text-gray-600">{person.gedcom_id} · {person.sex||""}</div>
    <div className="grid grid-cols-2 gap-4">
      <div className="border rounded p-3"><div className="text-xs text-gray-500">Birth</div><div>{person.birth_date_original||"—"} {person.birth_place?`· ${person.birth_place}`:""}</div></div>
      <div className="border rounded p-3"><div className="text-xs text-gray-500">Death</div><div>{person.death_date_original||"—"} {person.death_place?`· ${person.death_place}`:""}</div></div>
    </div>
    <div><h2 className="font-semibold">Findings ({findings.length})</h2>
      {findings.length===0? <div className="text-sm text-gray-500">No findings</div> : findings.map((f:any)=><div key={f.id} className="border rounded p-2 mt-2"><FindingBadge severity={f.severity} /> <span className="text-sm ml-2">{f.finding_type}: {f.message}</span></div>)}
    </div>
    <div><h2 className="font-semibold">Research Opportunities ({opps.length})</h2>
      {opps.map((o:any)=><div key={o.id} className="border rounded p-2 mt-2"><span className="font-semibold">Score {o.score}</span> — {o.why}</div>)}
    </div>
  </div>
}
