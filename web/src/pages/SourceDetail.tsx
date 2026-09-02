import { useEffect, useState } from "react";
import { useParams, Link } from "react-router-dom";
import { api } from "../api/client";
import { Loading, ErrorState } from "../components/common";

export default function SourceDetail(){
  const {treeId, sourceId}=useParams(); const tid=Number(treeId); const sid=Number(sourceId);
  const [source,setSource]=useState<any>(null);
  const [citations,setCitations]=useState<any[]>([]);
  const [err,setErr]=useState<string|null>(null); const [loading,setLoading]=useState(true);
  const [edit,setEdit]=useState(false);
  const [title,setTitle]=useState(""); const [author,setAuthor]=useState(""); const [pub,setPub]=useState(""); const [date,setDate]=useState(""); const [type,setType]=useState("BOOK");
  const [loc,setLoc]=useState(""); const [text,setText]=useState("");

  const load=async()=>{
    setLoading(true); setErr(null);
    try{
      const s=await api.getSource(tid,sid);
      setSource(s); setTitle(s.title); setAuthor(s.author||""); setPub(s.publication||""); setDate(s.date||""); setType(s.type);
      const c=await api.getCitations(tid,sid);
      setCitations(c.items);
    }catch(e:any){setErr(e.message)} finally{setLoading(false)}
  };
  useEffect(()=>{load()},[tid,sid]);

  const save=async()=>{
    try{
      const updated=await api.updateSource(tid,sid,{title, author: author||undefined, publication: pub||undefined, date: date||undefined, type});
      setSource(updated); setEdit(false);
    }catch(e:any){setErr(e.message)}
  };
  const remove=async()=>{
    if(!confirm("Delete source? Citations and evidence using it will be affected (citations deleted, evidence cascade).")) return;
    try{ await api.deleteSource(tid,sid); window.location.href=`/trees/${tid}/sources`; }catch(e:any){setErr(e.message)}
  };
  const createCitation=async()=>{
    try{
      const c=await api.createCitation(tid,sid,{locator: loc||undefined, text: text||undefined});
      setCitations([...citations, c]); setLoc(""); setText("");
    }catch(e:any){setErr(e.message)}
  };
  const deleteCitation=async(id:number)=>{
    if(!confirm("Delete citation?")) return;
    try{ await api.deleteCitation(tid,id); setCitations(citations.filter((c:any)=>c.id!==id)); }catch(e:any){setErr(e.message)}
  };

  if(loading) return <Loading msg="Loading source…" />;
  if(err) return <ErrorState msg={err} />;
  if(!source) return null;
  return <div className="space-y-6">
    <Link to={`/trees/${tid}/sources`} className="text-sm text-blue-600 underline">← Sources</Link>
    <h1 className="text-2xl font-bold">{source.title}</h1>
    <div className="border rounded p-4 bg-white space-y-2">
      {edit ? <>
        <input value={title} onChange={e=>setTitle(e.target.value)} className="w-full border rounded px-2 py-1" placeholder="Title" />
        <select value={type} onChange={e=>setType(e.target.value)} className="border rounded px-2 py-1 w-full">
          <option value="BOOK">BOOK</option><option value="REGISTER">REGISTER</option><option value="CENSUS">CENSUS</option><option value="CIVIL_RECORD">CIVIL_RECORD</option><option value="PARISH_RECORD">PARISH_RECORD</option><option value="NEWSPAPER">NEWSPAPER</option><option value="WEBSITE">WEBSITE</option><option value="OTHER">OTHER</option>
        </select>
        <input value={author} onChange={e=>setAuthor(e.target.value)} className="w-full border rounded px-2 py-1" placeholder="Author" />
        <input value={pub} onChange={e=>setPub(e.target.value)} className="w-full border rounded px-2 py-1" placeholder="Publication" />
        <input value={date} onChange={e=>setDate(e.target.value)} className="w-full border rounded px-2 py-1" placeholder="Date" />
        <div className="flex gap-2"><button onClick={save} className="px-3 py-1 bg-blue-600 text-white rounded">Save</button><button onClick={()=>setEdit(false)} className="px-3 py-1 border rounded">Cancel</button></div>
      </> : <>
        <div className="text-sm"><strong>Type:</strong> {source.type}</div>
        <div className="text-sm"><strong>Author:</strong> {source.author||"—"}</div>
        <div className="text-sm"><strong>Publication:</strong> {source.publication||"—"}</div>
        <div className="text-sm"><strong>Date:</strong> {source.date||"—"}</div>
        <div className="text-xs text-gray-600">Created {new Date(source.created_at).toLocaleString()}</div>
        <div className="flex gap-2"><button onClick={()=>setEdit(true)} className="px-3 py-1 bg-blue-600 text-white rounded">Edit</button><button onClick={remove} className="px-3 py-1 bg-red-600 text-white rounded">Delete Source</button></div>
      </>}
    </div>
    <div className="border rounded p-4 bg-white">
      <h3 className="font-semibold mb-2">Citations</h3>
      {citations.length===0 ? <div className="text-sm text-gray-500">No citations yet.</div> :
        <div className="space-y-2">{citations.map((c:any)=><div key={c.id} className="border rounded p-2 flex justify-between items-start">
          <div><div className="text-sm font-medium">{c.locator||"—"}</div><div className="text-xs text-gray-600">{c.text||"—"}</div></div>
          <button onClick={()=>deleteCitation(c.id)} className="text-xs text-red-600 underline">Delete</button>
        </div>)}</div>}
      <div className="mt-4 space-y-2 border-t pt-3">
        <div className="text-sm font-semibold">Add Citation</div>
        <input placeholder="Locator (e.g. Libro III folio 42)" value={loc} onChange={e=>setLoc(e.target.value)} className="w-full border rounded px-2 py-1" />
        <input placeholder="Text" value={text} onChange={e=>setText(e.target.value)} className="w-full border rounded px-2 py-1" />
        <button onClick={createCitation} className="px-3 py-1 bg-emerald-600 text-white rounded">Add Citation</button>
      </div>
    </div>
  </div>
}
