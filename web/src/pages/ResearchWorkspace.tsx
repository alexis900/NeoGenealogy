import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { api } from "../api/client";
import { Loading, ErrorState, Empty } from "../components/common";
import { TaskStatusBadge } from "../components/Badges";

function formatOutcomeType(t:string){
  const map:any={"CONFIRMED":"Confirmed","FALSE_LEAD":"False lead","INCONCLUSIVE":"Inconclusive","NEW_LEAD":"New lead","NO_EVIDENCE":"No evidence"};
  return map[t]||t;
}

export default function ResearchWorkspace(){
  const {treeId}=useParams(); const id=Number(treeId);
  const [summary,setSummary]=useState<any>(null);
  const [activeTasks,setActiveTasks]=useState<any[]>([]);
  const [recentOutcomes,setRecentOutcomes]=useState<any[]>([]);
  const [oppsCounts,setOppsCounts]=useState<{high:number;medium:number;low:number}|null>(null);
  const [loading,setLoading]=useState(true);
  const [err,setErr]=useState<string|null>(null);

  const load=async()=>{
    setLoading(true); setErr(null);
    try{
      const s=await api.getResearchSummary(id);
      setSummary(s);
      setOppsCounts(s.opportunities);
      const tasksRes=await api.getTasks(id,{limit:5});
      const active=tasksRes.items.filter((t:any)=> t.status==="IN_PROGRESS"||t.status==="OPEN").slice(0,5);
      setActiveTasks(active);
      const outcomes=await api.getOutcomes(id,{limit:5});
      setRecentOutcomes(outcomes.items);
    }catch(e:any){setErr(e.message)} finally{setLoading(false)}
  };
  useEffect(()=>{load()},[id]);

  if(loading) return <Loading msg="Loading research workspace…" />;
  if(err) return <ErrorState msg={err} onRetry={load} />;

  return <div className="space-y-6">
    <h1 className="text-2xl font-bold">Research</h1>
    <p className="text-sm text-gray-600">Your research workspace — what deserves investigation, what you are working on, and what you have discovered.</p>
    <nav className="flex gap-2 text-sm">
      <Link to={`/trees/${id}/research`} className="px-3 py-1 bg-gray-800 text-white rounded">Overview</Link>
      <Link to={`/trees/${id}/research/planning`} className="px-3 py-1 border rounded hover:bg-gray-50">Planning</Link>
      <Link to={`/trees/${id}/research/opportunities`} className="px-3 py-1 border rounded hover:bg-gray-50">Opportunities</Link>
      <Link to={`/trees/${id}/research/tasks`} className="px-3 py-1 border rounded hover:bg-gray-50">Tasks</Link>
      <Link to={`/trees/${id}/research/history`} className="px-3 py-1 border rounded hover:bg-gray-50">History</Link>
    </nav>

    {/* Opportunities block */}
    <div className="border rounded p-4 bg-white">
      <h2 className="font-semibold mb-2">Opportunities</h2>
      {oppsCounts ? <div className="flex gap-4 text-sm">
        <span>High priority: <strong>{oppsCounts.high}</strong></span>
        <span>Medium priority: <strong>{oppsCounts.medium}</strong></span>
        <span>Low priority: <strong>{oppsCounts.low}</strong></span>
      </div> : <div className="text-sm text-gray-500">No opportunities data</div>}
      <div className="text-sm text-gray-600 mt-1">Automatically detected research opportunities</div>
      <Link to={`/trees/${id}/research/opportunities`} className="text-sm text-blue-600 underline mt-2 inline-block">View Research Queue →</Link>
    </div>

    {/* Active Tasks block */}
    <div className="border rounded p-4 bg-white">
      <div className="flex justify-between items-center mb-2">
        <h2 className="font-semibold">Active Tasks</h2>
        <Link to={`/trees/${id}/research/tasks`} className="text-sm text-blue-600 underline">View all tasks</Link>
      </div>
      {summary && <div className="text-xs text-gray-600 mb-2">Open {summary.tasks.open} · In Progress {summary.tasks.in_progress}</div>}
      {activeTasks.length===0 ? <Empty msg="No research tasks yet. Start from a research opportunity to begin an investigation." />
        : <div className="space-y-2">
          {activeTasks.map((t:any)=><Link key={t.id} to={`/trees/${id}/research/tasks/${t.id}`} className="block border rounded p-3 hover:bg-gray-50">
            <div className="flex justify-between items-start gap-2">
              <span className="font-semibold text-sm">{t.title}</span>
              <TaskStatusBadge status={t.status} />
            </div>
            <div className="text-xs text-gray-600 mt-1">Person {t.person_id ?? "—"} · Updated {new Date(t.updated_at).toLocaleDateString()} · {t.has_outcome ? "Outcome recorded" : "Not recorded"}</div>
            {t.opportunity && <div className="text-xs text-gray-500 mt-1">From Opportunity Score: {t.opportunity.score} Priority: {t.opportunity.priority}</div>}
            <div className="text-xs mt-1">
              {t.status==="IN_PROGRESS" ? <span className="text-blue-600">Continue Research →</span> : t.status==="OPEN" ? <span className="text-emerald-600">Start Research →</span> : null}
            </div>
          </Link>)}
        </div>}
    </div>

     {/* Recent Outcomes block */}
    <div className="border rounded p-4 bg-white">
      <div className="flex justify-between items-center mb-2">
        <h2 className="font-semibold">Recent Outcomes</h2>
        <Link to={`/trees/${id}/research/history`} className="text-sm text-blue-600 underline">View research history</Link>
      </div>
      {recentOutcomes.length===0 ? <Empty msg="No research history yet. Completed investigations will appear here." />
        : <div className="space-y-2">
          {recentOutcomes.map((o:any)=><Link key={o.id} to={`/trees/${id}/research/tasks/${o.task_id}`} className="block border rounded p-3 hover:bg-gray-50">
            <div className="flex gap-2 items-center">
              <span className="px-2 py-1 bg-emerald-100 rounded text-xs font-semibold">{formatOutcomeType(o.type)}</span>
              <span className="text-xs text-gray-600">{new Date(o.created_at).toLocaleDateString()}</span>
              {o.evidence_assessment && <span className="text-xs text-gray-600">{o.evidence_assessment.status} · {o.evidence_assessment.score}</span>}
              {o.evidence_gaps && o.evidence_gaps.length>0 && <span className="text-xs text-amber-700">Gaps: {o.evidence_gaps.length}</span>}
            </div>
            <div className="text-sm font-medium mt-1">{o.summary}</div>
            <div className="text-xs text-gray-600">Task {o.task_id} · Person linkage via task</div>
            <div className="text-xs text-blue-600 mt-1">View Result →</div>
          </Link>)}
        </div>}
    </div>

    {summary && <div className="border rounded p-4 bg-gray-50">
      <div className="text-sm text-gray-600">Evidence recorded: <strong>{summary.evidence?.total ?? 0}</strong> · Sources: <strong>{summary.sources?.total ?? 0}</strong></div>
      <div className="flex gap-2 mt-2">
        <Link to={`/trees/${id}/sources`} className="text-sm text-blue-600 underline">View Sources</Link>
        <Link to={`/trees/${id}/evidence`} className="text-sm text-blue-600 underline">View Evidence</Link>
      </div>
      {summary.assessment && (
        <div className="mt-3 border-t pt-3">
          <div className="text-sm font-semibold">Evidence Assessment</div>
          <div className="text-xs text-gray-700 mt-1 space-y-0.5">
            <div>No Evidence: <strong>{summary.assessment.no_evidence ?? 0}</strong></div>
            <div>Weak: <strong>{summary.assessment.weak ?? 0}</strong></div>
            <div>Mixed: <strong>{summary.assessment.mixed ?? 0}</strong></div>
            <div>Supported: <strong>{summary.assessment.supported ?? 0}</strong></div>
            <div>Strongly Supported: <strong>{summary.assessment.strongly_supported ?? 0}</strong></div>
          </div>
        </div>
      )}
      {summary.evidence_gaps && (
        <div className="mt-3 border-t pt-3">
          <div className="text-sm font-semibold">Evidence Gaps</div>
          <div className="text-xs text-gray-700 mt-1 space-y-0.5">
            <div>Critical: <strong>{summary.evidence_gaps.critical ?? 0}</strong></div>
            <div>Warnings: <strong>{summary.evidence_gaps.warning ?? 0}</strong></div>
            <div>Info: <strong>{summary.evidence_gaps.info ?? 0}</strong></div>
          </div>
        </div>
      )}
      {summary.research_followups && (
        <div className="mt-3 border-t pt-3">
          <div className="text-sm font-semibold">Research Follow-ups</div>
          <div className="text-xs text-gray-700 mt-1 space-y-0.5">
            <div>High: <strong>{summary.research_followups.high ?? 0}</strong></div>
            <div>Medium: <strong>{summary.research_followups.medium ?? 0}</strong></div>
            <div>Low: <strong>{summary.research_followups.low ?? 0}</strong></div>
          </div>
        </div>
      )}
      {summary.followup_actions && (
        <div className="mt-3 border-t pt-3">
          <div className="text-sm font-semibold">Follow-up Actions</div>
          <div className="text-xs text-gray-700 mt-1 space-y-0.5">
            <div>Open: <strong>{summary.followup_actions.open ?? 0}</strong></div>
            <div>Completed: <strong>{summary.followup_actions.completed ?? 0}</strong></div>
            <div>Skipped: <strong>{summary.followup_actions.skipped ?? 0}</strong></div>
          </div>
        </div>
      )}
    </div>}
  </div>
}
