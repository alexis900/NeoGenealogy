import { render, screen } from "@testing-library/react";
import { PriorityBadge, ScoreBadge, ConfidenceIndicator } from "../Badges";

test("PriorityBadge critical", () => {
  render(<PriorityBadge p="critical" />);
  expect(screen.getByText("critical")).toBeInTheDocument();
});

test("ScoreBadge renders score", () => {
  render(<ScoreBadge score={91} />);
  expect(screen.getByText("91")).toBeInTheDocument();
});

test("ConfidenceIndicator shows percentage", () => {
  render(<ConfidenceIndicator c={0.87} />);
  expect(screen.getByText(/87% confidence/)).toBeInTheDocument();
});
