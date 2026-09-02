import { render, screen } from "@testing-library/react";
import { TaskStatusBadge } from "../Badges";

test("TaskStatusBadge OPEN", () => {
  render(<TaskStatusBadge status="OPEN" />);
  expect(screen.getByText("OPEN")).toBeInTheDocument();
});

test("TaskStatusBadge IN_PROGRESS", () => {
  render(<TaskStatusBadge status="IN_PROGRESS" />);
  expect(screen.getByText("IN PROGRESS")).toBeInTheDocument();
});

test("TaskStatusBadge RESOLVED", () => {
  render(<TaskStatusBadge status="RESOLVED" />);
  expect(screen.getByText("RESOLVED")).toBeInTheDocument();
});
