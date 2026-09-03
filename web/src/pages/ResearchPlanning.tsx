import { useEffect, useState, useCallback } from "react";
import { Link, useParams, useSearchParams } from "react-router-dom";
import { api, ApiError } from "../api/client";
import type { ResearchPlan, ResearchPlanItem } from "../api/types";
import { PriorityBadge, ScoreBadge, ResearchabilityBadge } from "../components/Badges";

function PlanningSkeleton() {
  return (
    <div className="space-y-4 animate-pulse" aria-busy="true" aria-label="Loading research planning">
      <p className="text-center text-gray-600">Loading research planning…</p>
      <div className="h-6 bg-gray-200 rounded w-48" />
      <div className="h-4 bg-gray-200 rounded w-96" />
      <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3">
        {Array.from({ length: 6 }).map((_, i) => (
          <div key={i} className="h-20 bg-gray-100 border rounded" />
        ))}
      </div>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
        {Array.from({ length: 4 }).map((_, i) => (
          <div key={i} className="h-48 bg-gray-100 border rounded" />
        ))}
      </div>
    </div>
  );
}

function ResearchPlanSummaryBlock({ summary }: { summary: ResearchPlan["summary"] }) {
  return (
    <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-6 gap-3 text-center" data-testid="plan-summary">
      <div className="bg-white border rounded p-3">
        <div className="text-xs text-gray-600 uppercase tracking-wide">Recommended</div>
        <div className="text-2xl font-bold">{summary.recommended_count}</div>
      </div>
      <div className="bg-white border rounded p-3">
        <div className="text-xs text-gray-600 uppercase tracking-wide">Candidates</div>
        <div className="text-2xl font-bold">{summary.total_candidates}</div>
      </div>
      <div className="bg-white border rounded p-3">
        <div className="text-xs text-gray-600 uppercase tracking-wide">Active research</div>
        <div className="text-2xl font-bold">{summary.active_count}</div>
      </div>
      <div className="bg-white border rounded p-3">
        <div className="text-xs text-gray-600 uppercase tracking-wide">Inconclusive</div>
        <div className="text-2xl font-bold">{summary.inconclusive_count}</div>
      </div>
      <div className="bg-white border rounded p-3">
        <div className="text-xs text-gray-600 uppercase tracking-wide">High priority</div>
        <div className="text-2xl font-bold">{summary.high_priority_count}</div>
      </div>
      <div className="bg-white border rounded p-3">
        <div className="text-xs text-gray-600 uppercase tracking-wide">Critical gaps</div>
        <div className="text-2xl font-bold">{summary.critical_gap_count}</div>
      </div>
    </div>
  );
}

function ResearchPlanCard({
  item,
  treeId,
  onStartResearch,
  startingId,
  startError,
  errorOppId,
}: {
  item: ResearchPlanItem;
  treeId: number;
  onStartResearch: (oppId: number, personId: number) => void;
  startingId: number | null;
  startError: string | null;
  errorOppId: number | null;
}) {
  const [expanded, setExpanded] = useState(false);
  const isActive = item.active_task;
  const isInconclusive = item.task_status === "INCONCLUSIVE";
  const showActiveLabel = isActive;
  const showInconclusiveLabel = !isActive && isInconclusive;
  const confidencePct = Math.round(item.confidence * 100);

  // Determine CTA: with task or inconclusive -> View Research Task, else Start Research
  const hasTask = isActive || isInconclusive;

  return (
    <div className="border rounded p-4 bg-white shadow-sm flex flex-col">
      {/* Priority - top */}
      <div className="flex justify-between items-start gap-2">
        <PriorityBadge p={item.priority.toLowerCase()} />
        {showActiveLabel && (
          <span className="text-xs px-2 py-0.5 bg-blue-100 text-blue-800 rounded font-medium">Already being researched</span>
        )}
        {showInconclusiveLabel && (
          <span className="text-xs px-2 py-0.5 bg-amber-100 text-amber-800 rounded font-medium">Previously investigated · Inconclusive</span>
        )}
      </div>

      {/* Title / Person */}
      <div className="mt-3">
        <div className="text-xs text-gray-500">Person {item.person_id} · Opportunity {item.opportunity_id}</div>
        <div className="font-semibold text-base mt-1">{item.title}</div>
      </div>

      {/* Score hierarchy: Research Score primary, Priority already, Planning secondary */}
      <div className="mt-3 space-y-2">
        <div className="flex items-center gap-2">
          <span className="text-xs text-gray-600 uppercase tracking-wide">Research Score</span>
          <ScoreBadge score={item.research_score} />
        </div>
        <div className="flex items-center gap-2 text-sm">
          <span className="text-xs text-gray-500">Planning Score</span>
          <span className="px-2 py-0.5 bg-gray-100 border rounded font-mono text-sm" aria-label={`Planning Score ${item.planning_score.toFixed(1)}`}>
            {item.planning_score.toFixed(1)}
          </span>
        </div>
      </div>

      {/* Researchability / Confidence */}
      <div className="flex gap-2 mt-3 items-center flex-wrap text-xs">
        <span className="flex items-center gap-1">
          <ResearchabilityBadge r={item.researchability.toLowerCase()} />
          <span className="sr-only">Researchability {item.researchability}</span>
        </span>
        <span className="text-gray-600">Confidence {confidencePct}%</span>
      </div>

      {/* Why is this here? */}
      {item.reasons && item.reasons.length > 0 && (
        <div className="mt-3">
          <button
            onClick={() => setExpanded(!expanded)}
            aria-expanded={expanded}
            aria-controls={`reasons-${item.opportunity_id}`}
            className="text-xs font-semibold text-blue-600 hover:underline focus:outline-none focus:ring-2 focus:ring-blue-500 rounded px-1"
          >
            {expanded ? "Hide reasons" : "Why is this here?"}
          </button>
          {expanded && (
            <ul id={`reasons-${item.opportunity_id}`} className="mt-2 text-xs text-gray-700 space-y-1 bg-gray-50 border rounded p-2">
              {item.reasons.map((r) => (
                <li key={r.code} className="flex gap-1">
                  <span aria-hidden="true">✓</span>
                  <span>
                    <span className="font-medium">{r.label}</span> – {r.description}
                  </span>
                </li>
              ))}
            </ul>
          )}
        </div>
      )}

      {/* Actions - max 2 */}
      <div className="flex gap-2 mt-4">
        <Link
          to={`/trees/${treeId}/research/opportunities/${item.opportunity_id}`}
          className="px-3 py-1 border rounded text-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-500"
        >
          View Opportunity
        </Link>
        {hasTask ? (
          <Link
            to={`/trees/${treeId}/research/tasks?opportunity_id=${item.opportunity_id}`}
            className="px-3 py-1 bg-emerald-600 text-white rounded text-sm hover:bg-emerald-700 focus:outline-none focus:ring-2 focus:ring-emerald-500"
          >
            View Research Task
          </Link>
        ) : (
          <button
            onClick={() => onStartResearch(item.opportunity_id, item.person_id)}
            disabled={startingId === item.opportunity_id}
            aria-label={`Start Research for opportunity ${item.opportunity_id}`}
            className="px-3 py-1 bg-blue-600 text-white rounded text-sm hover:bg-blue-700 disabled:opacity-50 focus:outline-none focus:ring-2 focus:ring-blue-500"
          >
            {startingId === item.opportunity_id ? "Starting research…" : "Start Research"}
          </button>
        )}
      </div>
      {errorOppId === item.opportunity_id && startError && (
        <p className="text-xs text-red-600 mt-2" role="alert">
          {startError}
        </p>
      )}
    </div>
  );
}

function DeferredList({ items, treeId }: { items: ResearchPlanItem[]; treeId: number }) {
  const [show, setShow] = useState(false);
  if (items.length === 0) return null;
  return (
    <div className="border rounded p-4 bg-gray-50 space-y-3">
      <h2 className="text-lg font-semibold">Deferred</h2>
      <p className="text-sm text-gray-600">{items.length} other candidates</p>
      <button
        onClick={() => setShow(!show)}
        aria-expanded={show}
        className="text-sm text-blue-600 hover:underline focus:outline-none focus:ring-2 focus:ring-blue-500 rounded px-1"
      >
        {show ? "Hide deferred candidates" : "Show deferred candidates"}
      </button>
      {show && (
        <div className="space-y-2 mt-2">
          {items.map((item) => (
            <div key={item.opportunity_id} className="border rounded p-3 bg-white text-sm flex flex-col gap-1">
              <div className="flex justify-between items-center gap-2">
                <PriorityBadge p={item.priority.toLowerCase()} />
                <span className="font-mono text-xs bg-gray-100 border rounded px-1.5 py-0.5">
                  Planning {item.planning_score.toFixed(1)}
                </span>
              </div>
              <div className="font-medium">
                Person {item.person_id} · {item.title}
              </div>
              <div className="text-xs text-gray-600">
                Research Score {item.research_score} · Planning Score {item.planning_score.toFixed(1)} · {item.researchability} · {Math.round(item.confidence * 100)}%
              </div>
              <Link to={`/trees/${treeId}/research/opportunities/${item.opportunity_id}`} className="text-blue-600 underline text-xs w-fit">
                View Opportunity
              </Link>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

export default function ResearchPlanning() {
  const { treeId } = useParams();
  const id = Number(treeId);
  const [searchParams, setSearchParams] = useSearchParams();

  // Initialize from URL (normalize to lower for priority/researchability)
  const initialPriority = (searchParams.get("priority") || "").toLowerCase();
  const initialResearchability = (searchParams.get("researchability") || "").toLowerCase();
  const initialMinScore = searchParams.get("min_score") || "";
  const initialLimit = searchParams.get("limit") || "10";

  const [data, setData] = useState<ResearchPlan | null>(null);
  const [loading, setLoading] = useState(true);
  const [err, setErr] = useState<string | null>(null);

  const [priority, setPriority] = useState(initialPriority);
  const [researchability, setResearchability] = useState(initialResearchability);
  const [minScore, setMinScore] = useState(initialMinScore);
  const [limit, setLimit] = useState(initialLimit);

  const [startingId, setStartingId] = useState<number | null>(null);
  const [startError, setStartError] = useState<string | null>(null);
  const [errorOppId, setErrorOppId] = useState<number | null>(null);
  const [successMsg, setSuccessMsg] = useState<string | null>(null);

  // Sync URL when filters change (skip initial mount duplication but keep simple)
  const syncUrl = useCallback(
    (p: string, r: string, ms: string, l: string) => {
      const sp = new URLSearchParams();
      if (p) sp.set("priority", p);
      if (r) sp.set("researchability", r);
      if (ms) sp.set("min_score", ms);
      if (l && l !== "10") sp.set("limit", l);
      else if (l) sp.set("limit", l);
      // Only set if different to avoid loop
      if (sp.toString() !== searchParams.toString()) {
        setSearchParams(sp, { replace: true });
      }
    },
    [searchParams, setSearchParams]
  );

  const load = useCallback(async () => {
    setLoading(true);
    setErr(null);
    try {
      const r = await api.getPlan(id, {
        limit: limit ? Number(limit) : undefined,
        min_score: minScore ? Number(minScore) : undefined,
        priority: priority || undefined,
        researchability: researchability || undefined,
      });
      setData(r as ResearchPlan);
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      setErr(msg);
    } finally {
      setLoading(false);
    }
  }, [id, priority, researchability, minScore, limit]);

  useEffect(() => {
    load();
  }, [load]);

  // Keep URL in sync with filter state (debounced via effect)
  useEffect(() => {
    syncUrl(priority, researchability, minScore, limit);
  }, [priority, researchability, minScore, limit, syncUrl]);

  const startResearch = async (oppId: number, personId: number) => {
    setStartingId(oppId);
    setStartError(null);
    setErrorOppId(null);
    setSuccessMsg(null);
    try {
      await api.createTaskFromOpportunity(id, oppId, {
        title: `Research opportunity ${oppId} - person ${personId}`,
      });
      setSuccessMsg("Research task created.");
      // refresh planning to show active_task
      await load();
    } catch (e: unknown) {
      const msg = e instanceof ApiError ? e.message : e instanceof Error ? e.message : String(e);
      // Handle duplicate task case gracefully - refresh and show message
      if (msg.toLowerCase().includes("already") || msg.toLowerCase().includes("duplicate") || msg.toLowerCase().includes("exists")) {
        setStartError("Research task already exists.");
        setErrorOppId(oppId);
        await load();
      } else {
        setStartError("Unable to start research.");
        setErrorOppId(oppId);
        setErr(msg);
      }
    } finally {
      setStartingId(null);
    }
  };

  if (loading) return <PlanningSkeleton />;
  if (err && !data) {
    return (
      <div className="p-8 text-center">
        <p className="text-red-600 mb-2">Unable to load research planning.</p>
        <p className="text-sm text-gray-600 mb-3">{err}</p>
        <button onClick={load} className="px-4 py-1 bg-gray-800 text-white rounded">
          Retry
        </button>
      </div>
    );
  }

  const summary = data?.summary;
  const recommended: ResearchPlanItem[] = data?.recommended || [];
  const deferred: ResearchPlanItem[] = data?.deferred || [];
  const totalCandidates = data?.total_candidates ?? summary?.total_candidates ?? 0;

  // Determine empty states
  const hasFilters = Boolean(priority || researchability || minScore);
  const isEmptyTree = totalCandidates === 0 && !hasFilters && recommended.length === 0 && deferred.length === 0;
  const isFilteredEmpty = recommended.length === 0 && deferred.length === 0 && totalCandidates === 0 && hasFilters;

  // Actually if api filters with min_score, it returns total_candidates filtered. So isFilteredEmpty when filtered but no results
  // Also case: recommended empty but deferred non-empty not empty.
  // Use totalCandidates==0 + hasFilters to decide filtered empty.

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-2xl font-bold">Research Planning</h1>
        <p className="text-lg font-medium text-gray-700 mt-1">What should I research next?</p>
        <p className="text-sm text-gray-600 mt-2 max-w-3xl">
          Planning combines existing Research Score, researchability, confidence, evidence gaps and current task state to suggest what is
          most useful to investigate next.
        </p>
        {summary && (
          <div className="flex gap-4 text-sm mt-3 flex-wrap">
            <span>
              <strong>{summary.recommended_count}</strong> recommended investigations
            </span>
            <span>
              <strong>{summary.total_candidates}</strong> total candidates
            </span>
          </div>
        )}
      </div>

      {summary && <ResearchPlanSummaryBlock summary={summary} />}

      {/* Success toast */}
      {successMsg && (
        <div className="bg-emerald-50 border border-emerald-200 text-emerald-800 px-4 py-2 rounded text-sm" role="status">
          {successMsg}
        </div>
      )}

      {/* Filters */}
      <div className="flex gap-3 flex-wrap items-end bg-gray-50 border rounded p-3">
        <div>
          <label htmlFor="filter-priority" className="text-xs text-gray-600 block">
            Priority
          </label>
          <select
            id="filter-priority"
            value={priority}
            onChange={(e) => setPriority(e.target.value)}
            className="border rounded px-2 py-1 text-sm"
          >
            <option value="">All priorities</option>
            <option value="critical">Critical</option>
            <option value="high">High</option>
            <option value="medium">Medium</option>
            <option value="low">Low</option>
          </select>
        </div>
        <div>
          <label htmlFor="filter-researchability" className="text-xs text-gray-600 block">
            Researchability
          </label>
          <select
            id="filter-researchability"
            value={researchability}
            onChange={(e) => setResearchability(e.target.value)}
            className="border rounded px-2 py-1 text-sm"
          >
            <option value="">All</option>
            <option value="high">High</option>
            <option value="medium">Medium</option>
            <option value="low">Low</option>
          </select>
        </div>
        <div>
          <label htmlFor="filter-min-score" className="text-xs text-gray-600 block">
            Min Planning Score
          </label>
          <div className="flex items-center gap-2">
            <input
              id="filter-min-score"
              type="range"
              min={0}
              max={100}
              value={minScore || "0"}
              onChange={(e) => setMinScore(e.target.value === "0" ? "" : e.target.value)}
              className="w-28"
              aria-label="Minimum Planning Score"
            />
            <input
              type="number"
              min={0}
              max={100}
              value={minScore}
              onChange={(e) => setMinScore(e.target.value)}
              placeholder="0"
              className="border rounded px-2 py-1 w-20 text-sm"
              aria-label="Minimum Planning Score value"
            />
          </div>
        </div>
        <div>
          <label htmlFor="filter-limit" className="text-xs text-gray-600 block">
            Limit
          </label>
          <select
            id="filter-limit"
            value={limit}
            onChange={(e) => setLimit(e.target.value)}
            className="border rounded px-2 py-1 text-sm"
          >
            <option value="10">10</option>
            <option value="20">20</option>
            <option value="50">50</option>
          </select>
        </div>
      </div>

      {/* Empty states */}
      {isEmptyTree && (
        <div className="p-8 text-center border rounded bg-white">
          <p className="font-medium">No research opportunities to plan.</p>
          <p className="text-sm text-gray-600 mt-1">Your current tree has no actionable research opportunities.</p>
          <p className="text-sm text-gray-500 mt-1">No recommended investigations. Try adjusting filters.</p>
        </div>
      )}
      {isFilteredEmpty && !isEmptyTree && (
        <div className="p-8 text-center border rounded bg-white">
          <p className="font-medium">No opportunities match these filters.</p>
          <p className="text-sm text-gray-600 mt-1">Try broadening your filters.</p>
        </div>
      )}

      {/* Recommended section */}
      {!isEmptyTree && !isFilteredEmpty && (
        <div className="space-y-4">
          <div>
            <h2 className="text-lg font-semibold">Recommended</h2>
            <p className="text-xs text-gray-600 mt-1">Research Score = importance/interest · Planning Score = practical priority</p>
          </div>

          {recommended.length === 0 ? (
            <div className="p-8 text-center text-gray-500 border rounded bg-white">No recommended investigations. Try adjusting filters.</div>
          ) : (
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              {recommended.map((item) => (
                <ResearchPlanCard
                  key={item.opportunity_id}
                  item={item}
                  treeId={id}
                  onStartResearch={startResearch}
                  startingId={startingId}
                  startError={startError}
                  errorOppId={errorOppId}
                />
              ))}
            </div>
          )}

          {/* Deferred */}
          {deferred.length > 0 && <DeferredList items={deferred} treeId={id} />}
          {deferred.length === 0 && recommended.length > 0 && (
            <div className="text-sm text-gray-500">{deferred.length} deferred · {summary?.deferred_count ?? 0} deferred candidates</div>
          )}
        </div>
      )}

      {/* If we have data but deferred empty, still show deferred header with count? Handled above */}
    </div>
  );
}
