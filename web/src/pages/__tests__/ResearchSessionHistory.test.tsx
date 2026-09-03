import { render, screen } from "@testing-library/react";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import { vi } from "vitest";
import ResearchSessionHistory from "../ResearchSessionHistory";

vi.mock("../../api/client", () => ({
  api: {
    getSessionHistory: vi.fn(() => Promise.resolve({
      items: [
        {
          id: 1, tree_id: 1, title: "Find the parents of Josep García", status: "COMPLETED", person_id: 5, opportunity_id: 10,
          created_at: new Date().toISOString(), updated_at: new Date().toISOString(), completed_at: new Date().toISOString(), description: "obj",
          stats: { total_tasks: 3, completed_tasks: 1, open_tasks: 1, in_progress_tasks: 1, inconclusive_tasks: 0, rejected_tasks: 0, total_outcomes: 2, confirmed_outcomes: 1, false_lead_outcomes: 0, inconclusive_outcomes: 0, new_lead_outcomes: 1, no_evidence_outcomes: 0, total_evidence: 3, supporting_evidence: 2, contradicting_evidence: 1, open_followups: 2, completed_followup_actions: 1, skipped_followup_actions: 0 }
        }
      ],
      pagination: { limit: 20, offset: 0, total: 1 }
    })),
    getSession: vi.fn(),
    getSessions: vi.fn(),
  }
}));

test("Session History shows completed session with stats", async () => {
  render(<MemoryRouter initialEntries={["/trees/1/research/sessions/history"]}><Routes><Route path="/trees/:treeId/research/sessions/history" element={<ResearchSessionHistory/>} /></Routes></MemoryRouter>);
  expect(await screen.findByText("Research Session History")).toBeInTheDocument();
  expect(await screen.findByText("Find the parents of Josep García")).toBeInTheDocument();
  expect(screen.getAllByText("COMPLETED").length).toBeGreaterThan(0);
  expect(screen.getByText(/3 tasks/)).toBeInTheDocument();
  expect(screen.getByText(/2 outcomes/)).toBeInTheDocument();
  expect(screen.getByText(/View Session/)).toBeInTheDocument();
});

test("Session History empty state", async () => {
  const { api } = await import("../../api/client");
  (api.getSessionHistory as any).mockReturnValueOnce(Promise.resolve({ items: [], pagination: { limit: 20, offset: 0, total: 0 } }));
  render(<MemoryRouter initialEntries={["/trees/1/research/sessions/history"]}><Routes><Route path="/trees/:treeId/research/sessions/history" element={<ResearchSessionHistory/>} /></Routes></MemoryRouter>);
  expect(await screen.findByText(/No completed research sessions yet/)).toBeInTheDocument();
});
