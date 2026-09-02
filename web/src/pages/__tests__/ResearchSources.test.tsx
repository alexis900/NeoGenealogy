import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import { vi } from "vitest";
import ResearchSources from "../ResearchSources";

vi.mock("../../api/client", () => ({
  api: {
    getSources: vi.fn(() => Promise.resolve({ items: [], pagination:{limit:20, offset:0, total:0}})),
    createSource: vi.fn(() => Promise.resolve({ id:1, title:"New", type:"BOOK"})),
  },
}));

test("Sources empty", async () => {
  render(<MemoryRouter initialEntries={["/trees/1/sources"]}><Routes><Route path="/trees/:treeId/sources" element={<ResearchSources/>} /></Routes></MemoryRouter>);
  expect(await screen.findByText(/No sources yet/)).toBeInTheDocument();
});

test("Sources list", async () => {
  const { api } = await import("../../api/client");
  (api.getSources as any).mockResolvedValueOnce({ items: [{ id:1, title:"Registro parroquial", type:"PARISH_RECORD", author:"A", date:"1874", publication:"Pub"}], pagination:{limit:20, offset:0, total:1}});
  render(<MemoryRouter initialEntries={["/trees/1/sources"]}><Routes><Route path="/trees/:treeId/sources" element={<ResearchSources/>} /></Routes></MemoryRouter>);
  expect(await screen.findByText("Registro parroquial")).toBeInTheDocument();
  expect(screen.getAllByText(/PARISH_RECORD/).length).toBeGreaterThanOrEqual(1);
});

test("Sources create", async () => {
  const user = userEvent.setup();
  const { api } = await import("../../api/client");
  render(<MemoryRouter initialEntries={["/trees/1/sources"]}><Routes><Route path="/trees/:treeId/sources" element={<ResearchSources/>} /></Routes></MemoryRouter>);
  await screen.findByText(/No sources yet/);
  await user.click(screen.getByText("Create Source"));
  const input = screen.getByPlaceholderText("Title (required)");
  await user.type(input, "New Source");
  await user.click(screen.getByText("Save"));
  expect(api.createSource).toHaveBeenCalledWith(1, expect.objectContaining({title:"New Source"}));
});

test("Sources loading and error", async () => {
  const { api } = await import("../../api/client");
  (api.getSources as any).mockRejectedValueOnce(new Error("fail"));
  render(<MemoryRouter initialEntries={["/trees/1/sources"]}><Routes><Route path="/trees/:treeId/sources" element={<ResearchSources/>} /></Routes></MemoryRouter>);
  expect(await screen.findByText(/fail/)).toBeInTheDocument();
});
