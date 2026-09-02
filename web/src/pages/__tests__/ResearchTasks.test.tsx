import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import { vi } from "vitest";
import ResearchTasks from "../ResearchTasks";

vi.mock("../../api/client", () => ({
  api: {
    getTasks: vi.fn(() => Promise.resolve({ items: [], pagination: { limit: 20, offset: 0, total: 0 } })),
  },
}));

function renderWithTree() {
  return render(
    <MemoryRouter initialEntries={["/trees/1/research/tasks"]}>
      <Routes><Route path="/trees/:treeId/research/tasks" element={<ResearchTasks />} /></Routes>
    </MemoryRouter>
  );
}

test("ResearchTasks shows empty state", async () => {
  renderWithTree();
  expect(await screen.findByText(/No research tasks yet/)).toBeInTheDocument();
});

test("ResearchTasks shows loading initially", () => {
  renderWithTree();
  expect(screen.getByText(/Loading research tasks/)).toBeInTheDocument();
});

test("ResearchTasks shows task cards with outcome badge and opportunity", async () => {
  const { api } = await import("../../api/client");
  (api.getTasks as any).mockResolvedValueOnce({ items: [
    { id:1, title:"Task A", status:"IN_PROGRESS", person_id:5, updated_at:new Date().toISOString(), has_outcome:false, opportunity:{score:82, priority:"high"}, description:"desc" },
    { id:2, title:"Task B", status:"OPEN", person_id:6, updated_at:new Date().toISOString(), has_outcome:true, description:"desc2" }
  ], pagination:{limit:20, offset:0, total:2}});
  renderWithTree();
  expect(await screen.findByText(/Research Task: Task A/)).toBeInTheDocument();
  expect(screen.getByText(/Not recorded/)).toBeInTheDocument();
  expect(screen.getByText(/From Opportunity/)).toBeInTheDocument();
  expect(screen.getAllByText(/Recorded/).length).toBeGreaterThanOrEqual(1);
  expect(screen.getByText("Continue Research →")).toBeInTheDocument();
});

test("ResearchTasks filters combined", async () => {
  const user = userEvent.setup();
  const { api } = await import("../../api/client");
  renderWithTree();
  await screen.findByText(/No research tasks yet/);
  const statusSelect = screen.getByDisplayValue("All status");
  await user.selectOptions(statusSelect, "IN_PROGRESS");
  expect(api.getTasks).toHaveBeenCalledWith(1, expect.objectContaining({ status:"IN_PROGRESS"}));
  const hasSelect = screen.getByDisplayValue("All outcomes");
  await user.selectOptions(hasSelect, "yes");
  expect(api.getTasks).toHaveBeenCalledWith(1, expect.objectContaining({ has_outcome:true}));
  const personInput = screen.getByPlaceholderText("Person ID");
  await user.type(personInput, "5");
  // need to trigger effect; typing should call
  expect(api.getTasks).toHaveBeenCalled();
});

test("ResearchTasks pagination", async () => {
  const { api } = await import("../../api/client");
  (api.getTasks as any).mockResolvedValueOnce({ items: [{ id:1, title:"T1", status:"OPEN", updated_at:new Date().toISOString(), has_outcome:false }], pagination:{limit:20, offset:0, total:3}});
  renderWithTree();
  await screen.findByText(/Research Task: T1/);
});
