import { render, screen } from "@testing-library/react";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import { vi } from "vitest";
import ResearchSessions from "../ResearchSessions";
import ResearchSessionDetail from "../ResearchSessionDetail";

vi.mock("../../api/client", () => ({
  api: {
    getSessions: vi.fn(() => Promise.resolve({ items: [
      { id: 1, tree_id: 1, title: "Find parents", status: "ACTIVE", person_id: 5, opportunity_id: 10, created_at: new Date().toISOString(), updated_at: new Date().toISOString(), description: "obj" },
      { id: 2, tree_id: 1, title: "Soler origin", status: "PLANNED", person_id: null, opportunity_id: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString(), description: null },
    ], pagination: { limit: 50, offset: 0, total: 2 } })),
    getSession: vi.fn(() => Promise.resolve({
      session: { id: 1, tree_id: 1, title: "Find parents", description: "obj", status: "ACTIVE", person_id: 5, opportunity_id: 10, created_at: new Date().toISOString(), updated_at: new Date().toISOString(), started_at: new Date().toISOString(), completed_at: null },
      person: { id: 5, name: "Josep García", gedcom_id: "I1" },
      opportunity: { id: 10, title: "Find his parents", priority: "HIGH", score: 80, person_id: 5 },
      tasks: [{ id: 100, tree_id: 1, title: "Task A", status: "OPEN", person_id: 5, updated_at: new Date().toISOString(), has_outcome: false }],
      summary: { total_tasks: 1, open_tasks: 1, in_progress_tasks: 0, terminal_tasks: 0, outcomes_count: 0 }
    })),
    getTasks: vi.fn(() => Promise.resolve({ items: [], pagination: { limit: 50, offset: 0, total: 0 } })),
    createSession: vi.fn(() => Promise.resolve({ session: { id: 3, title: "New" } })),
    updateSession: vi.fn((_tid:number,sid:number,body:any)=> Promise.resolve({ session: { id: sid, title: "Find parents", status: body.status||"ACTIVE" } })),
    deleteSession: vi.fn(() => Promise.resolve()),
    removeTaskFromSession: vi.fn(() => Promise.resolve()),
  },
}));

test("ResearchSessions list shows sessions", async () => {
  render(<MemoryRouter initialEntries={["/trees/1/research/sessions"]}><Routes><Route path="/trees/:treeId/research/sessions" element={<ResearchSessions/>} /></Routes></MemoryRouter>);
  expect(await screen.findByText("Find parents")).toBeInTheDocument();
  expect(screen.getByText("Soler origin")).toBeInTheDocument();
  expect(screen.getAllByText("ACTIVE").length).toBeGreaterThan(0);
  expect(screen.getAllByText("PLANNED").length).toBeGreaterThan(0);
});

test("ResearchSessionDetail shows tasks and progress", async () => {
  render(<MemoryRouter initialEntries={["/trees/1/research/sessions/1"]}><Routes><Route path="/trees/:treeId/research/sessions/:sessionId" element={<ResearchSessionDetail/>} /></Routes></MemoryRouter>);
  expect(await screen.findByText("Find parents")).toBeInTheDocument();
  expect(screen.getByText("Objective")).toBeInTheDocument();
  expect(screen.getByText("Task A")).toBeInTheDocument();
  expect(screen.getAllByText(/Progress/).length).toBeGreaterThan(0);
});
