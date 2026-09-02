import { render, screen } from "@testing-library/react";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import Layout from "../Layout";

test("Layout shows Research nav with sublinks", async () => {
  render(<MemoryRouter initialEntries={["/trees/1/research"]}><Routes><Route path="/trees/:treeId/*" element={<Layout/>} /></Routes></MemoryRouter>);
  expect(screen.getByText("Research")).toBeInTheDocument();
  expect(screen.getByText("Overview")).toBeInTheDocument();
  expect(screen.getByText("Opportunities")).toBeInTheDocument();
  expect(screen.getByText("Tasks")).toBeInTheDocument();
  expect(screen.getByText("History")).toBeInTheDocument();
});
