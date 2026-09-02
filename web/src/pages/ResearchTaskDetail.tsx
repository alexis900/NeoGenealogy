import { useEffect, useState } from "react";
import { useParams, Link } from "react-router-dom";
import { api } from "../api/client";
import { Loading, ErrorState } from "../components/common";
import { TaskStatusBadge, PriorityBadge, ScoreBadge } from "../components/Badges";
import { ScoreBreakdown } from "../components/OpportunityCard";

function formatOutcomeType(t:string){
  const map:any={"CONFIRMED":"Confirmed","FALSE_LEAD":"False lead","INCONCLUSIVE":"Inconclusive","NEW_LEAD":"New lead","NO_EVIDENCE":"No evidence found"};
  return map[t]||t;
}

export default function ResearchTaskDetail(){
  const {treeId, taskId}=useParams(); const tid=Number(treeId); const id=Number(taskId);
  const [task,setTask]=useState<any>(null); const [opp,setOpp]=useState<any>(null);
  const [err,setErr]=useState<string|null>(null); const [loading,setLoading]=useState(true);
  const [editTitle,setEditTitle]=useState(""); const [editDesc,setEditDesc]=useState("");
  const [editStatus,setEditStatus]=useState(""); const [editResolution,setEditResolution]=useState("");
  const [saving,setSaving]=useState(false);
  const [outcomeType,setOutcomeType]=useState("CONFIRMED");
  const [outcomeSummary,setOutcomeSummary]=useState(""); const [outcomeDetails,setOutcomeDetails]=useState("");
  const [outcomeSaving,setOutcomeSaving]=useState(false);

  const load=async()=>{
    setLoading(true); setErr(null);
    try{
      const t=await api.getTask(tid,id);
      setTask(t); setEditTitle(t.title); setEditDesc(t.description||""); setEditStatus(t.status); setEditResolution(t.resolution||"");
      if(t.outcome){
        setOutcomeType(t.outcome.type); setOutcomeSummary(t.outcome.summary); setOutcomeDetails(t.outcome.details||"");
      } else {
        setOutcomeType("CONFIRMED"); setOutcomeSummary(""); setOutcomeDetails("");
      }
      if(t.opportunity_id){
        try{
          const opps=await api.getOpportunities(tid,{limit:100});
          const found=opps.items.find((x:any)=> x.id===t.opportunity_id);
          if(found) setOpp(found);
        }catch{}
      }
    }catch(e:any){setErr(e.message)} finally{setLoading(false)}
  };
  useEffect(()=>{load()},[tid,id]);

  const save=async()=>{
    setSaving(true);
    try{
      const updated=await api.updateTask(tid,id,{title:editTitle, description:editDesc, status:editStatus, resolution:editResolution||undefined});
      setTask(updated);
    }catch(e:any){setErr(e.message)} finally{setSaving(false)}
  };
  const remove=async()=>{
    if(!confirm("Delete this research task?")) return;
    try{ await api.deleteTask(tid,id); window.location.href=`/trees/${tid}/research/tasks`; }catch(e:any){setErr(e.message)}
  };
  const recordOutcome=async()=>{
    setOutcomeSaving(true);
    try{
      const o=await api.createOutcome(tid,id,{type:outcomeType, summary:outcomeSummary, details:outcomeDetails||undefined});
      setTask({...task, outcome:o});
    }catch(e:any){setErr(e.message)} finally{setOutcomeSaving(false)}
  };
  const updateOutcome=async()=>{
    if(!task.outcome) return;
    setOutcomeSaving(true);
    try{
      const o=await api.updateOutcome(tid,task.outcome.id,{type:outcomeType, summary:outcomeSummary, details:outcomeDetails||undefined});
      setTask({...task, outcome:o});
    }catch(e:any){setErr(e.message)} finally{setOutcomeSaving(false)}
  };
  const deleteOutcome=async()=>{
    if(!task.outcome) return;
    if(!confirm("Delete outcome?")) return;
    try{
      await api.deleteOutcome(tid,task.outcome.id);
      setTask({...task, outcome:null}); setOutcomeSummary(""); setOutcomeDetails("");
    }catch(e:any){setErr(e.message)}
  };

  if(loading) return <Loading msg="Loading task…" />;
  if(err) return <ErrorState msg={err} />;
  if(!task) return null;
  return <div className="space-y-6">
    <Link to={`/trees/${tid}/research/tasks`} className="text-sm text-blue-600 underline">← Research Tasks</Link>
    <div className="flex justify-between items-start gap-4">
      <h1 className="text-2xl font-bold">{task.title}</h1>
      <TaskStatusBadge status={task.status} />
    </div>
    <div className="grid gap-4 border rounded p-4 bg-white">
      <label className="block"><span className="text-sm font-semibold">Title</span><input value={editTitle} onChange={e=>setEditTitle(e.target.value)} className="w-full border rounded px-2 py-1 mt-1" /></label>
      <label className="block"><span className="text-sm font-semibold">Description</span><textarea value={editDesc} onChange={e=>setEditDesc(e.target.value)} className="w-full border rounded px-2 py-1 mt-1" rows={3} /></label>
      <label className="block"><span className="text-sm font-semibold">Status</span>
        <select value={editStatus} onChange={e=>setEditStatus(e.target.value)} className="border rounded px-2 py-1 mt-1">
          <option value="OPEN">OPEN</option><option value="IN_PROGRESS">IN_PROGRESS</option><option value="RESOLVED">RESOLVED</option><option value="REJECTED">REJECTED</option><option value="INCONCLUSIVE">INCONCLUSIVE</option>
        </select>
      </label>
      {(editStatus==="RESOLVED"||editStatus==="REJECTED"||editStatus==="INCONCLUSIVE") && (
        <label className="block"><span className="text-sm font-semibold">Resolution</span><textarea value={editResolution} onChange={e=>setEditResolution(e.target.value)} className="w-full border rounded px-2 py-1 mt-1" rows={4} placeholder="Conclusion..." /></label>
      )}
      <div className="flex gap-2">
        <button onClick={save} disabled={saving} className="px-4 py-2 bg-blue-600 text-white rounded disabled:opacity-50">{saving?"Saving…":"Save"}</button>
        <button onClick={remove} className="px-4 py-2 bg-red-600 text-white rounded">Delete</button>
      </div>
    </div>

    <div className="border rounded p-4 bg-white">
      <h3 className="font-semibold mb-2">Research Outcome</h3>
      {task.outcome ? (
        <div className="space-y-3">
          <div className="flex gap-2 items-center"><span className="px-2 py-1 bg-emerald-100 rounded text-sm font-semibold">{formatOutcomeType(task.outcome.type)}</span><span className="text-xs text-gray-600">{new Date(task.outcome.created_at).toLocaleString()}</span></div>
          <div><div className="text-sm font-semibold">Summary</div><div className="text-sm">{task.outcome.summary}</div></div>
          {task.outcome.details && <div><div className="text-sm font-semibold">Details</div><div className="text-sm whitespace-pre-wrap">{task.outcome.details}</div></div>}
          <div className="border-t pt-3 space-y-2">
            <div className="text-sm font-semibold">Edit Outcome</div>
            <select value={outcomeType} onChange={e=>setOutcomeType(e.target.value)} className="border rounded px-2 py-1">
              <option value="CONFIRMED">Confirmed</option><option value="FALSE_LEAD">False lead</option><option value="INCONCLUSIVE">Inconclusive</option><option value="NEW_LEAD">New lead</option><option value="NO_EVIDENCE">No evidence found</option>
            </select>
            <input value={outcomeSummary} onChange={e=>setOutcomeSummary(e.target.value)} placeholder="Summary" className="w-full border rounded px-2 py-1" />
            <textarea value={outcomeDetails} onChange={e=>setOutcomeDetails(e.target.value)} placeholder="Details" className="w-full border rounded px-2 py-1" rows={3} />
            <div className="flex gap-2">
              <button onClick={updateOutcome} disabled={outcomeSaving} className="px-3 py-1 bg-blue-600 text-white rounded">Update Outcome</button>
              <button onClick={deleteOutcome} className="px-3 py-1 bg-red-600 text-white rounded">Delete Outcome</button>
            </div>
          </div>
        </div>
      ) : (
        <div className="space-y-3">
          <div className="text-sm text-gray-600">No research outcome recorded yet.</div>
          <select value={outcomeType} onChange={e=>setOutcomeType(e.target.value)} className="border rounded px-2 py-1">
            <option value="CONFIRMED">Confirmed</option><option value="FALSE_LEAD">False lead</option><option value="INCONCLUSIVE">Inconclusive</option><option value="NEW_LEAD">New lead</option><option value="NO_EVIDENCE">No evidence found</option>
          </select>
          <input value={outcomeSummary} onChange={e=>setOutcomeSummary(e.target.value)} placeholder="Summary (required)" className="w-full border rounded px-2 py-1" />
          <textarea value={outcomeDetails} onChange={e=>setOutcomeDetails(e.target.value)} placeholder="Details (optional)" className="w-full border rounded px-2 py-1" rows={3} />
          <button onClick={recordOutcome} disabled={outcomeSaving || !outcomeSummary.trim()} className="px-4 py-2 bg-emerald-600 text-white rounded disabled:opacity-50">{outcomeSaving?"Saving…":"Record Outcome"}</button>
        </div>
      )}
    </div>

    <div className="text-xs text-gray-600 space-y-1">
      <div>Created: {new Date(task.created_at).toLocaleString()}</div>
      <div>Updated: {new Date(task.updated_at).toLocaleString()}</div>
      <div>Started: {task.started_at? new Date(task.started_at).toLocaleString():"—"}</div>
      <div>Completed: {task.completed_at? new Date(task.completed_at).toLocaleString():"—"}</div>
      <div>Person: {task.person_id? <Link to={`/trees/${tid}/persons/${task.person_id}`} className="text-blue-600 underline">{task.person_id}</Link>:"—"}</div>
    </div>
    {opp && <div className="border rounded p-4 bg-gray-50">
      <h3 className="font-semibold mb-2">Original Research Opportunity</h3>
      <div className="flex gap-2 items-center"><PriorityBadge p={opp.priority} /><ScoreBadge score={opp.score} /></div>
      <div className="text-sm mt-2"><strong>Why:</strong> {opp.why}</div>
      <div className="text-sm"><strong>What:</strong> {JSON.stringify(opp.what)}</div>
      <div className="text-sm"><strong>Sources:</strong> {JSON.stringify(opp.potential_sources)}</div>
      {opp.breakdown && <div className="mt-2"><ScoreBreakdown breakdown={opp.breakdown} /></div>}
      <Link to={`/trees/${tid}/research`} className="text-sm text-blue-600 underline">Back to Queue</Link>
    </div>}
  </div>
}
