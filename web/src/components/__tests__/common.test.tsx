import { render, screen } from "@testing-library/react";
import { Loading, Empty, ErrorState, Pagination } from "../common";

test("Loading shows message", () => {
  render(<Loading msg="Loading research queue…" />);
  expect(screen.getByText("Loading research queue…")).toBeInTheDocument();
});

test("Empty shows message", () => {
  render(<Empty msg="No research opportunities found." />);
  expect(screen.getByText(/No research opportunities/)).toBeInTheDocument();
});

test("ErrorState with retry", async () => {
  const fn = vi.fn();
  render(<ErrorState msg="Unable to load" onRetry={fn} />);
  expect(screen.getByText("Unable to load")).toBeInTheDocument();
});

test("Pagination renders", () => {
  render(<Pagination limit={20} offset={0} total={100} onChange={()=>{}} />);
  expect(screen.getByText(/Page 1/)).toBeInTheDocument();
});
