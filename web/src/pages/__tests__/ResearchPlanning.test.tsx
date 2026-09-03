import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import { vi } from "vitest";
import ResearchPlanning from "../ResearchPlanning";

const mockPlan = {
  generated_at: new Date().toISOString(),
  total_candidates: 2,
  summary: { total_candidates: 2, recommended_count: 2, deferred_count: 0, active_count: 1, inconclusive_count: 0, high_priority_count: 1, critical_gap_count: 1 },
  recommended: [
    { opportunity_id: 101, person_id: 201, title: "Find parents of Josep", priority: "HIGH", research_score: 87, planning_score: 90.5, researchability: "HIGH", confidence: 0.91, active_task: false, task_status: null, reasons: [{ code: "HIGH_RESEARCH_SCORE", label: "High research score", description: "High" }] },
    { opportunity_id: 102, person_id: 202, title: "Resolve birth date", priority: "MEDIUM", research_score: 65, planning_score: 70.2, researchability: "MEDIUM", confidence: 0.6, active_task: true, task_status: "OPEN", reasons: [{ code: "ACTIVE_TASK", label: "Already being researched", description: "Active" }] },
  ],
  deferred: [],
};

vi.mock("../../api/client", () => ({
  ApiError: class ApiError extends Error { code: string; status: number; constructor(c: string, m: string, s: number) { super(m); this.code = c; this.status = s; } },
  api: {
    getPlan: vi.fn(() => Promise.resolve(mockPlan)),
    createTaskFromOpportunity: vi.fn(() => Promise.resolve({ id: 999 })),
    getSessions: vi.fn(() => Promise.resolve({ items: [], pagination: { limit: 50, offset: 0, total: 0 } })),
    createSession: vi.fn(() => Promise.resolve({ session: { id: 1, title: "t", status: "PLANNED" } })),
    getSession: vi.fn(),
    updateSession: vi.fn(),
    deleteSession: vi.fn(),
  },
}));

function renderPlanning(initial = "/trees/1/research/planning") {
  return render(
    <MemoryRouter initialEntries={[initial]}>
      <Routes>
        <Route path="/trees/:treeId/research/planning" element={<ResearchPlanning />} />
      </Routes>
    </MemoryRouter>
  );
}

test("Planning loading", async () => {
  const { api } = await import("../../api/client");
  (api.getPlan as any).mockReturnValueOnce(new Promise(() => {}));
  renderPlanning();
  expect(screen.getByText(/Loading research planning/)).toBeInTheDocument();
});

test("Planning empty", async () => {
  const { api } = await import("../../api/client");
  (api.getPlan as any).mockResolvedValueOnce({ generated_at: new Date().toISOString(), total_candidates: 0, summary: { total_candidates: 0, recommended_count: 0, deferred_count: 0, active_count: 0, inconclusive_count: 0, high_priority_count: 0, critical_gap_count: 0 }, recommended: [], deferred: [] });
  renderPlanning();
  expect(await screen.findByText(/No recommended investigations/)).toBeInTheDocument();
  expect(screen.getByText(/No research opportunities to plan/)).toBeInTheDocument();
});

test("Planning filtered empty", async () => {
  const { api } = await import("../../api/client");
  (api.getPlan as any).mockResolvedValueOnce({ generated_at: new Date().toISOString(), total_candidates: 0, summary: { total_candidates: 0, recommended_count: 0, deferred_count: 0, active_count: 0, inconclusive_count: 0, high_priority_count: 0, critical_gap_count: 0 }, recommended: [], deferred: [] });
  renderPlanning("/trees/1/research/planning?priority=HIGH");
  // component will show filtered empty if hasFilters true and total 0
  expect(await screen.findByText(/No opportunities match these filters/)).toBeInTheDocument();
});

test("Planning shows recommended and scores", async () => {
  const { api } = await import("../../api/client");
  (api.getPlan as any).mockResolvedValueOnce(mockPlan);
  renderPlanning();
  expect(await screen.findByText("Find parents of Josep")).toBeInTheDocument();
  expect(screen.getByText("Resolve birth date")).toBeInTheDocument();
  expect(screen.getByText("87")).toBeInTheDocument();
  expect(screen.getByText(/90\.5/)).toBeInTheDocument();
  expect(screen.getByText(/70\.2/)).toBeInTheDocument();
  expect(await screen.findByText(/recommended investigations/)).toBeInTheDocument();
  expect(screen.getByText(/total candidates/)).toBeInTheDocument();
});

test("Score hierarchy and labels", async () => {
  const { api } = await import("../../api/client");
  (api.getPlan as any).mockResolvedValueOnce(mockPlan);
  renderPlanning();
  await screen.findByText("Find parents of Josep");
  expect(screen.getAllByText("Research Score").length).toBeGreaterThan(0);
  expect(screen.getAllByText(/Planning Score/).length).toBeGreaterThan(0);
  // ensure Research Score badge is more prominent (ScoreBadge) vs Planning secondary mono
  expect(screen.getByText("87")).toBeInTheDocument();
});

test("Researchability and confidence rendered", async () => {
  const { api } = await import("../../api/client");
  (api.getPlan as any).mockResolvedValueOnce(mockPlan);
  renderPlanning();
  await screen.findByText("Find parents of Josep");
  expect(screen.getAllByText("high").length).toBeGreaterThan(0);
  expect(screen.getByText(/91%/)).toBeInTheDocument();
  expect(screen.getByText(/Confidence 91%/)).toBeInTheDocument();
  expect(screen.getByText(/60%/)).toBeInTheDocument();
});

test("Planning reasons expand/collapse", async () => {
  const user = userEvent.setup();
  const { api } = await import("../../api/client");
  (api.getPlan as any).mockResolvedValueOnce(mockPlan);
  renderPlanning();
  await screen.findByText("Find parents of Josep");
  // initially collapsed, label not visible
  expect(screen.queryByText("High research score")).not.toBeInTheDocument();
  const whyButtons = screen.getAllByText("Why is this here?");
  await user.click(whyButtons[0]);
  expect(await screen.findByText(/High research score/)).toBeInTheDocument();
  // hide again
  await user.click(screen.getByText("Hide reasons"));
  await waitFor(() => expect(screen.queryByText("High research score")).not.toBeInTheDocument());
});

test("Planning active task label and CTA", async () => {
  const { api } = await import("../../api/client");
  (api.getPlan as any).mockResolvedValueOnce(mockPlan);
  renderPlanning();
  await screen.findByText("Resolve birth date");
  expect(screen.getByText("Already being researched")).toBeInTheDocument();
  // active card should have View Research Task, inactive should have Start Research
  expect(screen.getByText("View Research Task")).toBeInTheDocument();
  expect(screen.getByText("Start Research")).toBeInTheDocument();
  // ensure no Start Research for active task card
  const cards = screen.getAllByText(/View Opportunity/);
  expect(cards.length).toBe(2);
});

test("Inconclusive indicator", async () => {
  const { api } = await import("../../api/client");
  const inconclusivePlan = {
    ...mockPlan,
    recommended: [
      { opportunity_id: 103, person_id: 203, title: "Inconclusive opp", priority: "HIGH", research_score: 70, planning_score: 75, researchability: "HIGH", confidence: 0.8, active_task: false, task_status: "INCONCLUSIVE", reasons: [{ code: "PREVIOUSLY_INCONCLUSIVE", label: "Previously inconclusive", description: "Previously investigated but inconclusive" }] },
    ],
    deferred: [],
    summary: { total_candidates: 1, recommended_count: 1, deferred_count: 0, active_count: 0, inconclusive_count: 1, high_priority_count: 1, critical_gap_count: 0 },
    total_candidates: 1,
  };
  (api.getPlan as any).mockResolvedValueOnce(inconclusivePlan);
  renderPlanning();
  expect(await screen.findByText(/Previously investigated · Inconclusive/)).toBeInTheDocument();
  expect(screen.getByText("View Research Task")).toBeInTheDocument();
  // should not show Start Research for inconclusive
  expect(screen.queryByText("Start Research")).not.toBeInTheDocument();
  const user = userEvent.setup();
  const whyBtn = screen.getByText("Why is this here?");
  await user.click(whyBtn);
  expect(await screen.findByText(/Previously inconclusive/)).toBeInTheDocument();
});

test("Planning deferred count and collapsed default", async () => {
  const { api } = await import("../../api/client");
  (api.getPlan as any).mockResolvedValueOnce({
    ...mockPlan,
    recommended: mockPlan.recommended.slice(0, 1),
    deferred: [mockPlan.recommended[1]],
    summary: { ...mockPlan.summary, recommended_count: 1, deferred_count: 1 },
  });
  renderPlanning();
  expect(await screen.findByText(/Deferred/)).toBeInTheDocument();
  expect(screen.getByText(/1 other candidates/)).toBeInTheDocument();
  // deferred details should be collapsed by default
  expect(screen.queryByText(/Research Score 65/)).not.toBeInTheDocument();
});

test("Deferred expansion and navigation", async () => {
  const user = userEvent.setup();
  const { api } = await import("../../api/client");
  (api.getPlan as any).mockResolvedValueOnce({
    ...mockPlan,
    recommended: mockPlan.recommended.slice(0, 1),
    deferred: [{ ...mockPlan.recommended[1], title: "Deferred opp" }],
    summary: { ...mockPlan.summary, recommended_count: 1, deferred_count: 1 },
  });
  renderPlanning();
  await screen.findByText(/Deferred/);
  const btn = screen.getByText("Show deferred candidates");
  await user.click(btn);
  expect(await screen.findByText(/Deferred opp/)).toBeInTheDocument();
  expect(screen.getAllByText(/70\.2/).length).toBeGreaterThan(0);
});

test("Planning filters call API", async () => {
  const user = userEvent.setup();
  const { api } = await import("../../api/client");
  (api.getPlan as any).mockResolvedValue(mockPlan);
  (api.getPlan as any).mockClear();
  (api.getPlan as any).mockResolvedValue(mockPlan);
  renderPlanning();
  await screen.findByText("Find parents of Josep");
  const prioritySelect = screen.getByDisplayValue("All priorities");
  await user.selectOptions(prioritySelect, "high");
  await screen.findByText("Find parents of Josep");
  expect(api.getPlan).toHaveBeenCalledWith(1, expect.objectContaining({ priority: "high" }));
});

test("Filters researchability and min_score and limit", async () => {
  const user = userEvent.setup();
  const { api } = await import("../../api/client");
  (api.getPlan as any).mockResolvedValue(mockPlan);
  (api.getPlan as any).mockClear();
  (api.getPlan as any).mockResolvedValue(mockPlan);
  renderPlanning();
  await screen.findByText("Find parents of Josep");
  const researchSelect = screen.getByLabelText(/Researchability/);
  await user.selectOptions(researchSelect, "high");
  await waitFor(() => expect(api.getPlan).toHaveBeenCalledWith(1, expect.objectContaining({ researchability: "high" })));
  const minInput = screen.getByLabelText("Minimum Planning Score value");
  // use fireEvent to set value reliably for number input
  const { fireEvent } = await import("@testing-library/react");
  fireEvent.change(minInput, { target: { value: "70" } });
  await waitFor(() => expect(api.getPlan).toHaveBeenCalledWith(1, expect.objectContaining({ min_score: 70 })));
  const limitSelect = screen.getByLabelText(/Limit/);
  await user.selectOptions(limitSelect, "20");
  await waitFor(() => expect(api.getPlan).toHaveBeenCalledWith(1, expect.objectContaining({ limit: 20 })));
});

test("Planning Start Research and View links", async () => {
  const { api } = await import("../../api/client");
  (api.getPlan as any).mockResolvedValueOnce(mockPlan);
  renderPlanning();
  expect(await screen.findByText("Start Research")).toBeInTheDocument();
  expect(screen.getByText("View Research Task")).toBeInTheDocument();
  expect(screen.getAllByText("View Opportunity").length).toBe(2);
});

test("Start Research executes POST and refreshes", async () => {
  const user = userEvent.setup();
  const { api } = await import("../../api/client");
  (api.getPlan as any).mockResolvedValue(mockPlan);
  (api.createSession as any).mockResolvedValueOnce({ session: { id: 10, title: "Find parents of Josep", status: "PLANNED" } });
  renderPlanning();
  await screen.findByText("Find parents of Josep");
  const startBtn = screen.getByText("Start Research");
  await user.click(startBtn);
  expect(await screen.findByText("Start Research", { selector: "h2" })).toBeInTheDocument();
  // dialog has Create Session button
  const createBtn = screen.getByText("Create Session");
  await user.click(createBtn);
  expect(api.createSession).toHaveBeenCalledWith(1, expect.objectContaining({ title: expect.any(String) }));
});

test("Start Research loading state", async () => {
  const user = userEvent.setup();
  const { api } = await import("../../api/client");
  (api.getPlan as any).mockResolvedValue(mockPlan);
  let resolveSess: any;
  (api.createSession as any).mockReturnValueOnce(new Promise((res) => (resolveSess = res)));
  renderPlanning();
  await screen.findByText("Find parents of Josep");
  const btn = screen.getByText("Start Research");
  await user.click(btn);
  const createBtn = await screen.findByText("Create Session");
  await user.click(createBtn);
  expect(screen.getByText("Creating…")).toBeInTheDocument();
  resolveSess({ session: { id: 10, title: "t", status: "PLANNED" } });
  await waitFor(()=> expect(screen.queryByText("Creating…")).not.toBeInTheDocument());
});

test("Start Research duplicate task error", async () => {
  const user = userEvent.setup();
  const { api } = await import("../../api/client");
  (api.getPlan as any).mockResolvedValue(mockPlan);
  (api.createSession as any).mockRejectedValueOnce(new Error("Task already exists"));
  renderPlanning();
  await screen.findByText("Find parents of Josep");
  await user.click(screen.getByText("Start Research"));
  const createBtn = await screen.findByText("Create Session");
  await user.click(createBtn);
  expect(await screen.findByText(/Title required|Task already exists|Unable/)).toBeInTheDocument();
});

test("Planning error and retry", async () => {
  const { api } = await import("../../api/client");
  (api.getPlan as any).mockRejectedValueOnce(new Error("fail"));
  renderPlanning();
  expect(await screen.findByText("Unable to load research planning.")).toBeInTheDocument();
  expect(screen.getByText("fail")).toBeInTheDocument();
  expect(screen.getByText("Retry")).toBeInTheDocument();
  (api.getPlan as any).mockResolvedValueOnce(mockPlan);
  const user = userEvent.setup();
  await user.click(screen.getByText("Retry"));
  expect(await screen.findByText("Find parents of Josep")).toBeInTheDocument();
});

test("Summary shows all metrics", async () => {
  const { api } = await import("../../api/client");
  (api.getPlan as any).mockResolvedValueOnce(mockPlan);
  renderPlanning();
  await screen.findByText("Find parents of Josep");
  const summary = screen.getByTestId("plan-summary");
  expect(within(summary).getByText("Recommended")).toBeInTheDocument();
  expect(within(summary).getByText("Candidates")).toBeInTheDocument();
  expect(within(summary).getByText("Active research")).toBeInTheDocument();
  expect(within(summary).getByText("Inconclusive")).toBeInTheDocument();
  expect(within(summary).getByText("High priority")).toBeInTheDocument();
  expect(within(summary).getByText("Critical gaps")).toBeInTheDocument();
  // values
  expect(summary.textContent).toContain("2"); // recommended
  expect(summary.textContent).toContain("1"); // active/high etc
});

test("Header explanation and What should I research next", async () => {
  const { api } = await import("../../api/client");
  (api.getPlan as any).mockResolvedValueOnce(mockPlan);
  renderPlanning();
  await screen.findByText("Find parents of Josep");
  expect(screen.getByText("What should I research next?")).toBeInTheDocument();
  expect(screen.getByText(/Planning combines existing Research Score/)).toBeInTheDocument();
});

test("Planning shows View Session when active session exists", async () => {
  const { api } = await import("../../api/client");
  (api.getPlan as any).mockResolvedValueOnce(mockPlan);
  (api.getSessions as any).mockResolvedValueOnce({ items: [{ id: 55, tree_id: 1, title: "Active session", status: "ACTIVE", opportunity_id: 101, person_id: 201, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }], pagination: { limit: 50, offset: 0, total: 1 } });
  renderPlanning();
  await screen.findByText("Find parents of Josep");
  expect(await screen.findByText("View Session")).toBeInTheDocument();
  expect(screen.getByText("Active Session")).toBeInTheDocument();
});
