import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import { vi } from "vitest";
import Evidence from "../Evidence";

vi.mock("../../api/client", () => ({
  api: {
    getEvidenceList: vi.fn(() => Promise.resolve({ items: [], pagination:{limit:20, offset:0, total:0}})),
    createEvidence: vi.fn(() => Promise.resolve({ id:1, statement:"stmt"})),
  },
}));

test("Evidence empty", async () => {
  render(<MemoryRouter initialEntries={["/trees/1/evidence"]}><Routes><Route path="/trees/:treeId/evidence" element={<Evidence/>} /></Routes></MemoryRouter>);
  expect(await screen.findByText(/No evidence yet/)).toBeInTheDocument();
});

test("Evidence list", async () => {
  const { api } = await import("../../api/client");
  (api.getEvidenceList as any).mockResolvedValueOnce({ items: [{ id:1, statement:"La partida identifica a Josep", source_id:1, source:{title:"Registro", type:"PARISH_RECORD"}, citation:{locator:"folio 42"}, created_at:new Date().toISOString()}], pagination:{limit:20, offset:0, total:1}});
  render(<MemoryRouter initialEntries={["/trees/1/evidence"]}><Routes><Route path="/trees/:treeId/evidence" element={<Evidence/>} /></Routes></MemoryRouter>);
  expect(await screen.findByText("La partida identifica a Josep")).toBeInTheDocument();
  expect(screen.getByText(/Registro/)).toBeInTheDocument();
});

test("Evidence create", async () => {
  const user = userEvent.setup();
  const { api } = await import("../../api/client");
  render(<MemoryRouter initialEntries={["/trees/1/evidence"]}><Routes><Route path="/trees/:treeId/evidence" element={<Evidence/>} /></Routes></MemoryRouter>);
  await screen.findByText(/No evidence yet/);
  await user.click(screen.getByText("Create Evidence"));
  const sourceInput = screen.getByPlaceholderText("Source ID (required)");
  await user.type(sourceInput, "1");
  const stmt = screen.getByPlaceholderText("Statement (required)");
  await user.type(stmt, "Test statement");
  await user.click(screen.getByText("Save"));
  expect(api.createEvidence).toHaveBeenCalledWith(1, expect.objectContaining({source_id:1, statement:"Test statement"}));
});
