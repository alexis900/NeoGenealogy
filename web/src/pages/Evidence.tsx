import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { api } from "../api/client";
import { Loading, ErrorState, Empty, Pagination } from "../components/common";

export default function Evidence(){
  const {treeId}=useParams(); const id=Number(treeId);
  const [items,setItems]=useState<any[]>([]);
  const [total,setTotal]=useState(0); const [offset,setOffset]=useState(0); const limit=20;
  const [err,setErr]=useState<string|null>(null); const [loading,setLoading]=useState(true);
  const [showCreate,setShowCreate]=useState(false);
  const [sourceId,setSourceId]=useState(""); const [citationId,setCitationId]=useState(""); const [statement,setStatement]=useState(""); const [notes,setNotes]=useState("");

  const load=async(o:number)=>{
    setLoading(true); setErr(null);
    try{
      const r=await api.getEvidenceList(id,{limit, offset:o});
      setItems(r.items); setTotal(r.pagination.total); setOffset(o);
    }catch(e:any){setErr(e.message)} finally{setLoading(false)}
  };
  useEffect(()=>{load(0)},[id]);

  const create=async()=>{
    try{
      await api.createEvidence(id,{source_id: Number(sourceId), citation_id: citationId?Number(citationId):undefined, statement, notes: notes||undefined});
      setShowCreate(false); setSourceId(""); setCitationId(""); setStatement(""); setNotes("");
      load(0);
    }catch(e:any){setErr(e.message)}
  };
  return <div className="space-y-4">
    <div className="flex justify-between items-center">
      <h1 className="text-2xl font-bold">Evidence</h1>
      <Link to={`/trees/${id}/research`} className="text-sm text-blue-600 underline">← Research</Link>
    </div>
    <p className="text-sm text-gray-600">Evidence pieces extracted from sources.</p>
    <button onClick={()=>setShowCreate(!showCreate)} className="px-3 py-1 bg-emerald-600 text-white rounded text-sm">{showCreate?"Cancel":"Create Evidence"}</button>
    {showCreate && <div className="border rounded p-4 bg-white space-y-2">
      <input placeholder="Source ID (required)" value={sourceId} onChange={e=>setSourceId(e.target.value)} className="w-full border rounded px-2 py-1" type="number" />
      <input placeholder="Citation ID (optional)" value={citationId} onChange={e=>setCitationId(e.target.value)} className="w-full border rounded px-2 py-1" type="number" />
      <textarea placeholder="Statement (required)" value={statement} onChange={e=>setStatement(e.target.value)} className="w-full border rounded px-2 py-1" rows={3} />
      <textarea placeholder="Notes" value={notes} onChange={e=>setNotes(e.target.value)} className="w-full border rounded px-2 py-1" rows={2} />
      <button onClick={create} disabled={!sourceId.trim()||!statement.trim()} className="px-4 py-2 bg-blue-600 text-white rounded disabled:opacity-50">Save</button>
    </div>}
    {loading ? <Loading msg="Loading evidence…" /> : err ? <ErrorState msg={err} onRetry={()=>load(offset)} /> : items.length===0 ? <Empty msg="No evidence yet. Create one from a source." /> :
      <div className="space-y-2">
        {items.map((e:any)=><div key={e.id} className="border rounded p-4 bg-white">
          <div className="text-sm font-medium">{e.statement}</div>
          {e.notes && <div className="text-xs text-gray-600 mt-1">{e.notes}</div>}
          <div className="text-xs text-gray-500 mt-1">Source {e.source_id} {e.citation_id?`· Citation ${e.citation_id}`:""} · {new Date(e.created_at).toLocaleDateString()}</div>
          {e.source && <div className="text-xs">Source: {e.source.title} ({e.source.type})</div>}
          {e.citation && <div className="text-xs">Citation: {e.citation.locator||"—"}</div>}
          <Link to={`/trees/${id}/evidence/${e.id}`} className="text-xs text-blue-600 underline">View</Link>
        </div>)}
        <Pagination limit={limit} offset={offset} total={total} onChange={load} />
      </div>
    }
  </div>
}
