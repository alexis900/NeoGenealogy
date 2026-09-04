import { useState, useEffect } from "react";
import { Link } from "react-router-dom";
import { api } from "../api/client";
import { Loading, ErrorState } from "../components/common";

export default function FamilySearchGlobalSearch(){
  const [q,setQ]=useState("");
  const [givenName,setGivenName]=useState("");
  const [surname,setSurname]=useState("");
  const [birthDate,setBirthDate]=useState("");
  const [results,setResults]=useState<any[]>([]);
  const [loading,setLoading]=useState(false);
  const [err,setErr]=useState<string|null>(null);
  const [authStatus,setAuthStatus]=useState<any>(null);

  useEffect(()=>{
    api.getFamilySearchAuthStatus().then(setAuthStatus).catch(()=>{});
  },[]);

  const search=async()=>{
    const query = q.trim() || [givenName, surname, birthDate].filter(Boolean).join(" ");
    if(!query && !surname && !givenName){
      setErr("Introduce al menos nombre o q");
      return;
    }
    setLoading(true); setErr(null);
    try{
      const res:any = await api.familySearchGlobalSearch({q: query, givenName, surname, birthLikeDate: birthDate});
      setResults(res.results || []);
    }catch(e:any){ setErr(e.message) }
    finally{ setLoading(false) }
  };

  const connect=async()=>{
    try{
      const r:any = await api.authorizeFamilySearch();
      window.location.href = r.authorization_url;
    }catch(e:any){ setErr(e.message) }
  };
  const disconnect=async()=>{
    try{ await api.disconnectFamilySearch(); setAuthStatus({...authStatus, connected:false, status:"not_configured"}); }catch(e:any){ setErr(e.message)}
  };

  return <div className="space-y-6 p-4">
    <h1 className="text-2xl font-bold">FamilySearch — Búsqueda Global</h1>
    <div className="text-xs text-gray-600">Búsqueda sin depender de un árbol específico. Usa la conexión FamilySearch global (no persiste ResearchQuery).</div>
    <div className="text-xs text-amber-800 bg-amber-50 border border-amber-200 rounded p-2">FamilySearch Result ≠ Evidence — resultados son candidatos, no se convierten en Evidence automáticamente.</div>
    
    <div className="border rounded p-4 bg-white space-y-2">
      <h3 className="font-semibold">Conexión FamilySearch</h3>
      {authStatus ? (
        <div className="text-sm space-y-1">
          <div>Estado: <span className={`px-2 py-0.5 rounded text-xs ${authStatus.connected ? "bg-emerald-100 text-emerald-800" : "bg-amber-100 text-amber-800"}`}>{authStatus.status}</span> {authStatus.connected ? "Conectado" : "No conectado"}</div>
          <div className="text-xs text-gray-600">Configurado: {authStatus.configured ? "sí" : "no"} · Requiere auth: {authStatus.requires_auth ? "sí" : "no"}</div>
          {authStatus.expires_at && <div className="text-xs">Expira: {new Date(authStatus.expires_at).toLocaleString()}</div>}
          <div className="text-xs text-gray-500">Redirect URI: <code>{authStatus.redirect_uri}</code></div>
          <div className="flex gap-2 mt-2">
            {!authStatus.connected && <button onClick={connect} className="px-3 py-1 bg-blue-600 text-white rounded text-sm">Conectar con FamilySearch</button>}
            {authStatus.connected && <button onClick={disconnect} className="px-3 py-1 border rounded text-sm">Desconectar</button>}
            <Link to="/trees" className="px-3 py-1 border rounded text-sm">Ver árboles</Link>
          </div>
          {!authStatus.configured && !authStatus.connected && <div className="text-xs text-amber-700 mt-1">Configura NEOGENEALOGY_FAMILYSEARCH_CLIENT_ID en el servidor o conecta vía OAuth.</div>}
        </div>
      ) : <Loading msg="Cargando estado…" />}
    </div>

    <div className="border rounded p-4 bg-white space-y-3">
      <h3 className="font-semibold">Búsqueda Libre (sin árbol)</h3>
      <div className="grid grid-cols-2 gap-2">
        <label className="block"><span className="text-xs">q (texto libre)</span><input value={q} onChange={e=>setQ(e.target.value)} placeholder="Josep García 1882" className="w-full border rounded px-2 py-1" /></label>
        <label className="block"><span className="text-xs">Nombre</span><input value={givenName} onChange={e=>setGivenName(e.target.value)} placeholder="Josep" className="w-full border rounded px-2 py-1" /></label>
        <label className="block"><span className="text-xs">Apellido</span><input value={surname} onChange={e=>setSurname(e.target.value)} placeholder="García" className="w-full border rounded px-2 py-1" /></label>
        <label className="block"><span className="text-xs">Año nacimiento</span><input value={birthDate} onChange={e=>setBirthDate(e.target.value)} placeholder="1882" className="w-full border rounded px-2 py-1" /></label>
      </div>
      <button onClick={search} disabled={loading} className="px-4 py-2 bg-emerald-600 text-white rounded disabled:opacity-50">{loading?"Buscando…":"Buscar en FamilySearch"}</button>
      {err && <ErrorState msg={err} />}
    </div>

    <div className="border rounded p-4 bg-white">
      <h3 className="font-semibold">Resultados ({results.length}) — Global, no persiste como ResearchQuery</h3>
      {results.length===0 ? <div className="text-sm text-gray-500">Sin resultados aún. Prueba la búsqueda.</div> :
        <div className="space-y-2 mt-2">
          {results.map((r:any, idx:number)=>(
            <div key={idx} className="border rounded p-3 bg-gray-50">
              <div className="text-sm font-semibold">{r.title} {r.external_id && <span className="text-xs text-gray-500">· {r.external_id}</span>}</div>
              {r.description && <div className="text-xs text-gray-600">{r.description}</div>}
              <div className="text-xs text-gray-500 mt-1">{r.date || ""} {r.place?`· ${r.place}`:""} {r.record_type?`· ${r.record_type}`:""} · {r.provider}</div>
              <div className="text-xs mt-1"><span className="px-2 py-0.5 bg-yellow-100 border rounded">External Research Result</span> <span className="ml-2 text-amber-800">This result is not evidence.</span></div>
              {r.url && <a href={r.url} target="_blank" rel="noopener noreferrer" className="text-xs text-blue-600 underline mt-1 inline-block">Open external source</a>}
            </div>
          ))}
        </div>
      }
    </div>
  </div>
}
