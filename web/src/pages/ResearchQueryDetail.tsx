import { useEffect, useState } from "react";
import { useParams, Link } from "react-router-dom";
import { api } from "../api/client";
import { Loading, ErrorState } from "../components/common";

export default function ResearchQueryDetail(){
  const {treeId, queryId}=useParams(); const tid=Number(treeId); const qid=Number(queryId);
  const [query,setQuery]=useState<any>(null);
  const [executions,setExecutions]=useState<any[]>([]);
  const [results,setResults]=useState<any[]>([]);
  const [err,setErr]=useState<string|null>(null);
  const [loading,setLoading]=useState(true);
  const [running,setRunning]=useState(false);

  const load=async()=>{
    setLoading(true); setErr(null);
    try{
      const q=await api.getResearchQuery(tid,qid);
      setQuery(q);
      const execs=await api.getResearchQueryExecutions(tid,qid,{limit:20});
      setExecutions(execs.items);
      const rres=await api.getResearchQueryResults(tid,qid,{limit:50});
      setResults(rres.items);
    }catch(e:any){setErr(e.message)} finally{setLoading(false)}
  };
  useEffect(()=>{load()},[tid,qid]);

  const runAgain=async()=>{
    setRunning(true);
    try{
      await api.runResearchQuery(tid,qid);
      await load();
    }catch(e:any){setErr(e.message)} finally{setRunning(false)}
  };

  if(loading) return <Loading msg="Loading query…" />;
  if(err) return <ErrorState msg={err} />;
  if(!query) return null;
  return <div className="space-y-6">
    <Link to={`/trees/${tid}/research/tasks/${query.task_id}`} className="text-sm text-blue-600 underline">← Task {query.task_id}</Link>
    <h1 className="text-2xl font-bold">External Research Query</h1>
    <div className="border rounded p-4 bg-white space-y-2">
      <div className="text-sm"><strong>Query:</strong> {query.query}</div>
      <div className="text-sm"><strong>Provider:</strong> {query.provider}</div>
      <div className="text-sm"><strong>Status:</strong> {query.status}</div>
      {query.latest_execution && <div className="text-sm"><strong>Latest:</strong> {query.latest_execution.status} · {query.latest_execution.result_count} results</div>}
      {query.error_code && <div className="text-sm text-red-600">{query.error_code}: {query.error_message}</div>}
      <button onClick={runAgain} disabled={running} className="mt-2 px-3 py-1 bg-emerald-600 text-white rounded text-sm disabled:opacity-50">{running?"Running…":"Run Again"}</button>
    </div>
    <div className="border rounded p-4 bg-white">
      <h3 className="font-semibold">Executions ({executions.length})</h3>
      <div className="mt-2 space-y-2">
        {executions.map((e:any)=>(
          <div key={e.id} className="border rounded p-2 bg-gray-50 text-sm">
            <div>#{e.id} · {e.status} · {e.result_count ?? 0} results</div>
            <div className="text-xs text-gray-600">{new Date(e.created_at).toLocaleString()} {e.error_code ? `· ${e.error_code}` : ""}</div>
          </div>
        ))}
        {executions.length===0 && <div className="text-sm text-gray-500">No executions yet.</div>}
      </div>
    </div>
    <div className="border rounded p-4 bg-white">
      <h3 className="font-semibold">Results ({results.length}) — External Research Result (candidate, not evidence)</h3>
      <div className="text-xs text-amber-800 mt-1">This result is not evidence.</div>
      <div className="mt-3 space-y-2">
        {results.length===0 ? <div className="text-sm text-gray-500">No results yet. Run the query or retry.</div> :
          results.map((r:any)=>(
            <div key={r.id} className="border rounded p-3 bg-gray-50">
              <div className="text-sm font-semibold">{r.title}</div>
              {r.description && <div className="text-xs text-gray-600">{r.description}</div>}
              <div className="text-xs text-gray-500 mt-1">{r.record_type||""} {r.date?`· ${r.date}`:""} {r.place?`· ${r.place}`:""} · {r.position}</div>
              {r.url && <a href={r.url} target="_blank" rel="noopener noreferrer" className="text-xs text-blue-600 underline">Open external source</a>}
              <div className="text-xs mt-1"><span className="px-2 py-0.5 bg-yellow-100 border rounded">External Research Result</span> This result is not evidence.</div>
              <Link to={`/trees/${tid}/research/results/${r.id}`} className="text-xs text-blue-600 underline ml-2">View Detail</Link>
            </div>
          ))}
      </div>
    </div>
  </div>
}
