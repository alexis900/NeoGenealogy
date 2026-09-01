import { useEffect, useState } from "react";
import { Link } from "react-router-dom";
import { api } from "../api/client";
import type { TreeSummary } from "../api/types";
import { Loading, ErrorState, Empty } from "../components/common";

export default function Trees(){
  const [data,setData]=useState<TreeSummary[]|null>(null);
  const [err,setErr]=useState<string|null>(null);
  const [loading,setLoading]=useState(true);
  const load=async()=>{ setLoading(true); setErr(null); try{ const r=await api.getTrees(); setData(r.items);}catch(e:any){setErr(e.message)} finally{setLoading(false)}}
  useEffect(()=>{load()},[]);
  if(loading) return <Loading msg="Loading trees…" />;
  if(err) return <ErrorState msg={err} onRetry={load} />;
  if(!data || data.length===0) return <Empty msg="No genealogy trees imported yet." />;
  return <div className="space-y-4">
    <h1 className="text-2xl font-bold">Trees</h1>
    {data.map(t=><Link key={t.id} to={`/trees/${t.id}`} className="block border rounded p-4 hover:bg-gray-50">
      <div className="font-semibold">{t.name}</div>
      <div className="text-sm text-gray-600">{t.persons} persons · {t.families} families · {t.findings} findings · {t.research_opportunities} opportunities</div>
    </Link>)}
  </div>
}
