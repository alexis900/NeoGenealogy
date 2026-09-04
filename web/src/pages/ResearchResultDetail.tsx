import { useEffect, useState } from "react";
import { useParams, Link } from "react-router-dom";
import { api } from "../api/client";
import { Loading, ErrorState } from "../components/common";

export default function ResearchResultDetail(){
  const {treeId, resultId}=useParams(); const tid=Number(treeId); const rid=Number(resultId);
  const [result,setResult]=useState<any>(null);
  const [err,setErr]=useState<string|null>(null);
  const [loading,setLoading]=useState(true);
  useEffect(()=>{
    const load=async()=>{
      setLoading(true); setErr(null);
      try{
        const r=await api.getResearchResult(tid,rid);
        setResult(r);
      }catch(e:any){setErr(e.message)} finally{setLoading(false)}
    };
    load();
  },[tid,rid]);
  if(loading) return <Loading msg="Loading result…" />;
  if(err) return <ErrorState msg={err} />;
  if(!result) return null;
  const isValidUrl = ()=>{
    try{
      const u=new URL(result.url);
      return u.protocol==="http:"||u.protocol==="https:";
    }catch{ return false; }
  };
  return <div className="space-y-6">
    <Link to={`/trees/${tid}/research/queries/${result.query_id}`} className="text-sm text-blue-600 underline">← Query {result.query_id}</Link>
    <h1 className="text-2xl font-bold">External Research Result</h1>
    <div className="border rounded p-4 bg-yellow-50 border-yellow-200">
      <div className="text-sm font-semibold">External Research Result — Candidate</div>
      <div className="text-sm text-amber-800">This result is not evidence. Possible matching record — not confirmed.</div>
    </div>
    <div className="border rounded p-4 bg-white space-y-2">
      <div className="text-sm"><strong>Title:</strong> {result.title}</div>
      {result.description && <div className="text-sm"><strong>Description:</strong> {result.description}</div>}
      <div className="text-sm"><strong>Provider:</strong> {result.provider}</div>
      {result.external_id && <div className="text-sm"><strong>External ID:</strong> {result.external_id}</div>}
      {result.record_type && <div className="text-sm"><strong>Record Type:</strong> {result.record_type}</div>}
      {result.date && <div className="text-sm"><strong>Date:</strong> {result.date}</div>}
      {result.place && <div className="text-sm"><strong>Place:</strong> {result.place}</div>}
      {result.url && <div className="text-sm"><strong>URL:</strong> {isValidUrl() ? <a href={result.url} target="_blank" rel="noopener noreferrer" className="text-blue-600 underline">Open external source</a> : <span className="text-red-600">Invalid URL</span>} <span className="ml-2 text-xs font-mono">{result.url}</span></div>}
      <div className="text-sm"><strong>Position:</strong> {result.position}</div>
      {result.metadata && <div className="text-sm"><strong>Metadata:</strong> <pre className="text-xs bg-gray-50 p-2 rounded">{JSON.stringify(result.metadata,null,2)}</pre></div>}
      <div className="text-xs text-gray-600 mt-2">Review Result — future phases may allow converting this candidate into Source/Citation/Evidence via explicit user action.</div>
      <div className="flex gap-2 mt-3">
        {result.url && isValidUrl() && <a href={result.url} target="_blank" rel="noopener noreferrer" className="px-3 py-1 bg-blue-600 text-white rounded text-sm">Open external source</a>}
        <button className="px-3 py-1 border rounded text-sm" disabled>Review Result (future)</button>
      </div>
    </div>
  </div>
}
