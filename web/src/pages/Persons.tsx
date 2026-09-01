import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { api } from "../api/client";
import { Loading, ErrorState, Empty, Pagination } from "../components/common";

export default function Persons(){
  const {treeId}=useParams(); const id=Number(treeId);
  const [items,setItems]=useState<any[]>([]); const [total,setTotal]=useState(0); const [offset,setOffset]=useState(0); const limit=20;
  const [err,setErr]=useState<string|null>(null); const [loading,setLoading]=useState(true);
  const load=async(o:number)=>{ setLoading(true); setErr(null); try{ const r=await api.getPersons(id,{limit,offset:o}); setItems(r.items); setTotal(r.pagination.total); setOffset(o);}catch(e:any){setErr(e.message)} finally{setLoading(false)}}
  useEffect(()=>{load(0)},[id]);
  if(loading) return <Loading msg="Loading persons…" />;
  if(err) return <ErrorState msg={err} onRetry={()=>load(offset)} />;
  if(items.length===0) return <Empty msg="No persons found." />;
  return <div className="space-y-4">
    <h1 className="text-2xl font-bold">Persons</h1>
    <div className="space-y-1">
      {items.map((p:any)=><Link key={p.id} to={`/trees/${id}/persons/${p.id}`} className="block border rounded p-2 hover:bg-gray-50">
        <div className="font-medium">{p.display_name || `${p.given_name||""} ${p.surname||""}`.trim() || p.gedcom_id}</div>
        <div className="text-xs text-gray-600">{p.gedcom_id} · {p.birth_place||"no place"} · {p.birth_date_original||"no date"}</div>
      </Link>)}
    </div>
    <Pagination limit={limit} offset={offset} total={total} onChange={load} />
  </div>
}
