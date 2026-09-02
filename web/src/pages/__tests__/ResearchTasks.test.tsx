import { render, screen } from "@testing-library/react";
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
