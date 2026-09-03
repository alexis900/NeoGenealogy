import { useEffect, useState } from "react";
import { Link, useParams, useNavigate } from "react-router-dom";
import { api } from "../api/client";
import { Loading, ErrorState } from "../components/common";
import { TaskStatusBadge } from "../components/Badges";

function statusClass(s:string){
  if(s==="ACTIVE") return "bg-emerald-600 text-white";
  if(s==="PLANNED") return "bg-blue-600 text-white";
  if(s==="COMPLETED") return "bg-gray-600 text-white";
  if(s==="ABANDONED") return "bg-red-600 text-white";
  return "bg-gray-400 text-white";
}

export default function ResearchSessionDetail(){
  const {treeId, sessionId}=useParams(); const tid=Number(treeId); const sid=Number(sessionId);
  const navigate=useNavigate();
  const [data,setData]=useState<any>(null);
  const [err,setErr]=useState<string|null>(null);
  const [loading,setLoading]=useState(true);
  const [confirmAction,setConfirmAction]=useState<string | null>(null);
  const load=async()=>{
    setLoading(true); setErr(null);
    try{
      const r=await api.getSession(tid, sid);
      setData(r);
    }catch(e:any){setErr(e.message)} finally{setLoading(false)}
  };
  useEffect(()=>{load()},[tid,sid]);
  const updateStatus=async(newStatus:string)=>{
    try{
      await api.updateSession(tid, sid, {status: newStatus});
      await load();
      setConfirmAction(null);
    }catch(e:any){setErr(e.message)}
  };
  const removeTask=async(taskId:number)=>{
    try{ await api.removeTaskFromSession(tid, taskId); await load(); }catch(e:any){setErr(e.message)}
  };
  if(loading) return <Loading msg="Loading research session…" />;
  if(err) return <ErrorState msg={err} onRetry={load} />;
  if(!data) return null;
  const session=data.session;
  const tasks=data.tasks||[];
  const summary=data.summary;
  const total=summary?.total_tasks||0;
  const progress = total>0 ? `${summary.terminal_tasks||0} / ${total}` : "No tasks yet";
  const openTasks = summary?.open_tasks||0;
  return <div className="space-y-4">
    <Link to={`/trees/${tid}/research/sessions`} className="text-sm text-blue-600 underline">← Sessions</Link>
    <div className="flex justify-between items-start gap-2">
      <h1 className="text-2xl font-bold">{session.title}</h1>
      <span className={`px-2 py-0.5 rounded text-xs font-semibold ${statusClass(session.status)}`}>{session.status}</span>
    </div>
    <div className="text-sm text-gray-600">Created {new Date(session.created_at).toLocaleDateString()} · Updated {new Date(session.updated_at).toLocaleDateString()}</div>
    <div className="border rounded p-4 bg-white">
      <h3 className="font-semibold">Objective</h3>
      <p className="text-sm text-gray-700 mt-1">{session.description || "No description"}</p>
    </div>
    {data.person && <div className="border rounded p-3 bg-white text-sm"><strong>Person</strong> <Link to={`/trees/${tid}/persons/${data.person.id}`} className="text-blue-600 underline">{data.person.name}</Link></div>}
    {data.opportunity && <div className="border rounded p-3 bg-white text-sm"><strong>Opportunity</strong> <Link to={`/trees/${tid}/research/opportunities/${data.opportunity.id}`} className="text-blue-600 underline">{data.opportunity.title} (Score {data.opportunity.score})</Link></div>}

    <div className="border rounded p-4 bg-white">
      <div className="flex justify-between items-center">
        <h3 className="font-semibold">Tasks</h3>
        <span className="text-xs text-gray-600">Progress {progress}</span>
      </div>
      {tasks.length===0 ? <p className="text-sm text-gray-500 mt-2">No tasks yet</p> :
        <div className="space-y-2 mt-2">
          {tasks.map((t:any)=><div key={t.id} className="border rounded p-3 flex justify-between items-center gap-2">
            <div>
              <Link to={`/trees/${tid}/research/tasks/${t.id}`} className="text-sm font-medium text-blue-600 underline">{t.title}</Link>
              <div className="text-xs text-gray-600 flex gap-1 items-center"><TaskStatusBadge status={t.status} /> {t.has_outcome? "Outcome recorded":"Not recorded"}</div>
            </div>
            <button onClick={()=>removeTask(t.id)} className="text-xs border rounded px-2 py-1">Remove</button>
          </div>)}
        </div>}
    </div>

    <div className="border rounded p-3 bg-gray-50 text-sm">
      <div>Progress: <strong>{progress}</strong></div>
      <div className="text-xs text-gray-600 mt-1">Total {summary.total_tasks} · Open {summary.open_tasks} · In progress {summary.in_progress_tasks} · Terminal {summary.terminal_tasks} · Outcomes {summary.outcomes_count}</div>
    </div>

    <div className="flex gap-2 flex-wrap">
      {session.status==="PLANNED" && <button onClick={()=>updateStatus("ACTIVE")} className="px-3 py-1 bg-emerald-600 text-white rounded text-sm">Start Session</button>}
      {session.status==="ACTIVE" && <button onClick={()=>setConfirmAction("COMPLETE")} className="px-3 py-1 bg-emerald-600 text-white rounded text-sm">Mark Completed</button>}
      {session.status==="ACTIVE" && <button onClick={()=>setConfirmAction("ABANDON")} className="px-3 py-1 bg-red-600 text-white rounded text-sm">Abandon</button>}
      {session.status==="PLANNED" && <button onClick={()=>setConfirmAction("ABANDON")} className="px-3 py-1 bg-red-600 text-white rounded text-sm">Abandon</button>}
      {(session.status==="COMPLETED"||session.status==="ABANDONED") && <button onClick={()=>updateStatus("ACTIVE")} className="px-3 py-1 bg-blue-600 text-white rounded text-sm">Reopen</button>}
      <button onClick={async()=>{ if(window.confirm("Delete session?")){ await api.deleteSession(tid,sid); navigate(`/trees/${tid}/research/sessions`); } }} className="px-3 py-1 border rounded text-sm">Delete</button>
    </div>

    {confirmAction==="COMPLETE" && <div className="border rounded p-4 bg-amber-50">
      <p className="text-sm font-medium">Complete this research session?</p>
      {openTasks>0 && <p className="text-xs text-amber-700 mt-1">{openTasks} tasks are still open.</p>}
      <div className="flex gap-2 mt-2">
        <button onClick={()=>setConfirmAction(null)} className="px-3 py-1 border rounded text-sm">Cancel</button>
        <button onClick={()=>updateStatus("COMPLETED")} className="px-3 py-1 bg-emerald-600 text-white rounded text-sm">Complete Session</button>
      </div>
    </div>}
    {confirmAction==="ABANDON" && <div className="border rounded p-4 bg-red-50">
      <p className="text-sm font-medium">Abandon this research session?</p>
      <div className="flex gap-2 mt-2">
        <button onClick={()=>setConfirmAction(null)} className="px-3 py-1 border rounded text-sm">Cancel</button>
        <button onClick={()=>updateStatus("ABANDONED")} className="px-3 py-1 bg-red-600 text-white rounded text-sm">Abandon Session</button>
      </div>
    </div>}

    {tasks.length>0 && <div className="border rounded p-3 bg-white text-sm">
      <h4 className="font-semibold">Outcomes</h4>
      <p className="text-gray-600">{summary.outcomes_count} outcomes linked to tasks in this session. View via tasks.</p>
    </div>}
  </div>
}
