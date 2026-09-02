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

function makeAssessment(status:string, score:number, overrides:any={}){
  return {
    status,
    score,
    evidence_total: overrides.evidence_total ?? 2,
    supporting_count: overrides.supporting_count ?? 2,
    contradicting_count: overrides.contradicting_count ?? 0,
    sources_count: overrides.sources_count ?? 2,
    cited_count: overrides.cited_count ?? 1,
    uncited_count: overrides.uncited_count ?? 1,
    reasons: overrides.reasons ?? [
      { code:"SUPPORTING_EVIDENCE", points:30, message:"Supporting evidence exists"},
      { code:"MULTIPLE_SUPPORTING_EVIDENCE", points:20, message:"Multiple supporting evidence"},
    ]
  };
}

test("assessment NO_EVIDENCE shows score 0 and counts", async ()=>{
  const outcome = { id:1, tree_id:1, task_id:1, type:"CONFIRMED", summary:"s", details:null, created_at:new Date().toISOString(), updated_at:new Date().toISOString()};
  const assessment = makeAssessment("NO_EVIDENCE",0,{evidence_total:0,supporting_count:0,contradicting_count:0,sources_count:0,cited_count:0,uncited_count:0,reasons:[]});
  (mockApi.getOutcome as any).mockResolvedValue({ ...outcome, evidence:[], evidence_assessment: assessment } as any);
  renderDetail({ outcome });
  await screen.findByText(/Find parents/);
  expect(await screen.findByText(/NO.?EVIDENCE/)).toBeInTheDocument();
  expect(screen.getByText("0 / 100")).toBeInTheDocument();
  expect(screen.getByText("0 supporting")).toBeInTheDocument();
  expect(screen.getByText("0 contradicting")).toBeInTheDocument();
  expect(screen.getByText("0 sources")).toBeInTheDocument();
  expect(screen.getByText("0 citations")).toBeInTheDocument();
});

test("assessment WEAK shows reasons", async ()=>{
  const outcome = { id:1, tree_id:1, task_id:1, type:"FALSE_LEAD", summary:"s", details:null, created_at:new Date().toISOString(), updated_at:new Date().toISOString()};
  const assessment = makeAssessment("WEAK",25,{supporting_count:1, contradicting_count:0, sources_count:1, cited_count:0, uncited_count:1, evidence_total:1,
    reasons:[{code:"SUPPORTING_EVIDENCE",points:30,message:"Supporting evidence exists"},{code:"NO_CITATION",points:-10,message:"No evidence has citation"}]});
  (mockApi.getOutcome as any).mockResolvedValue({ ...outcome, evidence:[{id:10,relationship:"SUPPORTS",statement:"stmt",source:{id:1,title:"Src"}}], evidence_assessment: assessment } as any);
  renderDetail({ outcome });
  await screen.findByText(/Find parents/);
  expect(await screen.findByText(/WEAK/)).toBeInTheDocument();
  expect(screen.getByText("25 / 100")).toBeInTheDocument();
  expect(screen.getByText("1 supporting")).toBeInTheDocument();
  expect(screen.getByText("Why this assessment?")).toBeInTheDocument();
  expect(screen.getByText("+30 Supporting evidence exists")).toBeInTheDocument();
  expect(screen.getByText("-10 No evidence has citation")).toBeInTheDocument();
});

test("assessment SUPPORTED shows correct counts", async ()=>{
  const outcome = { id:1, tree_id:1, task_id:1, type:"INCONCLUSIVE", summary:"s", details:null, created_at:new Date().toISOString(), updated_at:new Date().toISOString()};
  const assessment = makeAssessment("SUPPORTED",75,{supporting_count:2, contradicting_count:0, sources_count:1, cited_count:1, evidence_total:2});
  (mockApi.getOutcome as any).mockResolvedValue({ ...outcome, evidence:[] , evidence_assessment: assessment } as any);
  renderDetail({ outcome });
  await screen.findByText(/Find parents/);
  expect(await screen.findByText(/SUPPORTED/)).toBeInTheDocument();
  expect(screen.getByText("75 / 100")).toBeInTheDocument();
  expect(screen.getByText("2 supporting")).toBeInTheDocument();
  expect(screen.getByText("0 contradicting")).toBeInTheDocument();
});

test("assessment STRONGLY_SUPPORTED", async ()=>{
  const outcome = { id:1, tree_id:1, task_id:1, type:"NEW_LEAD", summary:"s", details:null, created_at:new Date().toISOString(), updated_at:new Date().toISOString()};
  const assessment = makeAssessment("STRONGLY_SUPPORTED",90,{supporting_count:3, contradicting_count:0, sources_count:2, cited_count:2, evidence_total:3,
    reasons:[{code:"SUPPORTING_EVIDENCE",points:30,message:"Supporting evidence exists"},{code:"SUPPORTING_EVIDENCE_HAS_CITATION",points:15,message:"Supporting evidence has citation"}]});
  (mockApi.getOutcome as any).mockResolvedValue({ ...outcome, evidence:[] , evidence_assessment: assessment } as any);
  renderDetail({ outcome });
  await screen.findByText(/Find parents/);
  expect(await screen.findByText(/STRONGLY/)).toBeInTheDocument();
  expect(screen.getByText("90 / 100")).toBeInTheDocument();
  expect(screen.getByText("3 supporting")).toBeInTheDocument();
  expect(screen.getByText("+15 Supporting evidence has citation")).toBeInTheDocument();
});

test("assessment MIXED shows contradicting", async ()=>{
  const outcome = { id:1, tree_id:1, task_id:1, type:"CONFIRMED", summary:"s", details:null, created_at:new Date().toISOString(), updated_at:new Date().toISOString()};
  const assessment = makeAssessment("MIXED",40,{supporting_count:2, contradicting_count:1, sources_count:2, cited_count:2, evidence_total:3,
    reasons:[{code:"CONTRADICTING_EVIDENCE",points:-30,message:"Contradicting evidence exists"}]});
  (mockApi.getOutcome as any).mockResolvedValue({ ...outcome, evidence:[] , evidence_assessment: assessment } as any);
  renderDetail({ outcome });
  await screen.findByText(/Find parents/);
  expect(await screen.findByText(/MIXED/)).toBeInTheDocument();
  expect(screen.getByText("1 contradicting")).toBeInTheDocument();
  expect(screen.getByText("-30 Contradicting evidence exists")).toBeInTheDocument();
});

test("CONFIRMED + NO_EVIDENCE shows warning", async ()=>{
  const outcome = { id:1, tree_id:1, task_id:1, type:"CONFIRMED", summary:"s", details:null, created_at:new Date().toISOString(), updated_at:new Date().toISOString()};
  const assessment = makeAssessment("NO_EVIDENCE",0,{evidence_total:0,supporting_count:0,contradicting_count:0,sources_count:0,cited_count:0,reasons:[]});
  (mockApi.getOutcome as any).mockResolvedValue({ ...outcome, evidence:[], evidence_assessment: assessment } as any);
  renderDetail({ outcome });
  await screen.findByText(/Find parents/);
  expect(await screen.findByText(/This outcome is marked as CONFIRMED but has no recorded supporting evidence/)).toBeInTheDocument();
});

test("CONFIRMED + MIXED shows warning", async ()=>{
  const outcome = { id:1, tree_id:1, task_id:1, type:"CONFIRMED", summary:"s", details:null, created_at:new Date().toISOString(), updated_at:new Date().toISOString()};
  const assessment = makeAssessment("MIXED",30,{supporting_count:1,contradicting_count:1});
  (mockApi.getOutcome as any).mockResolvedValue({ ...outcome, evidence:[] , evidence_assessment: assessment } as any);
  renderDetail({ outcome });
  await screen.findByText(/Find parents/);
  expect(await screen.findByText(/This outcome has contradictory evidence/)).toBeInTheDocument();
});

test("CONFIRMED + SUPPORTED does not show warnings", async ()=>{
  const outcome = { id:1, tree_id:1, task_id:1, type:"CONFIRMED", summary:"s", details:null, created_at:new Date().toISOString(), updated_at:new Date().toISOString()};
  const assessment = makeAssessment("SUPPORTED",75,{supporting_count:2,contradicting_count:0});
  (mockApi.getOutcome as any).mockResolvedValue({ ...outcome, evidence:[] , evidence_assessment: assessment, evidence_gaps:[] } as any);
  renderDetail({ outcome });
  await screen.findByText(/Find parents/);
  expect(screen.queryByText(/This outcome is marked as CONFIRMED but has no recorded supporting evidence/)).not.toBeInTheDocument();
  expect(screen.queryByText(/This outcome has contradictory evidence/)).not.toBeInTheDocument();
});

test("gaps - no gaps shows empty", async ()=>{
  const outcome = { id:1, tree_id:1, task_id:1, type:"CONFIRMED", summary:"s", details:null, created_at:new Date().toISOString(), updated_at:new Date().toISOString()};
  const assessment = makeAssessment("STRONGLY_SUPPORTED",90,{supporting_count:2, sources_count:2, cited_count:2, evidence_total:2});
  (mockApi.getOutcome as any).mockResolvedValue({ ...outcome, evidence:[], evidence_assessment: assessment, evidence_gaps:[] } as any);
  renderDetail({ outcome });
  await screen.findByText(/Find parents/);
  expect(await screen.findByText("Evidence Gaps")).toBeInTheDocument();
  expect(screen.getByText("No evidence gaps detected.")).toBeInTheDocument();
});

test("gaps - critical CONFIRMED_WITHOUT_SUPPORT", async ()=>{
  const outcome = { id:1, tree_id:1, task_id:1, type:"CONFIRMED", summary:"s", details:null, created_at:new Date().toISOString(), updated_at:new Date().toISOString()};
  const assessment = makeAssessment("NO_EVIDENCE",0,{evidence_total:0,supporting_count:0,reasons:[]});
  const gaps=[{code:"CONFIRMED_WITHOUT_SUPPORT", severity:"CRITICAL", title:"Confirmed without support", description:"This confirmed outcome has no recorded supporting evidence."}];
  (mockApi.getOutcome as any).mockResolvedValue({ ...outcome, evidence:[], evidence_assessment: assessment, evidence_gaps:gaps } as any);
  renderDetail({ outcome });
  await screen.findByText(/Find parents/);
  expect(await screen.findByText(/Confirmed without support/)).toBeInTheDocument();
  expect(screen.getByText("This confirmed outcome has no recorded supporting evidence.")).toBeInTheDocument();
  expect(screen.getByText("Add Evidence")).toBeInTheDocument();
  expect(screen.getByText(/Critical evidence gap/)).toBeInTheDocument();
});

test("gaps - warning CONTRADICTORY_EVIDENCE", async ()=>{
  const outcome = { id:1, tree_id:1, task_id:1, type:"CONFIRMED", summary:"s", details:null, created_at:new Date().toISOString(), updated_at:new Date().toISOString()};
  const assessment = makeAssessment("MIXED",40,{supporting_count:1, contradicting_count:1});
  const gaps=[{code:"CONTRADICTORY_EVIDENCE", severity:"WARNING", title:"Contradictory evidence", description:"Contradictory evidence is recorded for this outcome."}];
  (mockApi.getOutcome as any).mockResolvedValue({ ...outcome, evidence:[], evidence_assessment: assessment, evidence_gaps:gaps } as any);
  renderDetail({ outcome });
  await screen.findByText(/Find parents/);
  expect((await screen.findAllByText(/Contradictory evidence/)).length).toBeGreaterThanOrEqual(1);
  expect(screen.getByText("Review Contradictions")).toBeInTheDocument();
});

test("gaps - warning NO_CITATION quick action", async ()=>{
  const outcome = { id:1, tree_id:1, task_id:1, type:"INCONCLUSIVE", summary:"s", details:null, created_at:new Date().toISOString(), updated_at:new Date().toISOString()};
  const assessment = makeAssessment("WEAK",25,{supporting_count:1,cited_count:0});
  const gaps=[{code:"NO_CITATION", severity:"WARNING", title:"No citation", description:"Supporting evidence has no citation."}];
  (mockApi.getOutcome as any).mockResolvedValue({ ...outcome, evidence:[], evidence_assessment: assessment, evidence_gaps:gaps } as any);
  renderDetail({ outcome });
  await screen.findByText(/Find parents/);
  expect(await screen.findByText(/No citation/)).toBeInTheDocument();
  expect(screen.getByText("Review Evidence")).toBeInTheDocument();
});

test("gaps - info SINGLE_SOURCE", async ()=>{
  const outcome = { id:1, tree_id:1, task_id:1, type:"CONFIRMED", summary:"s", details:null, created_at:new Date().toISOString(), updated_at:new Date().toISOString()};
  const assessment = makeAssessment("WEAK",25,{supporting_count:1, sources_count:1});
  const gaps=[{code:"SINGLE_SOURCE", severity:"INFO", title:"Single source", description:"Evidence currently comes from a single source."}];
  (mockApi.getOutcome as any).mockResolvedValue({ ...outcome, evidence:[], evidence_assessment: assessment, evidence_gaps:gaps } as any);
  renderDetail({ outcome });
  await screen.findByText(/Find parents/);
  expect(await screen.findByText(/Single source/)).toBeInTheDocument();
  expect(screen.getByText("Evidence currently comes from a single source.")).toBeInTheDocument();
});

test("gaps - multiple gaps", async ()=>{
  const outcome = { id:1, tree_id:1, task_id:1, type:"CONFIRMED", summary:"s", details:null, created_at:new Date().toISOString(), updated_at:new Date().toISOString()};
  const assessment = makeAssessment("WEAK",25,{supporting_count:1});
  const gaps=[
    {code:"SINGLE_SUPPORTING_EVIDENCE", severity:"WARNING", title:"Single supporting evidence", description:"This outcome currently relies on a single supporting evidence record."},
    {code:"NO_CITATION", severity:"WARNING", title:"No citation", description:"Supporting evidence has no citation."},
    {code:"SINGLE_SOURCE", severity:"INFO", title:"Single source", description:"Evidence currently comes from a single source."},
  ];
  (mockApi.getOutcome as any).mockResolvedValue({ ...outcome, evidence:[], evidence_assessment: assessment, evidence_gaps:gaps } as any);
  renderDetail({ outcome });
  await screen.findByText(/Find parents/);
  expect(await screen.findByText(/Single supporting evidence/)).toBeInTheDocument();
  expect(screen.getByText(/No citation/)).toBeInTheDocument();
  expect(screen.getByText(/Single source/)).toBeInTheDocument();
});
