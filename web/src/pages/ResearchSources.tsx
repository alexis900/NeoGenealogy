import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { api } from "../api/client";
import { Loading, ErrorState, Empty, Pagination } from "../components/common";

export default function ResearchSources(){
  const {treeId}=useParams(); const id=Number(treeId);
  const [items,setItems]=useState<any[]>([]);
  const [type,setType]=useState("");
  const [total,setTotal]=useState(0); const [offset,setOffset]=useState(0); const limit=20;
  const [err,setErr]=useState<string|null>(null); const [loading,setLoading]=useState(true);
  const [showCreate,setShowCreate]=useState(false);
  const [title,setTitle]=useState(""); const [author,setAuthor]=useState(""); const [publication,setPublication]=useState(""); const [date,setDate]=useState(""); const [stype,setStype]=useState("BOOK");

  const load=async(o:number)=>{
    setLoading(true); setErr(null);
    try{
      const r=await api.getSources(id,{type:type||undefined, limit, offset:o});
      setItems(r.items); setTotal(r.pagination.total); setOffset(o);
    }catch(e:any){setErr(e.message)} finally{setLoading(false)}
  };
  useEffect(()=>{load(0)},[id, type]);

  const create=async()=>{
    try{
      await api.createSource(id,{title, author: author||undefined, publication: publication||undefined, date: date||undefined, type: stype});
      setShowCreate(false); setTitle(""); setAuthor(""); setPublication(""); setDate(""); setStype("BOOK");
      load(0);
    }catch(e:any){setErr(e.message)}
  };

  return <div className="space-y-4">
    <div className="flex justify-between items-center">
      <h1 className="text-2xl font-bold">Sources</h1>
      <Link to={`/trees/${id}/research`} className="text-sm text-blue-600 underline">← Research</Link>
    </div>
    <p className="text-sm text-gray-600">Bibliographic sources that support evidence.</p>
    <div className="flex gap-2 flex-wrap">
      <select value={type} onChange={e=>setType(e.target.value)} className="border rounded px-2 py-1">
        <option value="">All types</option><option value="BOOK">BOOK</option><option value="REGISTER">REGISTER</option><option value="CENSUS">CENSUS</option><option value="CIVIL_RECORD">CIVIL_RECORD</option><option value="PARISH_RECORD">PARISH_RECORD</option><option value="NEWSPAPER">NEWSPAPER</option><option value="WEBSITE">WEBSITE</option><option value="OTHER">OTHER</option>
      </select>
      <button onClick={()=>setShowCreate(!showCreate)} className="px-3 py-1 bg-emerald-600 text-white rounded text-sm">{showCreate?"Cancel":"Create Source"}</button>
    </div>
    {showCreate && <div className="border rounded p-4 bg-white space-y-2">
      <input placeholder="Title (required)" value={title} onChange={e=>setTitle(e.target.value)} className="w-full border rounded px-2 py-1" />
      <select value={stype} onChange={e=>setStype(e.target.value)} className="border rounded px-2 py-1 w-full">
        <option value="BOOK">BOOK</option><option value="REGISTER">REGISTER</option><option value="CENSUS">CENSUS</option><option value="CIVIL_RECORD">CIVIL_RECORD</option><option value="PARISH_RECORD">PARISH_RECORD</option><option value="NEWSPAPER">NEWSPAPER</option><option value="WEBSITE">WEBSITE</option><option value="OTHER">OTHER</option>
      </select>
      <input placeholder="Author" value={author} onChange={e=>setAuthor(e.target.value)} className="w-full border rounded px-2 py-1" />
      <input placeholder="Publication" value={publication} onChange={e=>setPublication(e.target.value)} className="w-full border rounded px-2 py-1" />
      <input placeholder="Date" value={date} onChange={e=>setDate(e.target.value)} className="w-full border rounded px-2 py-1" />
      <button onClick={create} disabled={!title.trim()} className="px-4 py-2 bg-blue-600 text-white rounded disabled:opacity-50">Save</button>
    </div>}
    {loading ? <Loading msg="Loading sources…" /> : err ? <ErrorState msg={err} onRetry={()=>load(offset)} /> : items.length===0 ? <Empty msg="No sources yet. Create one to support evidence." /> :
      <div className="space-y-2">
        {items.map((s:any)=><Link key={s.id} to={`/trees/${id}/sources/${s.id}`} className="block border rounded p-4 hover:bg-gray-50">
          <div className="font-semibold">{s.title}</div>
          <div className="text-xs text-gray-600">Type {s.type} · Author {s.author||"—"} · Date {s.date||"—"} · Publication {s.publication||"—"}</div>
          <div className="text-xs text-blue-600 mt-1">View Source →</div>
        </Link>)}
        <Pagination limit={limit} offset={offset} total={total} onChange={load} />
      </div>
    }
  </div>
}
