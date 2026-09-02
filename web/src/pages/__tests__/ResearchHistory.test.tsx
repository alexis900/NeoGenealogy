import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import { vi } from "vitest";
import ResearchHistory from "../ResearchHistory";

vi.mock("../../api/client", () => ({
  api: {
    getOutcomes: vi.fn(() => Promise.resolve({ items: [], pagination:{limit:20, offset:0, total:0}})),
  },
}));

test("History empty", async () => {
  render(<MemoryRouter initialEntries={["/trees/1/research/history"]}><Routes><Route path="/trees/:treeId/research/history" element={<ResearchHistory/>} /></Routes></MemoryRouter>);
  expect(await screen.findByText(/No research history yet/)).toBeInTheDocument();
});

test("History shows outcomes and filters", async () => {
  const { api } = await import("../../api/client");
  (api.getOutcomes as any).mockResolvedValueOnce({ items: [
    { id:1, type:"CONFIRMED", summary:"Found", task_id:5, created_at:new Date().toISOString(), details:"d" },
    { id:2, type:"FALSE_LEAD", summary:"Bad", task_id:6, created_at:new Date().toISOString() }
  ], pagination:{limit:20, offset:0, total:2}});
  render(<MemoryRouter initialEntries={["/trees/1/research/history"]}><Routes><Route path="/trees/:treeId/research/history" element={<ResearchHistory/>} /></Routes></MemoryRouter>);
  expect(await screen.findByText("Found")).toBeInTheDocument();
  expect(screen.getByText("Bad")).toBeInTheDocument();
  expect(screen.getByText("Task 5")).toBeInTheDocument();
});

test("History filter type calls API", async () => {
  const user = userEvent.setup();
  const { api } = await import("../../api/client");
  render(<MemoryRouter initialEntries={["/trees/1/research/history"]}><Routes><Route path="/trees/:treeId/research/history" element={<ResearchHistory/>} /></Routes></MemoryRouter>);
  await screen.findByText(/No research history yet/);
  const select = screen.getByDisplayValue("All types");
  await user.selectOptions(select, "CONFIRMED");
  // should trigger new fetch with type filter
  await screen.findByText(/No research history yet/);
  expect(api.getOutcomes).toHaveBeenCalledWith(1, expect.objectContaining({ type:"CONFIRMED"}));
});

test("History loading and error", async () => {
  const { api } = await import("../../api/client");
  (api.getOutcomes as any).mockResolvedValueOnce(new Promise(()=>{})); // never resolves
  render(<MemoryRouter initialEntries={["/trees/1/research/history"]}><Routes><Route path="/trees/:treeId/research/history" element={<ResearchHistory/>} /></Routes></MemoryRouter>);
  expect(screen.getByText(/Loading history/)).toBeInTheDocument();
});
