import { useEffect, useState } from "react";
import { Link, useSearchParams } from "react-router-dom";
import { api } from "../api/client";

export default function FamilySearchAuthCallback(){
  const [params]=useSearchParams();
  const [status,setStatus]=useState<string>("Verificando…");
  useEffect(()=>{
    const code = params.get("code");
    const err = params.get("familysearch_error") || params.get("error");
    if(err){
      setStatus(`Error: ${err} - ${params.get("familysearch_error_description")||params.get("error_description")||""}`);
      return;
    }
    if(params.get("familysearch")==="connected"){
      setStatus("Conectado con FamilySearch ✅");
      return;
    }
    if(code){
      setStatus("Código recibido, procesando en servidor… (si ves esta página, el backend ya debería haber redirigido)");
      return;
    }
    // Poll status
    api.getFamilySearchAuthStatus().then(s=>{
      if(s.connected) setStatus("Conectado con FamilySearch ✅");
      else setStatus("No conectado — revisa configuración");
    }).catch(()=> setStatus("No se pudo verificar estado"));
  },[params]);
  return <div className="p-6 space-y-4">
    <h1 className="text-xl font-bold">FamilySearch Auth Callback</h1>
    <div className="border rounded p-4 bg-white">{status}</div>
    <div className="flex gap-2">
      <Link to="/familysearch" className="px-3 py-1 bg-blue-600 text-white rounded">Ir a búsqueda global</Link>
      <Link to="/trees" className="px-3 py-1 border rounded">Ver árboles</Link>
    </div>
  </div>
}
