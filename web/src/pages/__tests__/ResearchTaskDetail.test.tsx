import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter, Routes, Route } from "react-router-dom";
import { vi } from "vitest";
import ResearchTaskDetail from "../ResearchTaskDetail";

const mockApi = vi.hoisted(() => ({
  getTask: vi.fn() as any,
  getOpportunities: vi.fn(() => Promise.resolve({ items: [] } as any)) as any,
  updateTask: vi.fn() as any,
  deleteTask: vi.fn(() => Promise.resolve() as any) as any,
  createOutcome: vi.fn() as any,
  updateOutcome: vi.fn() as any,
  deleteOutcome: vi.fn(() => Promise.resolve() as any) as any,
  getOutcome: vi.fn(() => Promise.resolve({ id:1, type:"CONFIRMED", summary:"s", evidence:[] } as any)) as any,
  getOutcomeEvidence: vi.fn(() => Promise.resolve({ items: [] } as any)) as any,
  getSources: vi.fn(() => Promise.resolve({ items: [] } as any)) as any,
  getCitations: vi.fn(() => Promise.resolve({ items: [] } as any)) as any,
  createEvidence: vi.fn(() => Promise.resolve({ id:10, statement:"stmt"} as any)) as any,
  attachEvidence: vi.fn(() => Promise.resolve({} as any)) as any,
  detachEvidence: vi.fn(() => Promise.resolve() as any) as any,
}));

vi.mock("../../api/client", () => ({
  api: mockApi,
}));

function makeTask(overrides: any = {}) {
  return {
    id: 1,
    tree_id: 1,
    title: "Find parents",
    status: "OPEN",
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
    description: "test",
    person_id: 5,
    opportunity_id: null,
    outcome: null,
    ...overrides,
  };
}

function renderDetail(taskOverrides: any = {}, opp: any = null) {
  const task = makeTask(taskOverrides);
  mockApi.getTask.mockResolvedValue(task);
  if (opp) {
    (mockApi.getOpportunities as any).mockResolvedValue({ items: [opp] });
  } else {
    (mockApi.getOpportunities as any).mockResolvedValue({ items: [] });
  }
  // mock confirm
  (global as any).confirm = vi.fn(() => true);
  // mock location
  delete (window as any).location;
  (window as any).location = { href: "" };

  render(
    <MemoryRouter initialEntries={["/trees/1/research/tasks/1"]}>
      <Routes><Route path="/trees/:treeId/research/tasks/:taskId" element={<ResearchTaskDetail />} /></Routes>
    </MemoryRouter>
  );
  return task;
}

beforeEach(() => {
  vi.clearAllMocks();
  mockApi.getOpportunities.mockResolvedValue({ items: [] });
  mockApi.deleteTask.mockResolvedValue(undefined);
  mockApi.deleteOutcome.mockResolvedValue(undefined);
});

test("shows loading then task", async () => {
  renderDetail();
  expect(screen.getByText(/Loading task/)).toBeInTheDocument();
  expect(await screen.findByText(/Find parents/)).toBeInTheDocument();
});

test("task without outcome shows Record Outcome", async () => {
  renderDetail({ outcome: null });
  expect(await screen.findByText(/Find parents/)).toBeInTheDocument();
  expect(screen.getByText("No research outcome recorded yet.")).toBeInTheDocument();
  expect(screen.getByText("Record Outcome")).toBeInTheDocument();
  // edit outcome section should not be shown
  expect(screen.queryByText("Edit Outcome")).not.toBeInTheDocument();
});

test("create outcome - type/summary/details and show after", async () => {
  const user = userEvent.setup();
  renderDetail({ outcome: null });
  await screen.findByText(/Find parents/);
  // type select default is CONFIRMED
  const typeSelect = screen.getAllByDisplayValue("Confirmed")[0];
  await user.selectOptions(typeSelect, "NEW_LEAD");
  const summaryInput = screen.getByPlaceholderText("Summary (required)");
  await user.type(summaryInput, "found new lead in parish");
  const detailsInput = screen.getByPlaceholderText("Details (optional)");
  await user.type(detailsInput, "check 1880 census page 12");

  const createdOutcome = {
    id: 10,
    tree_id: 1,
    task_id: 1,
    type: "NEW_LEAD",
    summary: "found new lead in parish",
    details: "check 1880 census page 12",
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };
  mockApi.createOutcome.mockResolvedValue(createdOutcome);

  await user.click(screen.getByText("Record Outcome"));

  await waitFor(() => expect(mockApi.createOutcome).toHaveBeenCalledWith(1, 1, {
    type: "NEW_LEAD",
    summary: "found new lead in parish",
    details: "check 1880 census page 12",
  }));
  // After creation, outcome should be displayed
  expect(await screen.findByText("found new lead in parish")).toBeInTheDocument();
  expect(screen.getAllByText("New lead").length).toBeGreaterThanOrEqual(1);
  expect(screen.getAllByText("check 1880 census page 12").length).toBeGreaterThanOrEqual(1);
  // Now Edit Outcome should appear, and Record Outcome should not
  expect(screen.getByText("Edit Outcome")).toBeInTheDocument();
  expect(screen.queryByText("No research outcome recorded yet.")).not.toBeInTheDocument();
});

test("edit outcome - modify type/summary/details", async () => {
  const user = userEvent.setup();
  const existingOutcome = {
    id: 10,
    tree_id: 1,
    task_id: 1,
    type: "INCONCLUSIVE",
    summary: "initial summary",
    details: "initial details",
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };
  renderDetail({ outcome: existingOutcome });
  await screen.findByText(/Find parents/);
  expect(screen.getByText("initial summary")).toBeInTheDocument();
  expect(screen.getByText("Edit Outcome")).toBeInTheDocument();
  // Should not show Record Outcome button
  expect(screen.queryByText("No research outcome recorded yet.")).not.toBeInTheDocument();

  // edit fields - there is a select and inputs for outcome
  const selects = screen.getAllByDisplayValue("Inconclusive");
  // outcome type select is the one near Edit Outcome; there is also status select
  // Find the outcome type select among them
  const outcomeSelect = selects.find(el => el.closest("div")?.textContent?.includes("Edit Outcome") || true)!;
  await user.selectOptions(outcomeSelect, "CONFIRMED");

  const summaryInputs = screen.getAllByDisplayValue("initial summary");
  const outcomeSummaryInput = summaryInputs[summaryInputs.length - 1];
  await user.clear(outcomeSummaryInput);
  await user.type(outcomeSummaryInput, "updated summary");

  const detailsInputs = screen.getAllByDisplayValue("initial details");
  const outcomeDetailsInput = detailsInputs[detailsInputs.length - 1];
  await user.clear(outcomeDetailsInput);
  await user.type(outcomeDetailsInput, "updated details");

  const updatedOutcome = { ...existingOutcome, type: "CONFIRMED", summary: "updated summary", details: "updated details" };
  mockApi.updateOutcome.mockResolvedValue(updatedOutcome);

  await user.click(screen.getByText("Update Outcome"));

  await waitFor(() => expect(mockApi.updateOutcome).toHaveBeenCalledWith(1, 10, {
    type: "CONFIRMED",
    summary: "updated summary",
    details: "updated details",
  }));
  expect(await screen.findByText("updated summary")).toBeInTheDocument();
});

test("delete outcome shows Record Outcome again", async () => {
  const user = userEvent.setup();
  const existingOutcome = {
    id: 10,
    tree_id: 1,
    task_id: 1,
    type: "CONFIRMED",
    summary: "to delete",
    details: null,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };
  renderDetail({ outcome: existingOutcome });
  await screen.findByText(/Find parents/);
  expect(screen.getByText("to delete")).toBeInTheDocument();

  await user.click(screen.getByText("Delete Outcome"));
  await waitFor(() => expect(mockApi.deleteOutcome).toHaveBeenCalledWith(1, 10));
  // After delete, should show Record Outcome again
  expect(await screen.findByText("No research outcome recorded yet.")).toBeInTheDocument();
  expect(screen.getByText("Record Outcome")).toBeInTheDocument();
  expect(screen.queryByText("to delete")).not.toBeInTheDocument();
});

test("all five outcome types render and selectable", async () => {
  const user = userEvent.setup();
  renderDetail({ outcome: null });
  await screen.findByText(/Find parents/);
  const typeSelect = screen.getAllByDisplayValue("Confirmed")[0];
  const options = Array.from(typeSelect.querySelectorAll("option")).map(o => o.value);
  expect(options).toEqual(["CONFIRMED", "FALSE_LEAD", "INCONCLUSIVE", "NEW_LEAD", "NO_EVIDENCE"]);

  for (const t of ["CONFIRMED", "FALSE_LEAD", "INCONCLUSIVE", "NEW_LEAD", "NO_EVIDENCE"] as const) {
    await user.selectOptions(typeSelect, t);
    expect((typeSelect as HTMLSelectElement).value).toBe(t);
  }

  // also verify badge shows for existing outcome (already covered by create/edit tests)
  void mockApi.getTask;
});

test("existing outcome does not show duplicate creation form", async () => {
  const existingOutcome = {
    id: 10,
    tree_id: 1,
    task_id: 1,
    type: "CONFIRMED",
    summary: "existing",
    details: null,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };
  renderDetail({ outcome: existingOutcome });
  await screen.findByText(/Find parents/);
  expect(screen.getByText("existing")).toBeInTheDocument();
  // Only one Record Outcome scenario, now should show Edit Outcome not second Record
  expect(screen.queryByText("No research outcome recorded yet.")).not.toBeInTheDocument();
  expect(screen.getByText("Edit Outcome")).toBeInTheDocument();
  // There should not be a second "Record Outcome" button
  expect(screen.queryAllByText("Record Outcome").length).toBe(0);
});

test("Original Opportunity remains visible after create and update", async () => {
  const user = userEvent.setup();
  const opp = {
    id: 5,
    priority: "high",
    score: 85,
    why: "missing birth date",
    what: { hint: "check census" },
    potential_sources: ["census"],
    breakdown: { total: 85, components: [{ name: "x", points: 10, reason: "y" }] }
  };
  renderDetail({ outcome: null, opportunity_id: 5 }, opp);
  await screen.findByText(/Find parents/);
  expect(screen.getByText("Original Research Opportunity")).toBeInTheDocument();
  expect(screen.getByText("missing birth date")).toBeInTheDocument();

  // create outcome
  const summaryInput = screen.getByPlaceholderText("Summary (required)");
  await user.type(summaryInput, "my summary");
  mockApi.createOutcome.mockResolvedValue({
    id: 10, tree_id: 1, task_id: 1, type: "CONFIRMED", summary: "my summary", details: null,
    created_at: new Date().toISOString(), updated_at: new Date().toISOString()
  });
  await user.click(screen.getByText("Record Outcome"));
  await waitFor(() => expect(mockApi.createOutcome).toHaveBeenCalled());
  // opportunity still visible
  expect(screen.getByText("Original Research Opportunity")).toBeInTheDocument();

  // update
  const updated = { id: 10, tree_id: 1, task_id: 1, type: "CONFIRMED", summary: "my summary edited", details: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() };
  mockApi.updateOutcome.mockResolvedValue(updated);
  // edit summary
  const editInput = screen.getByDisplayValue("my summary");
  await user.clear(editInput);
  await user.type(editInput, "my summary edited");
  await user.click(screen.getByText("Update Outcome"));
  await waitFor(() => expect(mockApi.updateOutcome).toHaveBeenCalled());
  expect(screen.getByText("Original Research Opportunity")).toBeInTheDocument();
});

test("shows error state when API fails", async () => {
  mockApi.getTask.mockRejectedValue(new Error("network error"));
  render(
    <MemoryRouter initialEntries={["/trees/1/research/tasks/1"]}>
      <Routes><Route path="/trees/:treeId/research/tasks/:taskId" element={<ResearchTaskDetail />} /></Routes>
    </MemoryRouter>
  );
  expect(await screen.findByText(/network error/)).toBeInTheDocument();
});

test("empty no outcome has disabled Record button when summary empty", async () => {
  renderDetail({ outcome: null });
  await screen.findByText(/Find parents/);
  const btn = screen.getByText("Record Outcome") as HTMLButtonElement;
  expect(btn.disabled).toBe(true);
});

test("Record Outcome enabled when summary filled", async () => {
  const user = userEvent.setup();
  renderDetail({ outcome: null });
  await screen.findByText(/Find parents/);
  const input = screen.getByPlaceholderText("Summary (required)");
  await user.type(input, "non-empty");
  const btn = screen.getByText("Record Outcome") as HTMLButtonElement;
  expect(btn.disabled).toBe(false);
});

test("outcome shows evidence and can add", async () => {
  const user = userEvent.setup();
  const outcome = { id:1, tree_id:1, task_id:1, type:"CONFIRMED", summary:"s", details:null, created_at:new Date().toISOString(), updated_at:new Date().toISOString()};
  (mockApi.getOutcome as any).mockResolvedValue({ ...outcome, evidence: [{ id:10, relationship:"SUPPORTS", statement:"stmt", source:{id:1, title:"Src"}, citation:{id:2, locator:"folio"} }] } as any);
  (mockApi.getSources as any).mockResolvedValue({ items: [{id:1, title:"Src", type:"BOOK"}], pagination:{limit:100, offset:0, total:1}} as any);
  (mockApi.getCitations as any).mockResolvedValue({ items: [{id:2, locator:"folio"}]} as any);
  renderDetail({ outcome });
  await screen.findByText(/Find parents/);
  expect(await screen.findByText("Evidence")).toBeInTheDocument();
  expect(await screen.findByText("✓ SUPPORTS")).toBeInTheDocument();
  expect(screen.getByText("Src")).toBeInTheDocument();
  await user.click(screen.getByText("+ Add Evidence"));
  // select source
  const sourceSelect = screen.getByDisplayValue("Select Source");
  await user.selectOptions(sourceSelect, "1");
  await user.type(screen.getByPlaceholderText("Statement (required)"), "New statement");
  await user.click(screen.getByText("Save Evidence"));
  expect(mockApi.createEvidence).toHaveBeenCalled();
  expect(mockApi.attachEvidence).toHaveBeenCalledWith(1,1,10, expect.objectContaining({relationship:"SUPPORTS"}));
});

test("evidence contradict label", async () => {
  const outcome = { id:1, tree_id:1, task_id:1, type:"CONFIRMED", summary:"s", details:null, created_at:new Date().toISOString(), updated_at:new Date().toISOString()};
  (mockApi.getOutcome as any).mockResolvedValue({ ...outcome, evidence: [{ id:11, relationship:"CONTRADICTS", statement:"contra", source:{id:1, title:"Src"} }] } as any);
  renderDetail({ outcome });
  await screen.findByText(/Find parents/);
  expect(await screen.findByText("⚠ CONTRADICTS")).toBeInTheDocument();
});
