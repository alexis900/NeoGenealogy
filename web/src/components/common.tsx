export function Loading({msg}:{msg?:string}){ return <div className="p-8 text-center text-gray-600">{msg||"Loading…"}</div> }
export function ErrorState({msg, onRetry}:{msg:string; onRetry?:()=>void}){
  return <div className="p-8 text-center"><p className="text-red-600 mb-2">{msg}</p>{onRetry&&<button onClick={onRetry} className="px-4 py-1 bg-gray-800 text-white rounded">Retry</button>}</div>
}
export function Empty({msg}:{msg:string}){ return <div className="p-8 text-center text-gray-500">{msg}</div> }
export function Pagination({limit, offset, total, onChange}:{limit:number;offset:number;total:number;onChange:(o:number)=>void}){
  const pages=Math.ceil(total/limit);
  const cur=Math.floor(offset/limit);
  return <div className="flex gap-2 items-center py-2">
    <button disabled={offset===0} onClick={()=>onChange(Math.max(0,offset-limit))} className="px-2 py-1 border rounded disabled:opacity-50">Prev</button>
    <span className="text-sm text-gray-600">Page {cur+1} / {pages||1} ({total} items)</span>
    <button disabled={offset+limit>=total} onClick={()=>onChange(offset+limit)} className="px-2 py-1 border rounded disabled:opacity-50">Next</button>
  </div>
}
