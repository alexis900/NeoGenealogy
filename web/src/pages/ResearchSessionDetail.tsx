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
  const stats=data.stats;
  const timeline=data.timeline||[];
  const total=summary?.total_tasks||0;
  const progress = total>0 ? `${summary.terminal_tasks||0} / ${total}` : "No tasks yet";
  const taskProgress = stats ? `${stats.completed_tasks} / ${stats.total_tasks}` : progress;
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

    {/* Session Summary - derived stats */}
    {stats && (
      <div className="border rounded p-4 bg-white">
        <h3 className="font-semibold">Session Summary</h3>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-3 mt-2 text-sm">
          <div>
            <div className="font-medium text-xs text-gray-600">Tasks</div>
            <div className="text-xs mt-1 space-y-0.5">
              <div>{stats.total_tasks} total</div>
              <div>{stats.completed_tasks} completed</div>
              <div>{stats.open_tasks} open</div>
              {stats.in_progress_tasks>0 && <div>{stats.in_progress_tasks} in progress</div>}
              {stats.inconclusive_tasks>0 && <div>{stats.inconclusive_tasks} inconclusive</div>}
              {stats.rejected_tasks>0 && <div>{stats.rejected_tasks} rejected</div>}
            </div>
            <div className="text-xs text-gray-600 mt-1">Task progress {taskProgress}</div>
          </div>
          <div>
            <div className="font-medium text-xs text-gray-600">Outcomes</div>
            <div className="text-xs mt-1 space-y-0.5">
              <div>{stats.total_outcomes} total</div>
              {stats.total_outcomes>0 && (
                <>
                  {stats.confirmed_outcomes>0 && <div>Confirmed {stats.confirmed_outcomes}</div>}
                  {stats.false_lead_outcomes>0 && <div>False leads {stats.false_lead_outcomes}</div>}
                  {stats.inconclusive_outcomes>0 && <div>Inconclusive {stats.inconclusive_outcomes}</div>}
                  {stats.new_lead_outcomes>0 && <div>New leads {stats.new_lead_outcomes}</div>}
                  {stats.no_evidence_outcomes>0 && <div>No evidence {stats.no_evidence_outcomes}</div>}
                  {stats.total_outcomes===0 && <div>0 outcomes</div>}
                </>
              )}
              {stats.total_outcomes===0 && <div className="text-gray-500">0 outcomes</div>}
            </div>
          </div>
          <div>
            <div className="font-medium text-xs text-gray-600">Evidence</div>
            <div className="text-xs mt-1 space-y-0.5">
              <div>{stats.total_evidence} total</div>
              {stats.total_evidence>0 && (
                <>
                  <div>{stats.supporting_evidence} supporting</div>
                  <div>{stats.contradicting_evidence} contradicting</div>
                </>
              )}
              {stats.total_evidence===0 && <div className="text-gray-500">No evidence</div>}
            </div>
          </div>
          <div>
            <div className="font-medium text-xs text-gray-600">Follow-ups</div>
            <div className="text-xs mt-1 space-y-0.5">
              <div>{stats.open_followups} open</div>
              <div>{stats.completed_followup_actions} completed</div>
              <div>{stats.skipped_followup_actions} skipped</div>
            </div>
          </div>
        </div>
        <div className="text-xs text-gray-500 mt-3">Progress {progress} · Terminal tasks include resolved, rejected and inconclusive.</div>
      </div>
    )}

    {/* Research Activity - links */}
    {stats && (
      <div className="border rounded p-4 bg-white">
        <h3 className="font-semibold">Research Activity</h3>
        <div className="text-sm mt-2 space-y-1">
          <div>Tasks: {stats.total_tasks} total · {stats.open_tasks} open · {stats.in_progress_tasks} in progress</div>
          <div>Outcomes: {stats.total_outcomes} {stats.total_outcomes>0 && <Link to={`/trees/${tid}/research/history`} className="text-blue-600 underline ml-1">View outcomes via tasks</Link>}</div>
          <div>Evidence: {stats.total_evidence} {stats.total_evidence>0 && <><span className="text-gray-600">({stats.supporting_evidence} supporting, {stats.contradicting_evidence} contradicting)</span> <Link to={`/trees/${tid}/evidence`} className="text-blue-600 underline ml-1">View Evidence</Link></>}</div>
          <div>Follow-up actions: {stats.open_followups + stats.completed_followup_actions + stats.skipped_followup_actions} total</div>
        </div>
      </div>
    )}

    <div className="border rounded p-4 bg-white">
      <div className="flex justify-between items-center">
        <h3 className="font-semibold">Tasks</h3>
        <span className="text-xs text-gray-600">Task progress {taskProgress} · Progress {progress}</span>
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
      <div>Task progress: <strong>{taskProgress}</strong></div>
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

    {/* Activity timeline - derived, no event table */}
    <div className="border rounded p-4 bg-white">
      <h3 className="font-semibold">Activity</h3>
      {timeline.length===0 ? <p className="text-sm text-gray-500 mt-2">No activity yet.</p> : (
        <div className="mt-2 space-y-2">
          {timeline.map((ev:any, idx:number)=>(
            <div key={idx} className="flex gap-3 text-sm border-b pb-1 last:border-0">
              <span className="text-xs text-gray-500 whitespace-nowrap">{new Date(ev.timestamp).toLocaleDateString()}</span>
              <span className="text-xs font-mono bg-gray-100 rounded px-1">{ev.event_type}</span>
              <span className="text-xs">{ev.label}</span>
            </div>
          ))}
        </div>
      )}
      <p className="text-xs text-gray-500 mt-2">Activity timeline is derived from existing timestamps and does not claim to capture every action.</p>
    </div>
  </div>
}
