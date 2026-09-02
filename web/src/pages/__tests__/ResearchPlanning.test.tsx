import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import { vi } from "vitest";
import ResearchPlanning from "../ResearchPlanning";

const mockPlan = {
  generated_at: new Date().toISOString(),
  total_candidates: 2,
  summary: { total_candidates:2, recommended_count:2, deferred_count:0, active_count:1, inconclusive_count:0, high_priority_count:1, critical_gap_count:1 },
  recommended: [
    { opportunity_id: 101, person_id: 201, title: "Find parents of Josep", priority: "HIGH", research_score: 87, planning_score: 90.5, researchability:"HIGH", confidence:0.91, active_task:false, task_status:null, reasons:[{code:"HIGH_RESEARCH_SCORE",label:"High research score",description:"High"}] },
    { opportunity_id: 102, person_id: 202, title: "Resolve birth date", priority: "MEDIUM", research_score: 65, planning_score: 70.2, researchability:"MEDIUM", confidence:0.6, active_task:true, task_status:"OPEN", reasons:[{code:"ACTIVE_TASK",label:"Already being researched",description:"Active"}] },
  ],
  deferred: []
};

vi.mock("../../api/client", () => ({
  api: {
    getPlan: vi.fn(() => Promise.resolve(mockPlan)),
    createTaskFromOpportunity: vi.fn(() => Promise.resolve({id: 999})),
  },
}));



test("Planning loading", async () => {
  const { api } = await import("../../api/client");
  (api.getPlan as any).mockReturnValueOnce(new Promise(()=>{}));
  render(<MemoryRouter initialEntries={["/trees/1/research/planning"]}><Routes><Route path="/trees/:treeId/research/planning" element={<ResearchPlanning/>} /></Routes></MemoryRouter>);
  expect(screen.getByText(/Loading research planning/)).toBeInTheDocument();
});

test("Planning empty", async () => {
  const { api } = await import("../../api/client");
  (api.getPlan as any).mockResolvedValueOnce({ generated_at:new Date().toISOString(), total_candidates:0, summary:{total_candidates:0,recommended_count:0,deferred_count:0,active_count:0,inconclusive_count:0,high_priority_count:0,critical_gap_count:0}, recommended:[], deferred:[] });
  render(<MemoryRouter initialEntries={["/trees/1/research/planning"]}><Routes><Route path="/trees/:treeId/research/planning" element={<ResearchPlanning/>} /></Routes></MemoryRouter>);
  expect(await screen.findByText(/No recommended investigations/)).toBeInTheDocument();
});

test("Planning shows recommended and scores", async () => {
  const { api } = await import("../../api/client");
  (api.getPlan as any).mockResolvedValueOnce(mockPlan);
  render(<MemoryRouter initialEntries={["/trees/1/research/planning"]}><Routes><Route path="/trees/:treeId/research/planning" element={<ResearchPlanning/>} /></Routes></MemoryRouter>);
  expect(await screen.findByText("Find parents of Josep")).toBeInTheDocument();
  expect(screen.getByText("Resolve birth date")).toBeInTheDocument();
  expect(screen.getByText("87")).toBeInTheDocument(); // research_score badge
  expect(screen.getByText(/90\.5/)).toBeInTheDocument();
  expect(screen.getByText(/70\.2/)).toBeInTheDocument();
  expect(await screen.findByText(/recommended investigations/)).toBeInTheDocument();
  expect(screen.getByText(/total candidates/)).toBeInTheDocument();
});

test("Planning reasons and active task", async () => {
  const { api } = await import("../../api/client");
  (api.getPlan as any).mockResolvedValueOnce(mockPlan);
  render(<MemoryRouter initialEntries={["/trees/1/research/planning"]}><Routes><Route path="/trees/:treeId/research/planning" element={<ResearchPlanning/>} /></Routes></MemoryRouter>);
  expect(await screen.findByText(/High research score/)).toBeInTheDocument();
  expect(await screen.findByText(/Already being researched/)).toBeInTheDocument();
});

test("Planning deferred count", async () => {
  const { api } = await import("../../api/client");
  (api.getPlan as any).mockResolvedValueOnce({
    ...mockPlan,
    recommended: mockPlan.recommended.slice(0,1),
    deferred: [mockPlan.recommended[1]],
    summary:{...mockPlan.summary, recommended_count:1, deferred_count:1}
  });
  render(<MemoryRouter initialEntries={["/trees/1/research/planning"]}><Routes><Route path="/trees/:treeId/research/planning" element={<ResearchPlanning/>} /></Routes></MemoryRouter>);
  expect(await screen.findByText(/Deferred/)).toBeInTheDocument();
});

test("Planning filters call API", async () => {
  const user = userEvent.setup();
  const { api } = await import("../../api/client");
  (api.getPlan as any).mockResolvedValue(mockPlan);
  // reset call history
  (api.getPlan as any).mockClear();
  (api.getPlan as any).mockResolvedValue(mockPlan);
  render(<MemoryRouter initialEntries={["/trees/1/research/planning"]}><Routes><Route path="/trees/:treeId/research/planning" element={<ResearchPlanning/>} /></Routes></MemoryRouter>);
  await screen.findByText("Find parents of Josep");
  const prioritySelect = screen.getByDisplayValue("All priorities");
  await user.selectOptions(prioritySelect, "high");
  await screen.findByText("Find parents of Josep");
  expect(api.getPlan).toHaveBeenCalledWith(1, expect.objectContaining({ priority:"high" }));
});

test("Planning Start Research and View links", async () => {
  const { api } = await import("../../api/client");
  (api.getPlan as any).mockResolvedValueOnce(mockPlan);
  render(<MemoryRouter initialEntries={["/trees/1/research/planning"]}><Routes><Route path="/trees/:treeId/research/planning" element={<ResearchPlanning/>} /></Routes></MemoryRouter>);
  expect(await screen.findByText("Start Research")).toBeInTheDocument();
  expect(screen.getByText("View Research Task")).toBeInTheDocument();
  expect(screen.getAllByText("View Opportunity").length).toBe(2);
});

test("Planning error and retry", async () => {
  const { api } = await import("../../api/client");
  (api.getPlan as any).mockRejectedValueOnce(new Error("fail"));
  render(<MemoryRouter initialEntries={["/trees/1/research/planning"]}><Routes><Route path="/trees/:treeId/research/planning" element={<ResearchPlanning/>} /></Routes></MemoryRouter>);
  expect(await screen.findByText("fail")).toBeInTheDocument();
  expect(screen.getByText("Retry")).toBeInTheDocument();
});
