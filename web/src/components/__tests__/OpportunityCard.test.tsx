import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { OpportunityCard, ScoreBreakdown } from "../OpportunityCard";

const opp: any = {
  id: 1, person_id: 5, priority: "critical", score: 91, confidence: 0.82, researchability: "high", why: "Missing parent",
};

test("OpportunityCard shows score and priority", () => {
  render(<MemoryRouter><OpportunityCard opp={opp} treeId={1} /></MemoryRouter>);
  expect(screen.getByText("91")).toBeInTheDocument();
  expect(screen.getByText("critical")).toBeInTheDocument();
});

test("ScoreBreakdown renders components", () => {
  const bd = { total: 91, components: [{ name: "Direct ancestor", points: 30, reason: "is ancestor" }] };
  render(<ScoreBreakdown breakdown={bd as any} />);
  expect(screen.getByText("Direct ancestor")).toBeInTheDocument();
  expect(screen.getByText(/Total 91/)).toBeInTheDocument();
});
