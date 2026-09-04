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
function formatAssessmentStatus(s:string){
  return s.replace(/_/g," ");
}

function gapIcon(severity:string){
  if(severity==="CRITICAL" || severity==="WARNING") return "⚠";
  if(severity==="INFO") return "ℹ";
  return "•";
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
  const [evidence,setEvidence]=useState<any[]>([]);
  const [assessment,setAssessment]=useState<any>(null);
  const [gaps,setGaps]=useState<any[]>([]);
  const [followups,setFollowups]=useState<any[]>([]);
  const [followupActions,setFollowupActions]=useState<any[]>([]);
  const [followupNotes,setFollowupNotes]=useState<{[key:number]:string}>({});
  const [sources,setSources]=useState<any[]>([]);
  const [citations,setCitations]=useState<any[]>([]);
  const [showAddEvidence,setShowAddEvidence]=useState(false);
  const [evSourceId,setEvSourceId]=useState(""); const [evCitationId,setEvCitationId]=useState(""); const [evStatement,setEvStatement]=useState(""); const [evNotes,setEvNotes]=useState(""); const [evRelationship,setEvRelationship]=useState("SUPPORTS");
  const [evSaving,setEvSaving]=useState(false);
  const [caseSummary,setCaseSummary]=useState<any>(null);
  const [caseSummaryLoading,setCaseSummaryLoading]=useState(true);
  const [caseSummaryError,setCaseSummaryError]=useState<string|null>(null);
  const [sessions,setSessions]=useState<any[]>([]);
  const [selectedSession,setSelectedSession]=useState<string>("");
  const [sessionSaving,setSessionSaving]=useState(false);
  // External Research
  const [extQueries,setExtQueries]=useState<any[]>([]);
  const [extProvider,setExtProvider]=useState("mock");
  const [extProviders,setExtProviders]=useState<any[]>([]);
  const [extProvidersLoading,setExtProvidersLoading]=useState(true);
  const [extQueryText,setExtQueryText]=useState("");
  const [extCreating,setExtCreating]=useState(false);
  const [extRunning,setExtRunning]=useState<{[key:number]:boolean}>({});
  const [extResults,setExtResults]=useState<{[key:number]:any[]}>({});
  const [extResultsLoading,setExtResultsLoading]=useState<{[key:number]:boolean}>({});

  const load=async()=>{
    setLoading(true); setErr(null);
    try{
      const t=await api.getTask(tid,id);
      setTask(t); setEditTitle(t.title); setEditDesc(t.description||""); setEditStatus(t.status); setEditResolution(t.resolution||"");
      if(t.outcome){
        setOutcomeType(t.outcome.type); setOutcomeSummary(t.outcome.summary); setOutcomeDetails(t.outcome.details||"");
        try{
          const detailed=await api.getOutcome(tid,t.outcome.id);
          setEvidence(detailed.evidence||[]);
          setAssessment((detailed as any).evidence_assessment || null);
          setGaps((detailed as any).evidence_gaps || []);
          setFollowups((detailed as any).research_followups || []);
          setFollowupActions((detailed as any).followup_actions || []);
        }catch{ setEvidence([]); setAssessment(null); setGaps([]); setFollowups([]); setFollowupActions([]); }
      } else {
        setOutcomeType("CONFIRMED"); setOutcomeSummary(""); setOutcomeDetails(""); setEvidence([]); setAssessment(null); setGaps([]); setFollowups([]); setFollowupActions([]);
      }
      if(t.opportunity_id){
        try{
          const opps=await api.getOpportunities(tid,{limit:100});
          const found=opps.items.find((x:any)=> x.id===t.opportunity_id);
          if(found) setOpp(found);
        }catch{}
      }
      try{
        const srcs=await api.getSources(tid,{limit:100});
        setSources(srcs.items);
      }catch{}
      try{
        const sess=await api.getSessions(tid,{limit:50});
        setSessions(sess.items);
      }catch{}
      // Load providers
      try{
        setExtProvidersLoading(true);
        const prov = await api.getResearchProviders(tid);
        setExtProviders(prov.providers);
        // default to first configured provider if mock not configured? mock always configured
        const fs = prov.providers.find((p:any)=>p.name==="familysearch");
        if(fs && fs.configured) {
          // keep mock as default but provider list will show both
        }
      }catch{
        setExtProviders([{name:"mock", display_name:"Mock", configured:true, enabled:true, status:"configured"}]);
      } finally{ setExtProvidersLoading(false); }
      // Load external research queries
      try{
        const qres = await api.getResearchQueriesForTask(tid, id, {limit:50});
        setExtQueries(qres.items);
        // auto fill query text from task title if empty
        if(!extQueryText && t.title){
          setExtQueryText(`${t.title} baptism 1882 Sant Martí`);
        }
        for(const q of qres.items){
          if(q.latest_execution && q.latest_execution.status==="COMPLETED"){
            try{
              const rres = await api.getResearchQueryResults(tid, q.id, {limit:50});
              setExtResults(prev=>({...prev, [q.id]: rres.items}));
            }catch{}
          }
        }
      }catch{}
      // Load case summary (derived view, never blocks main task)
      try{
        setCaseSummaryLoading(true); setCaseSummaryError(null);
        const cs = await (api as any).getCaseSummary ? await (api as any).getCaseSummary(tid,id) : null;
        if(cs) setCaseSummary(cs);
      }catch(e:any){ setCaseSummaryError(e.message); setCaseSummary(null); } finally{ setCaseSummaryLoading(false); }
    }catch(e:any){setErr(e.message)} finally{setLoading(false)}
  };
  useEffect(()=>{load()},[tid,id]);

  useEffect(()=>{
    if(evSourceId){
      api.getCitations(tid, Number(evSourceId)).then(r=>setCitations(r.items)).catch(()=>setCitations([]));
    } else { setCitations([]); setEvCitationId(""); }
  },[evSourceId]);

  const save=async()=>{
    setSaving(true);
    try{
      const updated=await api.updateTask(tid,id,{title:editTitle, description:editDesc, status:editStatus, resolution:editResolution||undefined});
      setTask(updated);
      setEditStatus(updated.status); setEditResolution(updated.resolution||"");
      try{ const cs=await (api as any).getCaseSummary?.(tid,id); if(cs) setCaseSummary(cs); }catch{}
    }catch(e:any){setErr(e.message)} finally{setSaving(false)}
  };
  const quickStatus=async(status:string)=>{
    setSaving(true);
    try{
      const updated=await api.updateTask(tid,id,{status});
      setTask(updated); setEditStatus(updated.status);
      try{ const cs=await (api as any).getCaseSummary?.(tid,id); if(cs) setCaseSummary(cs); }catch{}
    }catch(e:any){setErr(e.message)} finally{setSaving(false)}
  };
  const remove=async()=>{
    if(!confirm("Delete this research task?")) return;
    try{ await api.deleteTask(tid,id); window.location.href=`/trees/${tid}/research/tasks`; }catch(e:any){setErr(e.message)}
  };
  const attachToSession=async()=>{
    if(!selectedSession) return;
    setSessionSaving(true);
    try{
      await api.assignTaskToSession(tid,id,Number(selectedSession));
      const t=await api.getTask(tid,id);
      setTask(t);
    }catch(e:any){setErr(e.message)} finally{setSessionSaving(false)}
  };
  const detachFromSession=async()=>{
    setSessionSaving(true);
    try{
      await api.removeTaskFromSession(tid,id);
      const t=await api.getTask(tid,id);
      setTask(t);
    }catch(e:any){setErr(e.message)} finally{setSessionSaving(false)}
  };
  const createNewSessionFromTask=async()=>{
    const title=window.prompt("Session title", `Research session for ${task.title}`);
    if(!title) return;
    setSessionSaving(true);
    try{
      const sess:any=await api.createSession(tid,{title, person_id: task.person_id||undefined, opportunity_id: task.opportunity_id||undefined});
      const sid=sess.session?.id || sess.id;
      if(sid) await api.assignTaskToSession(tid,id,sid);
      const t=await api.getTask(tid,id);
      setTask(t);
      const s=await api.getSessions(tid,{limit:50}); setSessions(s.items);
    }catch(e:any){setErr(e.message)} finally{setSessionSaving(false)}
  };
  const recordOutcome=async()=>{
    setOutcomeSaving(true);
    try{
      const o=await api.createOutcome(tid,id,{type:outcomeType, summary:outcomeSummary, details:outcomeDetails||undefined});
      try{
        const detailed=await api.getOutcome(tid, o.id);
          if((detailed as any).id===o.id){
            setTask({...task, outcome:detailed}); setEvidence((detailed as any).evidence||[]); setAssessment((detailed as any).evidence_assessment||null); setGaps((detailed as any).evidence_gaps||[]); setFollowups((detailed as any).research_followups||[]); setFollowupActions((detailed as any).followup_actions||[]);
          } else {
            setTask({...task, outcome:o}); setEvidence([]); setAssessment(null); setGaps([]); setFollowups([]); setFollowupActions([]);
          }
        }catch{
          setTask({...task, outcome:o}); setEvidence([]); setAssessment(null); setGaps([]); setFollowups([]); setFollowupActions([]);
        }
    }catch(e:any){setErr(e.message)} finally{setOutcomeSaving(false)}
  };
  const updateOutcome=async()=>{
    if(!task.outcome) return;
    setOutcomeSaving(true);
    try{
      const o=await api.updateOutcome(tid,task.outcome.id,{type:outcomeType, summary:outcomeSummary, details:outcomeDetails||undefined});
      try{
        const detailed=await api.getOutcome(tid,task.outcome.id);
        if((detailed as any).id===task.outcome.id){
          setTask({...task, outcome:detailed}); setEvidence((detailed as any).evidence||[]); setAssessment((detailed as any).evidence_assessment||null); setGaps((detailed as any).evidence_gaps||[]); setFollowups((detailed as any).research_followups||[]); setFollowupActions((detailed as any).followup_actions||[]);
        } else {
          setTask({...task, outcome:o});
        }
      }catch{
        setTask({...task, outcome:o});
      }
    }catch(e:any){setErr(e.message)} finally{setOutcomeSaving(false)}
  };
  const deleteOutcome=async()=>{
    if(!task.outcome) return;
    if(!confirm("Delete outcome?")) return;
    try{
      await api.deleteOutcome(tid,task.outcome.id);
      setTask({...task, outcome:null}); setOutcomeSummary(""); setOutcomeDetails(""); setEvidence([]); setAssessment(null); setGaps([]); setFollowups([]); setFollowupActions([]);
    }catch(e:any){setErr(e.message)}
  };
  const addEvidence=async()=>{
    if(!task.outcome) return;
    if(!evSourceId || !evStatement.trim()) return;
    setEvSaving(true);
    try{
      const ev=await api.createEvidence(tid,{source_id: Number(evSourceId), citation_id: evCitationId?Number(evCitationId):undefined, statement: evStatement, notes: evNotes||undefined});
      await api.attachEvidence(tid, task.outcome.id, ev.id, {relationship: evRelationship});
      const detailed=await api.getOutcome(tid, task.outcome.id);
      setEvidence(detailed.evidence||[]);
      setAssessment((detailed as any).evidence_assessment||null);
      setGaps((detailed as any).evidence_gaps||[]);
      setFollowups((detailed as any).research_followups||[]);
      setFollowupActions((detailed as any).followup_actions||[]);
      setEvStatement(""); setEvNotes(""); setEvCitationId(""); setEvSourceId(""); setShowAddEvidence(false);
    }catch(e:any){setErr(e.message)} finally{setEvSaving(false)}
  };
  const removeEvidence=async(evidenceId:number)=>{
    if(!task.outcome) return;
    if(!confirm("Remove evidence from outcome?")) return;
    try{
      await api.detachEvidence(tid, task.outcome.id, evidenceId);
      const detailed=await api.getOutcome(tid, task.outcome.id);
      setEvidence(detailed.evidence||[]);
      setAssessment((detailed as any).evidence_assessment||null);
      setGaps((detailed as any).evidence_gaps||[]);
      setFollowups((detailed as any).research_followups||[]);
      setFollowupActions((detailed as any).followup_actions||[]);
    }catch(e:any){setErr(e.message)}
  };

  const startFollowup=async(followupCode:string)=>{
    if(!task.outcome) return;
    try{
      await api.createFollowupAction(tid, task.outcome.id, {followup_code: followupCode});
      const detailed=await api.getOutcome(tid, task.outcome.id);
      setFollowupActions((detailed as any).followup_actions||[]);
      setFollowups((detailed as any).research_followups||[]);
    }catch(e:any){setErr(e.message)}
  };
  const updateFollowupStatus=async(actionId:number, status:string)=>{
    try{
      const notes = followupNotes[actionId];
      await api.updateFollowupAction(tid, actionId, {status, notes: notes!==undefined?notes:undefined});
      const detailed=await api.getOutcome(tid, task.outcome.id);
      setFollowupActions((detailed as any).followup_actions||[]);
    }catch(e:any){setErr(e.message)}
  };
  const deleteFollowupAction=async(actionId:number)=>{
    if(!confirm("Delete follow-up action?")) return;
    try{
      await api.deleteFollowupAction(tid, actionId);
      const detailed=await api.getOutcome(tid, task.outcome.id);
      setFollowupActions((detailed as any).followup_actions||[]);
    }catch(e:any){setErr(e.message)}
  };

  const createExternalQuery=async()=>{
    if(!extQueryText.trim()) return;
    setExtCreating(true);
    try{
      const q = await api.createResearchQuery(tid, id, {provider: extProvider, query: extQueryText});
      const qres = await api.getResearchQueriesForTask(tid, id, {limit:50});
      setExtQueries(qres.items);
      setExtQueryText(q.query);
    }catch(e:any){setErr(e.message)} finally{setExtCreating(false)}
  };
  const runExternalQuery=async(queryId:number)=>{
    setExtRunning(prev=>({...prev, [queryId]: true}));
    try{
      await api.runResearchQuery(tid, queryId);
      const qres = await api.getResearchQueriesForTask(tid, id, {limit:50});
      setExtQueries(qres.items);
      setExtResultsLoading(prev=>({...prev, [queryId]: true}));
      try{
        const rres = await api.getResearchQueryResults(tid, queryId, {limit:50});
        setExtResults(prev=>({...prev, [queryId]: rres.items}));
      }catch{}
      setExtResultsLoading(prev=>({...prev, [queryId]: false}));
    }catch(e:any){setErr(e.message)} finally{setExtRunning(prev=>({...prev, [queryId]: false}))}
  };
  const viewResults=async(queryId:number)=>{
    setExtResultsLoading(prev=>({...prev, [queryId]: true}));
    try{
      const rres = await api.getResearchQueryResults(tid, queryId, {limit:50});
      setExtResults(prev=>({...prev, [queryId]: rres.items}));
    }catch(e:any){setErr(e.message)} finally{setExtResultsLoading(prev=>({...prev, [queryId]: false}))}
  };

  if(loading) return <Loading msg="Loading task…" />;
  if(err) return <ErrorState msg={err} />;
  if(!task) return null;
  return <div className="space-y-6">
    <Link to={`/trees/${tid}/research/tasks`} className="text-sm text-blue-600 underline">← Research Tasks</Link>
    <div className="flex justify-between items-start gap-4">
      <h1 className="text-2xl font-bold">Research Task: {task.title}</h1>
      <TaskStatusBadge status={task.status} />
    </div>
    {/* Workflow actions */}
    <div className="border rounded p-3 bg-gray-50 flex gap-2 flex-wrap">
      {task.status==="OPEN" && <button onClick={()=>quickStatus("IN_PROGRESS")} className="px-3 py-1 bg-emerald-600 text-white rounded">Start Research</button>}
      {task.status==="IN_PROGRESS" && <>
        <button onClick={()=>quickStatus("RESOLVED")} className="px-3 py-1 bg-blue-600 text-white rounded">Mark Resolved</button>
        <button onClick={()=>quickStatus("REJECTED")} className="px-3 py-1 bg-red-600 text-white rounded">Mark Rejected</button>
        <button onClick={()=>quickStatus("INCONCLUSIVE")} className="px-3 py-1 bg-yellow-600 text-white rounded">Mark Inconclusive</button>
      </>}
      {(task.status==="RESOLVED"||task.status==="REJECTED"||task.status==="INCONCLUSIVE") && <span className="text-sm text-gray-600">Completed — you can edit outcome below</span>}
    </div>
    {/* Research Session */}
    <div className="border rounded p-4 bg-white">
      <h3 className="font-semibold">Research Session</h3>
      {task.session ? (
        <div className="mt-2 space-y-2">
          <div className="text-sm">{task.session.title} <span className={`ml-2 px-2 py-0.5 rounded text-xs ${task.session.status==="ACTIVE"?"bg-emerald-600 text-white": task.session.status==="PLANNED"?"bg-blue-600 text-white":"bg-gray-600 text-white"}`}>{task.session.status}</span></div>
          <div className="flex gap-2">
            <Link to={`/trees/${tid}/research/sessions/${task.session.id}`} className="px-3 py-1 bg-purple-600 text-white rounded text-sm">View Session</Link>
            <button onClick={detachFromSession} disabled={sessionSaving} className="px-3 py-1 border rounded text-sm disabled:opacity-50">Remove from Session</button>
          </div>
        </div>
      ) : (
        <div className="mt-2 space-y-2">
          <div className="text-sm text-gray-600">Not assigned</div>
          <div className="flex gap-2 flex-wrap items-end">
            <select value={selectedSession} onChange={e=>setSelectedSession(e.target.value)} className="border rounded px-2 py-1 text-sm">
              <option value="">Select session</option>
              {sessions.map((s:any)=><option key={s.id} value={s.id}>{s.title} ({s.status})</option>)}
            </select>
            <button onClick={attachToSession} disabled={!selectedSession || sessionSaving} className="px-3 py-1 bg-blue-600 text-white rounded text-sm disabled:opacity-50">Add to Session</button>
            <button onClick={createNewSessionFromTask} disabled={sessionSaving} className="px-3 py-1 border rounded text-sm">Create new session</button>
          </div>
        </div>
      )}
    </div>
    {/* External Research */}
    <div className="border rounded p-4 bg-white">
      <h3 className="font-semibold">External Research</h3>
      <div className="text-xs text-gray-600 mt-1">External Research finds candidates. The researcher decides what constitutes evidence. Results are <strong>not</strong> evidence.</div>
      <div className="text-xs text-amber-800 mt-1">FamilySearch Result ≠ Evidence — External results are candidates only. NeoGenealogy never writes to the FamilySearch Family Tree automatically.</div>
      <div className="mt-3 space-y-2">
        <label className="block"><span className="text-sm font-semibold">Provider</span>
          {extProvidersLoading ? <div className="text-xs text-gray-500 mt-1">Loading providers…</div> :
          <select value={extProvider} onChange={e=>setExtProvider(e.target.value)} className="border rounded px-2 py-1 mt-1">
            {extProviders.map((p:any)=>(
              <option key={p.name} value={p.name}>{p.display_name} {p.configured ? "" : "(not configured)"}</option>
            ))}
          </select>
          }
          {(() => {
            const sel = extProviders.find((p:any)=>p.name===extProvider);
            if(!sel) return null;
            if(sel.name==="familysearch" && !sel.configured){
              return <div className="text-xs text-amber-700 mt-1 bg-amber-50 border border-amber-200 rounded px-2 py-1">FamilySearch is not configured. Set <code>NEOGENEALOGY_FAMILYSEARCH_CLIENT_ID</code> or <code>NEOGENEALOGY_FAMILYSEARCH_ACCESS_TOKEN</code>. See <code>docs/FAMILYSEARCH.md</code>.</div>
            }
            if(sel.name==="familysearch" && sel.configured){
              return <div className="text-xs text-emerald-700 mt-1">FamilySearch adapter ready — searches FamilySearch Family Tree (via <code>/platform/tree/search</code>).</div>
            }
            return null;
          })()}
        </label>
        <label className="block"><span className="text-sm font-semibold">Query</span>
          <input value={extQueryText} onChange={e=>setExtQueryText(e.target.value)} placeholder="Josep García baptism 1882 Sant Martí" className="w-full border rounded px-2 py-1 mt-1" />
        </label>
        <button onClick={createExternalQuery} disabled={extCreating || !extQueryText.trim()} className="px-3 py-1 bg-indigo-600 text-white rounded text-sm disabled:opacity-50">{extCreating?"Creating…":"Create Query"}</button>
        <button onClick={()=>setExtQueryText(`${task.title} baptism 1882 Sant Martí`)} className="ml-2 px-3 py-1 border rounded text-sm">Suggest Query</button>
      </div>
      <div className="mt-4 space-y-3">
        {extQueries.length===0 ? <div className="text-sm text-gray-500">No external queries yet. Create a query to search.</div> :
          extQueries.map((q:any)=>(
            <div key={q.id} className="border rounded p-3 bg-gray-50">
              <div className="flex justify-between items-start gap-2">
                <div>
                  <div className="text-sm font-medium">{q.query}</div>
                  <div className="text-xs text-gray-600">{q.provider.toUpperCase()} · {q.status} {q.latest_execution ? `· ${q.latest_execution.result_count??0} results` : ""}</div>
                  {q.error_code && <div className="text-xs text-red-600">{q.error_code}: {q.error_message} {q.error_code==="AUTH_REQUIRED" && q.provider==="familysearch" ? <span className="ml-1">— <span className="bg-amber-100 border border-amber-200 rounded px-1">FamilySearch connection required</span> Check configuration in <code>docs/FAMILYSEARCH.md</code></span> : null}</div>}
                  {q.latest_execution?.error_code==="AUTH_REQUIRED" && <div className="text-xs text-amber-700">FamilySearch connection required — verify <code>NEOGENEALOGY_FAMILYSEARCH_CLIENT_ID</code>.</div>}
                </div>
                <div className="flex gap-2 flex-wrap">
                  <button onClick={()=>runExternalQuery(q.id)} disabled={!!extRunning[q.id]} className="px-2 py-1 bg-emerald-600 text-white rounded text-xs disabled:opacity-50">{extRunning[q.id]?"Running…": q.status==="COMPLETED"||q.status==="FAILED" ? "Run Again" : "Run Research"}</button>
                  <button onClick={()=>viewResults(q.id)} className="px-2 py-1 border rounded text-xs bg-white">View Results</button>
                  <Link to={`/trees/${tid}/research/queries/${q.id}`} className="px-2 py-1 border rounded text-xs bg-white">Detail</Link>
                </div>
              </div>
              {extResults[q.id] && (
                <div className="mt-3 space-y-2">
                  <div className="text-xs font-semibold">Results ({extResults[q.id].length})</div>
                  {extResults[q.id].length===0 ? <div className="text-xs text-gray-500">No results — search completed successfully but nothing found.</div> :
                    extResults[q.id].map((r:any)=>(
                      <div key={r.id} className="border rounded p-3 bg-white">
                        <div className="text-sm font-semibold">{r.title} {r.external_id && <span className="text-xs font-normal text-gray-500">· {r.external_id}</span>}</div>
                        {r.description && <div className="text-xs text-gray-600 mt-1">{r.description}</div>}
                        <div className="text-xs text-gray-500 mt-1">{r.date || ""} {r.place ? `· ${r.place}` : ""} {r.record_type ? `· ${r.record_type}` : ""} · {r.provider.toUpperCase()} {r.external_id ? `· ${r.external_id}` : ""}</div>
                        <div className="text-xs mt-1"><span className="px-2 py-0.5 bg-yellow-100 border border-yellow-200 rounded">External Research Result</span> <span className="ml-2 text-amber-800">This result is not evidence.</span></div>
                        <div className="text-xs mt-1">Possible matching record {r.provider==="familysearch" ? "— FamilySearch Family Tree" : ""}</div>
                        {r.url && <a href={r.url} target="_blank" rel="noopener noreferrer" className="text-xs text-blue-600 underline mt-1 inline-block">Open external source</a>}
                        <Link to={`/trees/${tid}/research/results/${r.id}`} className="ml-2 text-xs text-blue-600 underline">Review Result</Link>
                      </div>
                    ))}
                </div>
              )}
              {extResultsLoading[q.id] && <div className="text-xs text-gray-600 mt-2">Loading results…</div>}
            </div>
          ))}
      </div>
    </div>

    <div className="grid gap-4 border rounded p-4 bg-white">
      <div className="text-xs font-semibold text-gray-500">Research Task — What you decided to investigate</div>
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
      <div className="text-xs font-semibold text-gray-500 mb-1">Research Outcome — What you discovered</div>
      <h3 className="font-semibold mb-2">Research Outcome</h3>
      {task.outcome ? (
        <div className="space-y-3">
          <div className="flex gap-2 items-center"><span className="px-2 py-1 bg-emerald-100 rounded text-sm font-semibold">{formatOutcomeType(task.outcome.type)}</span><span className="text-xs text-gray-600">{new Date(task.outcome.created_at).toLocaleString()}</span></div>
          <div><div className="text-sm font-semibold">Summary</div><div className="text-sm">{task.outcome.summary}</div></div>
          {task.outcome.details && <div><div className="text-sm font-semibold">Details</div><div className="text-sm whitespace-pre-wrap">{task.outcome.details}</div></div>}

          {/* Evidence Assessment */}
          {assessment && (
            <div className="border rounded p-3 bg-gray-50 space-y-2">
              <div className="text-sm font-semibold">Evidence Assessment</div>
              <div className="flex flex-col gap-1">
                <span className="inline-block px-2 py-1 bg-white border rounded text-sm font-semibold w-fit">{formatAssessmentStatus(assessment.status)}</span>
                <span className="text-sm">{assessment.score} / 100</span>
                <div className="text-xs text-gray-700 space-y-0.5">
                  <div>{assessment.supporting_count} supporting</div>
                  <div>{assessment.contradicting_count} contradicting</div>
                  <div>{assessment.sources_count} sources</div>
                  <div>{assessment.cited_count} citations</div>
                </div>
                {task.outcome.type==="CONFIRMED" && assessment.status==="NO_EVIDENCE" && (
                  <div className="mt-2 text-sm text-amber-800 bg-amber-100 border border-amber-200 rounded px-3 py-2">This outcome is marked as CONFIRMED but has no recorded supporting evidence.</div>
                )}
                {task.outcome.type==="CONFIRMED" && assessment.status==="MIXED" && (
                  <div className="mt-2 text-sm text-orange-800 bg-orange-100 border border-orange-200 rounded px-3 py-2">This outcome has contradictory evidence.</div>
                )}
              </div>
              {assessment.reasons && assessment.reasons.length>0 && (
                <div className="mt-2 border-t pt-2">
                  <div className="text-sm font-semibold">Why this assessment?</div>
                  <ul className="text-xs text-gray-700 space-y-1 mt-1">
                    {assessment.reasons.map((r:any, idx:number)=>(
                      <li key={idx}>{r.points>0?`+${r.points}`:r.points} {r.message}</li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          )}

          {/* Evidence Gaps */}
          <div className="border rounded p-3 bg-gray-50 space-y-2">
            <div className="text-sm font-semibold">Evidence Gaps</div>
            {gaps.length===0 ? (
              <div className="text-sm text-gray-600">No evidence gaps detected.</div>
            ) : (
              <div className="space-y-2">
                {gaps.map((g:any)=>(
                  <div key={g.code} className={`rounded px-3 py-2 text-sm ${g.severity==="CRITICAL"?"bg-red-50 border border-red-200 text-red-800": g.severity==="WARNING"?"bg-amber-50 border border-amber-200 text-amber-800":"bg-blue-50 border border-blue-200 text-blue-800"}`}>
                    <div className="font-semibold">{gapIcon(g.severity)} {g.title}</div>
                    <div className="text-xs mt-0.5">{g.description}</div>
                  </div>
                ))}
                {/* Quick actions */}
                <div className="flex gap-2 flex-wrap mt-2">
                  {(gaps.some((g:any)=>g.code==="NO_SUPPORTING_EVIDENCE"||g.code==="CONFIRMED_WITHOUT_SUPPORT")) && (
                    <button onClick={()=>setShowAddEvidence(true)} className="text-xs px-2 py-1 bg-emerald-600 text-white rounded">Add Evidence</button>
                  )}
                  {(gaps.some((g:any)=>g.code==="NO_CITATION")) && (
                    <button onClick={()=>setShowAddEvidence(true)} className="text-xs px-2 py-1 bg-blue-600 text-white rounded">Review Evidence</button>
                  )}
                  {(gaps.some((g:any)=>g.code==="CONTRADICTORY_EVIDENCE")) && (
                    <button onClick={()=>{const el=document.getElementById("evidence-section"); el?.scrollIntoView({behavior:"smooth"});}} className="text-xs px-2 py-1 border rounded bg-white">Review Contradictions</button>
                  )}
                </div>
              </div>
            )}
            {gaps.some((g:any)=>g.severity==="CRITICAL") && (
              <div className="text-xs text-red-700 mt-1">⚠ Critical evidence gap</div>
            )}
          </div>

          {/* Research Follow-ups */}
          {followups.length>0 && (
            <div className="border rounded p-3 bg-gray-50 space-y-2">
              <div className="text-sm font-semibold">Research Follow-ups</div>
              <div className="space-y-2">
                {followups.map((f:any)=>(
                  <div key={f.code} className={`rounded px-3 py-2 text-sm ${f.priority==="HIGH"?"bg-red-50 border border-red-200 text-red-800": f.priority==="MEDIUM"?"bg-amber-50 border border-amber-200 text-amber-800":"bg-blue-50 border border-blue-200 text-blue-800"}`}>
                    <div className="text-xs font-semibold opacity-70">{f.priority}</div>
                    <div className="font-semibold">{f.title}</div>
                    <div className="text-xs mt-0.5">{f.description}</div>
                    {f.gap_code && <div className="text-xs mt-1 opacity-70">Gap: {f.gap_code}</div>}
                    <div className="mt-2 flex gap-2">
                      <button onClick={()=>startFollowup(f.code)} className="text-xs px-2 py-1 bg-indigo-600 text-white rounded">Start follow-up</button>
                      {(f.code==="ADD_SUPPORTING_EVIDENCE" || f.code==="ADD_SECOND_SUPPORTING_EVIDENCE") && (
                        <button onClick={()=>setShowAddEvidence(true)} className="text-xs px-2 py-1 bg-emerald-600 text-white rounded">Add Evidence</button>
                      )}
                      {(f.code==="ADD_CITATION" || f.code==="REVIEW_CONTRADICTION" || f.code==="REVIEW_SOURCE_COVERAGE") && (
                        <button onClick={()=>{const el=document.getElementById("evidence-section"); el?.scrollIntoView({behavior:"smooth"});}} className="text-xs px-2 py-1 bg-blue-600 text-white rounded">Review Evidence</button>
                      )}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Follow-up Actions */}
          {followupActions.length>0 && (
            <div className="border rounded p-3 bg-white space-y-2">
              <div className="text-sm font-semibold">Research Follow-up Actions</div>
              <div className="text-xs text-gray-600">{followupActions.length} action{followupActions.length!==1?"s":""} recorded — ordered by updated_at DESC</div>
              <div className="space-y-2">
                {followupActions.slice().sort((a:any,b:any)=> new Date(b.updated_at).getTime() - new Date(a.updated_at).getTime()).map((act:any)=>(
                  <div key={act.id} className="border rounded p-3 bg-gray-50">
                    <div className="flex justify-between items-start">
                      <div>
                        <div className="text-sm font-semibold">{act.followup_code.replace(/_/g," ")}</div>
                        <div className="text-xs">Status: <span className={`px-1 rounded font-semibold ${act.status==="COMPLETED"?"bg-emerald-100 text-emerald-800":act.status==="SKIPPED"?"bg-gray-200": "bg-yellow-100"}`}>{act.status}</span></div>
                        <div className="text-xs text-gray-600">Created: {new Date(act.created_at).toLocaleString()}</div>
                        {act.completed_at && <div className="text-xs text-gray-600">Completed: {new Date(act.completed_at).toLocaleString()}</div>}
                      </div>
                      <button onClick={()=>deleteFollowupAction(act.id)} className="text-xs text-red-600 underline">Delete</button>
                    </div>
                    {act.notes && <div className="text-sm mt-2 whitespace-pre-wrap border-t pt-2">"{act.notes}"</div>}
                    <div className="mt-2">
                      <textarea placeholder="Notes" value={followupNotes[act.id] ?? act.notes ?? ""} onChange={e=>setFollowupNotes({...followupNotes, [act.id]: e.target.value})} className="w-full border rounded px-2 py-1 text-sm" rows={2} />
                    </div>
                    <div className="flex gap-2 mt-2 flex-wrap">
                      {act.status==="OPEN" && <>
                        <button onClick={()=>updateFollowupStatus(act.id,"COMPLETED")} className="text-xs px-2 py-1 bg-emerald-600 text-white rounded">Mark completed</button>
                        <button onClick={()=>updateFollowupStatus(act.id,"SKIPPED")} className="text-xs px-2 py-1 bg-gray-600 text-white rounded">Skip</button>
                      </>}
                      {(act.status==="COMPLETED" || act.status==="SKIPPED") && (
                        <button onClick={()=>updateFollowupStatus(act.id,"OPEN")} className="text-xs px-2 py-1 border rounded bg-white">Reopen</button>
                      )}
                    </div>
                    {act.status==="COMPLETED" && <div className="text-xs text-emerald-700 mt-1">✓ Completed — Action completed, not gap resolved</div>}
                  </div>
                ))}
              </div>
            </div>
          )}

          <div id="evidence-section" className="border rounded p-3 bg-gray-50">
            <div className="flex justify-between items-center mb-2"><h4 className="font-semibold">Evidence</h4><span className="text-xs text-gray-600">{evidence.length} attached</span></div>
            {evidence.length===0 ? <div className="text-sm text-gray-500">No evidence attached yet.</div> :
              <div className="space-y-2">{evidence.map((ev:any)=><div key={ev.id} className="border rounded p-3 bg-white">
                <div className="flex justify-between items-start">
                  <span className={`px-2 py-1 rounded text-xs font-semibold ${ev.relationship==="SUPPORTS"?"bg-emerald-100 text-emerald-800":"bg-orange-100 text-orange-800"}`}>{ev.relationship==="SUPPORTS"?"✓ SUPPORTS":"⚠ CONTRADICTS"}</span>
                  <button onClick={()=>removeEvidence(ev.id)} className="text-xs text-red-600 underline">Remove</button>
                </div>
                <div className="text-sm font-medium mt-1">{ev.source?.title||`Source ${ev.source?.id}`}</div>
                {ev.citation && <div className="text-xs text-gray-600">{ev.citation.locator || "Citation"}</div>}
                <div className="text-sm mt-1 italic">"{ev.statement}"</div>
                {ev.notes && <div className="text-xs text-gray-600">Notes: {ev.notes}</div>}
              </div>)}</div>}
            {!showAddEvidence ? <button onClick={()=>setShowAddEvidence(true)} className="mt-3 px-3 py-1 bg-emerald-600 text-white rounded text-sm">+ Add Evidence</button> :
              <div className="mt-3 space-y-2 border-t pt-3">
                <div className="text-sm font-semibold">Add Evidence</div>
                <select value={evSourceId} onChange={e=>setEvSourceId(e.target.value)} className="w-full border rounded px-2 py-1">
                  <option value="">Select Source</option>
                  {sources.map((s:any)=><option key={s.id} value={s.id}>{s.title} ({s.type})</option>)}
                </select>
                <Link to={`/trees/${tid}/sources`} className="text-xs text-blue-600 underline">Create Source</Link>
                <select value={evCitationId} onChange={e=>setEvCitationId(e.target.value)} className="w-full border rounded px-2 py-1">
                  <option value="">No citation (optional)</option>
                  {citations.map((c:any)=><option key={c.id} value={c.id}>{c.locator || `Citation ${c.id}`}</option>)}
                </select>
                <select value={evRelationship} onChange={e=>setEvRelationship(e.target.value)} className="border rounded px-2 py-1">
                  <option value="SUPPORTS">SUPPORTS</option><option value="CONTRADICTS">CONTRADICTS</option>
                </select>
                <textarea value={evStatement} onChange={e=>setEvStatement(e.target.value)} placeholder="Statement (required)" className="w-full border rounded px-2 py-1" rows={2} />
                <textarea value={evNotes} onChange={e=>setEvNotes(e.target.value)} placeholder="Notes (optional)" className="w-full border rounded px-2 py-1" rows={2} />
                <div className="flex gap-2">
                  <button onClick={addEvidence} disabled={evSaving || !evSourceId || !evStatement.trim()} className="px-3 py-1 bg-blue-600 text-white rounded disabled:opacity-50">{evSaving?"Saving…":"Save Evidence"}</button>
                  <button onClick={()=>setShowAddEvidence(false)} className="px-3 py-1 border rounded">Cancel</button>
                </div>
              </div>}
          </div>
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

    <div className="border rounded p-4 bg-white">
      <div className="text-xs font-semibold text-gray-500">Research Task — Status / Person / Dates</div>
      <div className="text-xs text-gray-600 space-y-1 mt-2">
        <div>Status: {task.status}</div>
        <div>Person: {task.person_id? <Link to={`/trees/${tid}/persons/${task.person_id}`} className="text-blue-600 underline">{task.person_id}</Link>:"—"}</div>
        <div>Description: {task.description || "—"}</div>
        <div>Created: {new Date(task.created_at).toLocaleString()}</div>
        <div>Updated: {new Date(task.updated_at).toLocaleString()}</div>
        <div>Started: {task.started_at? new Date(task.started_at).toLocaleString():"—"}</div>
        <div>Completed: {task.completed_at? new Date(task.completed_at).toLocaleString():"—"}</div>
      </div>
    </div>
    {/* Research Case Summary */}
    <div className="border rounded p-4 bg-white space-y-3">
      <h3 className="font-semibold">Research Case Summary</h3>
      {caseSummaryLoading ? <div className="text-sm text-gray-600">Loading case summary…</div> :
       caseSummaryError ? <div className="text-sm text-red-700"><span>{caseSummaryError}</span> <button onClick={async()=>{setCaseSummaryError(null); setCaseSummaryLoading(true); try{const cs=await (api as any).getCaseSummary(tid,id); setCaseSummary(cs);}catch(e:any){setCaseSummaryError(e.message)} finally{setCaseSummaryLoading(false)}} } className="ml-2 px-2 py-1 border rounded text-xs bg-white">Retry</button></div> :
       !caseSummary ? <div className="text-sm text-gray-500">No case summary available.</div> :
       <div className="space-y-3">
         <div className="grid grid-cols-2 gap-2 text-sm">
           <div><span className="font-semibold">Status</span><div>{caseSummary.task?.status}</div></div>
           <div><span className="font-semibold">Resolution</span><div>{caseSummary.task?.resolution || "—"}</div></div>
           <div><span className="font-semibold">Outcome</span><div>{caseSummary.outcome ? formatOutcomeType(caseSummary.outcome.type) : "—"}</div></div>
           <div><span className="font-semibold">Evidence</span><div>{caseSummary.evidence_assessment ? caseSummary.evidence_assessment.evidence_total : 0}</div></div>
           <div><span className="font-semibold">Assessment</span><div>{caseSummary.evidence_assessment ? `${formatAssessmentStatus(caseSummary.evidence_assessment.status)} · ${caseSummary.evidence_assessment.score}` : "—"}</div></div>
           <div><span className="font-semibold">Evidence gaps</span><div>{caseSummary.evidence_gaps?.length ?? 0}</div></div>
           <div><span className="font-semibold">Follow-ups</span><div>{caseSummary.research_followups?.length ?? 0}</div></div>
           <div><span className="font-semibold">Follow-up actions</span><div>{caseSummary.followup_actions ? `${caseSummary.followup_actions.filter((a:any)=>a.status==="COMPLETED").length} completed / ${caseSummary.followup_actions.length} total` : "0"}</div></div>
         </div>
         {/* Warnings */}
         {caseSummary.closure_warnings && caseSummary.closure_warnings.length>0 && (
           <div className="space-y-2">
             <div className="text-sm font-semibold">Closure Warnings</div>
             {caseSummary.closure_warnings.map((w:any)=>(
               <div key={w.code} className={`rounded px-3 py-2 text-sm ${w.severity==="CRITICAL"?"bg-red-50 border border-red-200 text-red-800": w.severity==="WARNING"?"bg-amber-50 border border-amber-200 text-amber-800":"bg-blue-50 border border-blue-200 text-blue-800"}`}>
                 <div className="font-semibold">{w.severity==="CRITICAL"?"Critical":w.severity==="WARNING"?"Warning":"Info"}: {w.title}</div>
                 <div className="text-xs mt-0.5">{w.description}</div>
                 <div className="text-xs mt-1 opacity-70">Code: {w.code}</div>
               </div>
             ))}
           </div>
         )}
         {/* Timeline */}
         {caseSummary.timeline && caseSummary.timeline.length>0 && (
           <div className="border rounded p-3 bg-gray-50">
             <div className="text-sm font-semibold mb-2">Timeline</div>
             <div className="space-y-1">
               {caseSummary.timeline.map((ev:any,idx:number)=>(
                 <div key={idx} className="text-xs flex gap-2">
                   <span className="font-mono text-gray-600">{new Date(ev.timestamp).toLocaleString()}</span>
                   <span className="font-semibold">{ev.event_type}</span>
                   <span>{ev.label}</span>
                 </div>
               ))}
             </div>
           </div>
         )}
         {/* Case Closure when terminal */}
         {(["RESOLVED","REJECTED","INCONCLUSIVE"].includes(caseSummary.task?.status)) && (
           <div className="border rounded p-3 bg-gray-50 space-y-2">
             <div className="text-sm font-semibold">Case Closure</div>
             <div className="text-xs space-y-1">
               <div>Status: {caseSummary.task?.status}</div>
               <div>Resolution: {caseSummary.task?.resolution || "—"}</div>
               <div>Completed at: {caseSummary.task?.completed_at ? new Date(caseSummary.task.completed_at).toLocaleString() : "—"}</div>
               <div>Outcome: {caseSummary.outcome ? `${formatOutcomeType(caseSummary.outcome.type)} — ${caseSummary.outcome.summary}` : "—"}</div>
               <div>Assessment: {caseSummary.evidence_assessment ? `${formatAssessmentStatus(caseSummary.evidence_assessment.status)} · ${caseSummary.evidence_assessment.score}` : "—"}</div>
               <div>Warnings: {caseSummary.closure_warnings?.length ? caseSummary.closure_warnings.map((w:any)=>w.code).join(", ") : "none"}</div>
             </div>
           </div>
         )}
       </div>
      }
    </div>

    {opp && <div className="border rounded p-4 bg-gray-50">
      <div className="text-xs font-semibold text-gray-500">Original Opportunity — What the system found</div>
      <h3 className="font-semibold mb-2 mt-1">Original Research Opportunity</h3>
      <div className="flex gap-2 items-center"><PriorityBadge p={opp.priority} /><ScoreBadge score={opp.score} /></div>
      <div className="text-sm mt-2"><strong>Why:</strong> {opp.why}</div>
      <div className="text-sm"><strong>What:</strong> {JSON.stringify(opp.what)}</div>
      <div className="text-sm"><strong>Sources:</strong> {JSON.stringify(opp.potential_sources)}</div>
      {opp.breakdown && <div className="mt-2"><ScoreBreakdown breakdown={opp.breakdown} /></div>}
      <Link to={`/trees/${tid}/research`} className="text-sm text-blue-600 underline">Back to Queue</Link>
    </div>}
  </div>
}
