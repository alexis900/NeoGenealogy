import { NavLink, Outlet, useParams } from "react-router-dom";
export default function Layout(){
  const {treeId} = useParams();
  const base = treeId? `/trees/${treeId}` : "";
  const link = (to:string, label:string)=> <NavLink to={to} className={({isActive})=>`block px-3 py-2 rounded ${isActive?"bg-gray-800 text-white":"hover:bg-gray-100"}`}>{label}</NavLink>;
  return <div className="min-h-screen flex flex-col">
    <header className="border-b px-4 py-3 flex justify-between items-center bg-white"><div className="font-bold">NeoGenealogy</div><div className="text-sm text-gray-600">{treeId?`Tree: ${treeId}`:""}</div></header>
    <div className="flex flex-1">
      <aside className="w-56 border-r p-3 space-y-1 bg-gray-50">
        {treeId ? <>
          {link(`${base}`, "Dashboard")}
          {link(`${base}/research`, "Research")}
          {link(`/familysearch`, "FamilySearch (Global)")}
          <div className="pl-3 space-y-1 text-sm">
            {link(`${base}/research`, "Overview")}
            {link(`${base}/research/planning`, "Planning")}
            {link(`${base}/research/opportunities`, "Opportunities")}
            {link(`${base}/research/sessions`, "Sessions")}
            <div className="pl-3">
              {link(`${base}/research/sessions/history`, "History")}
            </div>
            {link(`${base}/research/tasks`, "Tasks")}
            {link(`${base}/research/history`, "History")}
          </div>
          {link(`${base}/sources`, "Sources")}
          {link(`${base}/evidence`, "Evidence")}
          {link(`${base}/persons`, "Persons")}
          {link(`${base}/findings`, "Findings")}
          {link(`${base}/branches`, "Branches")}
          {link(`${base}/coverage`, "Coverage")}
        </> : <>
          {link("/", "Home")}
          {link("/trees", "Trees")}
          {link("/familysearch", "FamilySearch (Global)")}
        </>}
      </aside>
      <main className="flex-1 p-6 bg-white"><Outlet/></main>
    </div>
  </div>
}
