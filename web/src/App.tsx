import { BrowserRouter, Routes, Route } from "react-router-dom";
import Layout from "./components/Layout";
import Trees from "./pages/Trees";
import Dashboard from "./pages/Dashboard";
import ResearchQueue from "./pages/ResearchQueue";
import ResearchWorkspace from "./pages/ResearchWorkspace";
import ResearchHistory from "./pages/ResearchHistory";
import OpportunityDetail from "./pages/OpportunityDetail";
import ResearchTasks from "./pages/ResearchTasks";
import ResearchTaskDetail from "./pages/ResearchTaskDetail";
import Persons from "./pages/Persons";
import PersonDetail from "./pages/PersonDetail";
import Findings from "./pages/Findings";
import Branches from "./pages/Branches";
import Sources from "./pages/Sources";

export default function App(){
  return <BrowserRouter>
    <Routes>
      <Route element={<Layout/>}>
        <Route path="/" element={<Trees/>} />
        <Route path="/trees" element={<Trees/>} />
        <Route path="/trees/:treeId" element={<Dashboard/>} />
        <Route path="/trees/:treeId/research" element={<ResearchWorkspace/>} />
        <Route path="/trees/:treeId/research/opportunities" element={<ResearchQueue/>} />
        <Route path="/trees/:treeId/research/tasks" element={<ResearchTasks/>} />
        <Route path="/trees/:treeId/research/tasks/:taskId" element={<ResearchTaskDetail/>} />
        <Route path="/trees/:treeId/research/history" element={<ResearchHistory/>} />
        <Route path="/trees/:treeId/research/:oppId" element={<OpportunityDetail/>} />
        <Route path="/trees/:treeId/research/opportunities/:oppId" element={<OpportunityDetail/>} />
        <Route path="/trees/:treeId/persons" element={<Persons/>} />
        <Route path="/trees/:treeId/persons/:personId" element={<PersonDetail/>} />
        <Route path="/trees/:treeId/findings" element={<Findings/>} />
        <Route path="/trees/:treeId/branches" element={<Branches/>} />
        <Route path="/trees/:treeId/sources" element={<Sources/>} />
      </Route>
    </Routes>
  </BrowserRouter>
}
