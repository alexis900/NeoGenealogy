import { useEffect, useState } from "react";
import { useParams, Link } from "react-router-dom";
import { api } from "../api/client";
import { Loading, ErrorState } from "../components/common";

export default function EvidenceDetail(){
  const {treeId, evidenceId}=useParams(); const tid=Number(treeId); const eid=Number(evidenceId);
  const [ev,setEv]=useState<any>(null); const [err,setErr]=useState<string|null>(null); const [loading,setLoading]=useState(true);
  const [edit,setEdit]=useState(false); const [statement,setStatement]=useState(""); const [notes,setNotes]=useState("");

  const load=async()=>{
    setLoading(true); setErr(null);
    try{
      const e=await api.getEvidence(tid,eid);
      setEv(e); setStatement(e.statement); setNotes(e.notes||"");
    }catch(e:any){setErr(e.message)} finally{setLoading(false)}
  };
  useEffect(()=>{load()},[tid,eid]);

  const save=async()=>{
    try{
      const updated=await api.updateEvidence(tid,eid,{statement, notes: notes||undefined});
      setEv(updated); setEdit(false);
    }catch(e:any){setErr(e.message)}
  };
  const remove=async()=>{
    if(!confirm("Delete evidence?")) return;
    try{ await api.deleteEvidence(tid,eid); window.location.href=`/trees/${tid}/evidence`; }catch(e:any){setErr(e.message)}
  };
  if(loading) return <Loading msg="Loading evidence…" />;
  if(err) return <ErrorState msg={err} />;
  if(!ev) return null;
  return <div className="space-y-4">
    <Link to={`/trees/${tid}/evidence`} className="text-sm text-blue-600 underline">← Evidence</Link>
    <h1 className="text-2xl font-bold">Evidence #{ev.id}</h1>
    <div className="border rounded p-4 bg-white space-y-2">
      {edit ? <>
        <textarea value={statement} onChange={e=>setStatement(e.target.value)} className="w-full border rounded px-2 py-1" rows={3} />
        <textarea value={notes} onChange={e=>setNotes(e.target.value)} className="w-full border rounded px-2 py-1" rows={2} placeholder="Notes" />
        <div className="flex gap-2"><button onClick={save} className="px-3 py-1 bg-blue-600 text-white rounded">Save</button><button onClick={()=>setEdit(false)} className="px-3 py-1 border rounded">Cancel</button></div>
      </> : <>
        <div className="text-sm font-medium">{ev.statement}</div>
        {ev.notes && <div className="text-sm text-gray-600">Notes: {ev.notes}</div>}
        <div className="text-xs text-gray-600">Source {ev.source_id} {ev.citation_id?`· Citation ${ev.citation_id}`:""}</div>
        {ev.source && <div className="text-xs">Source: {ev.source.title}</div>}
        {ev.citation && <div className="text-xs">Citation: {ev.citation.locator}</div>}
        <div className="flex gap-2"><button onClick={()=>setEdit(true)} className="px-3 py-1 bg-blue-600 text-white rounded">Edit</button><button onClick={remove} className="px-3 py-1 bg-red-600 text-white rounded">Delete</button></div>
      </>}
    </div>
  </div>
}
