import { render, screen } from "@testing-library/react";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import { vi } from "vitest";
import ResearchWorkspace from "../ResearchWorkspace";

vi.mock("../../api/client", () => ({
  api: {
    getResearchSummary: vi.fn(() => Promise.resolve({ opportunities:{high:2,medium:3,low:5}, tasks:{open:1,in_progress:2,resolved:1,rejected:0,inconclusive:0}, outcomes:{total:1}})),
    getTasks: vi.fn(() => Promise.resolve({ items: [
      { id:1, title:"Task A", status:"IN_PROGRESS", person_id:5, updated_at:new Date().toISOString(), has_outcome:false, opportunity:{score:82, priority:"high"} },
      { id:2, title:"Task B", status:"OPEN", person_id:6, updated_at:new Date().toISOString(), has_outcome:true }
    ], pagination:{limit:5, offset:0, total:2}})),
    getOutcomes: vi.fn(() => Promise.resolve({ items: [
      { id:10, type:"CONFIRMED", summary:"Found birth", task_id:1, created_at:new Date().toISOString() }
    ], pagination:{limit:5, offset:0, total:1}})),
  },
}));

test("ResearchWorkspace shows loading then overview", async () => {
  render(<MemoryRouter initialEntries={["/trees/1/research"]}><Routes><Route path="/trees/:treeId/research" element={<ResearchWorkspace/>} /></Routes></MemoryRouter>);
  expect(screen.getByText(/Loading research workspace/)).toBeInTheDocument();
  expect(await screen.findByText("Research")).toBeInTheDocument();
  expect(await screen.findAllByText("Opportunities")).toBeTruthy();
  expect(await screen.findByText(/High priority:/)).toBeInTheDocument();
  expect(await screen.findByText("Active Tasks")).toBeInTheDocument();
  expect(await screen.findByText("Task A")).toBeInTheDocument();
  expect(screen.getByText("View all tasks")).toBeInTheDocument();
  expect(await screen.findByText("Recent Outcomes")).toBeInTheDocument();
  expect(await screen.findByText("Found birth")).toBeInTheDocument();
  expect(screen.getByText("View research history")).toBeInTheDocument();
});

test("Workspace empty states", async () => {
  const { api } = await import("../../api/client");
  (api.getTasks as any).mockResolvedValueOnce({ items: [], pagination:{limit:5, offset:0, total:0}});
  (api.getOutcomes as any).mockResolvedValueOnce({ items: [], pagination:{limit:5, offset:0, total:0}});
  render(<MemoryRouter initialEntries={["/trees/1/research"]}><Routes><Route path="/trees/:treeId/research" element={<ResearchWorkspace/>} /></Routes></MemoryRouter>);
  expect(await screen.findByText(/No research tasks yet/)).toBeInTheDocument();
  expect(await screen.findByText(/No research history yet/)).toBeInTheDocument();
});

test("Workspace error state", async () => {
  const { api } = await import("../../api/client");
  (api.getResearchSummary as any).mockRejectedValueOnce(new Error("fail"));
  render(<MemoryRouter initialEntries={["/trees/1/research"]}><Routes><Route path="/trees/:treeId/research" element={<ResearchWorkspace/>} /></Routes></MemoryRouter>);
  expect(await screen.findByText(/fail/)).toBeInTheDocument();
});
