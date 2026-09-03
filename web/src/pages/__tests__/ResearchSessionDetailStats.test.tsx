import { render, screen } from "@testing-library/react";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import { vi } from "vitest";
import ResearchSessionDetail from "../ResearchSessionDetail";

vi.mock("../../api/client", () => ({
  api: {
    getSession: vi.fn(() => Promise.resolve({
      session: { id: 1, tree_id: 1, title: "Find parents", description: "obj", status: "COMPLETED", person_id: 5, opportunity_id: 10, created_at: "2026-09-01T00:00:00Z", updated_at: "2026-09-03T00:00:00Z", started_at: "2026-09-01T00:00:00Z", completed_at: "2026-09-03T00:00:00Z" },
      person: null,
      opportunity: null,
      tasks: [],
      summary: { total_tasks: 3, open_tasks: 2, in_progress_tasks: 0, terminal_tasks: 1, outcomes_count: 2 },
      stats: { total_tasks: 3, completed_tasks: 1, open_tasks: 2, in_progress_tasks: 0, inconclusive_tasks: 0, rejected_tasks: 0, total_outcomes: 2, confirmed_outcomes: 1, false_lead_outcomes: 0, inconclusive_outcomes: 1, new_lead_outcomes: 0, no_evidence_outcomes: 0, total_evidence: 3, supporting_evidence: 2, contradicting_evidence: 1, open_followups: 1, completed_followup_actions: 1, skipped_followup_actions: 0 },
      timeline: [
        { event_type: "SESSION_CREATED", timestamp: "2026-09-01T00:00:00Z", label: "Session created" },
        { event_type: "SESSION_COMPLETED", timestamp: "2026-09-03T00:00:00Z", label: "Session completed" }
      ]
    })),
    updateSession: vi.fn(() => Promise.resolve({})),
    deleteSession: vi.fn(() => Promise.resolve()),
    removeTaskFromSession: vi.fn(() => Promise.resolve()),
  }
}));

test("Session Detail shows stats and timeline", async () => {
  render(<MemoryRouter initialEntries={["/trees/1/research/sessions/1"]}><Routes><Route path="/trees/:treeId/research/sessions/:sessionId" element={<ResearchSessionDetail/>} /></Routes></MemoryRouter>);
  expect(await screen.findByText("Session Summary")).toBeInTheDocument();
  expect(screen.getAllByText(/3 total/).length).toBeGreaterThan(0);
  expect(screen.getAllByText(/Task progress/).length).toBeGreaterThan(0);
  expect(screen.getByText("Research Activity")).toBeInTheDocument();
  expect(screen.getByText("Activity")).toBeInTheDocument();
  expect(screen.getByText("SESSION_CREATED")).toBeInTheDocument();
});

test("Session Detail terminal empty timeline not failing", async () => {
  const { api } = await import("../../api/client");
  (api.getSession as any).mockReturnValueOnce(Promise.resolve({
    session: { id: 2, tree_id: 1, title: "Empty", description: null, status: "PLANNED", person_id: null, opportunity_id: null, created_at: "2026-09-01T00:00:00Z", updated_at: "2026-09-01T00:00:00Z", started_at: null, completed_at: null },
    person: null, opportunity: null, tasks: [], summary: { total_tasks: 0, open_tasks: 0, in_progress_tasks: 0, terminal_tasks: 0, outcomes_count: 0 },
    stats: { total_tasks: 0, completed_tasks: 0, open_tasks: 0, in_progress_tasks: 0, inconclusive_tasks: 0, rejected_tasks: 0, total_outcomes: 0, confirmed_outcomes: 0, false_lead_outcomes: 0, inconclusive_outcomes: 0, new_lead_outcomes: 0, no_evidence_outcomes: 0, total_evidence: 0, supporting_evidence: 0, contradicting_evidence: 0, open_followups: 0, completed_followup_actions: 0, skipped_followup_actions: 0 },
    timeline: []
  }));
  render(<MemoryRouter initialEntries={["/trees/1/research/sessions/2"]}><Routes><Route path="/trees/:treeId/research/sessions/:sessionId" element={<ResearchSessionDetail/>} /></Routes></MemoryRouter>);
  expect(await screen.findByText("Empty")).toBeInTheDocument();
  expect(screen.getByText(/0 outcomes/)).toBeInTheDocument();
});
