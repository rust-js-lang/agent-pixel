// Browser transports. Editor behavior and the React tree live in frontend/src.
let worker, workerError, sequence = 0;
const pending = new Map();

export function requestWorker(command, json) {
  if (!worker) {
    worker = new Worker(new URL("./worker.js", import.meta.url), { type: "module" });
    worker.onmessage = ({ data }) => {
      const task = pending.get(data.seq);
      pending.delete(data.seq);
      if (data.error) task.reject(new Error(data.error));
      else task.resolve(data.json);
    };
    worker.onerror = () => {
      workerError = new Error("Document worker stopped. Reload the page.");
      for (const task of pending.values()) task.reject(workerError);
      pending.clear();
    };
  }
  if (workerError) return Promise.reject(workerError);
  const seq = ++sequence;
  return new Promise((resolve, reject) => {
    pending.set(seq, { resolve, reject });
    worker.postMessage({ seq, command, json });
  });
}

export function workspaceQuery(name) {
  return new URLSearchParams(location.search).has(name);
}

export function loadDraft(key) { return localStorage.getItem(key) ?? undefined; }
export function saveDraft(key, body) {
  if (body === undefined) localStorage.removeItem(key);
  else localStorage.setItem(key, body);
}

export async function documentRequest(body) {
  const response = await fetch("./api/document", body === undefined ? undefined : {
    method: "PUT", headers: { "Content-Type": "application/json" }, body
  });
  return { status: response.status, body: await response.text() };
}

export async function registerTools(schema, read, edit) {
  const registry = document.modelContext ?? navigator.modelContext;
  if (!registry?.registerTool) return false;
  const [reader, writer] = JSON.parse(schema);
  async function jsonResult(result) {
    const settled = await result;
    if (settled.TAG === "Err") throw new Error(settled._0);
    return JSON.parse(settled._0);
  }
  await registry.registerTool({ ...reader, execute: () => jsonResult(read()) });
  await registry.registerTool({ ...writer, execute: input => jsonResult(edit(JSON.stringify(input))) });
  return true;
}
