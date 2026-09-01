import { useEffect, useState } from "react";
import { useParams } from "react-router-dom";
import { api } from "../api/client";
import { Loading, ErrorState } from "../components/common";

function Bar({label, value}:{label:string;value:number}){
  return <div className="flex items-center gap-2">
    <div className="w-24 text-sm">{label}</div>
    <div className="flex-1 h-4 bg-gray-200 rounded overflow-hidden"><div className="h-full bg-emerald-600" style={{width:`${value}%`}} /></div>
    <div className="w-12 text-sm text-right">{Math.round(value)}%</div>
  </div>
}

export default function Sources(){
  const {treeId}=useParams(); const id=Number(treeId);
  const [cov,setCov]=useState<any>(null); const [err,setErr]=useState<string|null>(null);
  useEffect(()=>{(async()=>{ try{ const r=await api.getCoverage(id); setCov(r);}catch(e:any){setErr(e.message)}})()},[id]);
  if(err) return <ErrorState msg={err} />;
  if(!cov) return <Loading msg="Loading coverage…" />;
  return <div className="space-y-4">
    <h1 className="text-2xl font-bold">Source Coverage</h1>
    <div className="space-y-2 border rounded p-4">
      <Bar label="Birth" value={cov.birth||0} />
      <Bar label="Marriage" value={cov.marriage||0} />
      <Bar label="Death" value={cov.death||0} />
      <Bar label="Other" value={cov.other_events||0} />
      <div className="border-t pt-2"><Bar label="Overall" value={cov.overall||0} /></div>
    </div>
  </div>
}
