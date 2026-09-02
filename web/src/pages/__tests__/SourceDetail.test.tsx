import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import { vi } from "vitest";
import SourceDetail from "../SourceDetail";

vi.mock("../../api/client", () => ({
  api: {
    getSource: vi.fn(() => Promise.resolve({ id:1, title:"Src", type:"BOOK", author:"A", publication:"P", date:"1874", created_at:new Date().toISOString()})),
    getCitations: vi.fn(() => Promise.resolve({ items: [], pagination:{limit:20, offset:0, total:0}})),
    updateSource: vi.fn(() => Promise.resolve({ id:1, title:"Updated", type:"BOOK"})),
    deleteSource: vi.fn(() => Promise.resolve()),
    createCitation: vi.fn(() => Promise.resolve({ id:10, locator:"loc", text:"t"})),
    deleteCitation: vi.fn(() => Promise.resolve()),
  },
}));

test("SourceDetail shows source", async () => {
  render(<MemoryRouter initialEntries={["/trees/1/sources/1"]}><Routes><Route path="/trees/:treeId/sources/:sourceId" element={<SourceDetail/>} /></Routes></MemoryRouter>);
  expect(await screen.findByText("Src")).toBeInTheDocument();
  expect(screen.getByText(/BOOK/)).toBeInTheDocument();
});

test("SourceDetail add citation", async () => {
  const user = userEvent.setup();
  const { api } = await import("../../api/client");
  render(<MemoryRouter initialEntries={["/trees/1/sources/1"]}><Routes><Route path="/trees/:treeId/sources/:sourceId" element={<SourceDetail/>} /></Routes></MemoryRouter>);
  await screen.findByText("Src");
  const locInput = screen.getByPlaceholderText("Locator (e.g. Libro III folio 42)");
  await user.type(locInput, "folio 42");
  await user.click(screen.getByRole("button", {name:"Add Citation"}));
  expect(api.createCitation).toHaveBeenCalledWith(1,1, expect.objectContaining({locator:"folio 42"}));
});
