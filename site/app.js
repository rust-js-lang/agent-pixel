const el = id => document.getElementById(id);
const status = message => { el("status").textContent = message; };
const worker = new Worker(new URL("./worker.js", import.meta.url), { type: "module" });
const pending = new Map();
const serverMode = new URLSearchParams(location.search).has("server");
const storageKey = "agent-pixel:poster:v1" + (serverMode ? ":server" : "");
let sequence = 0, scene, board, selected, revision = 0;
function request(command, input) {
  const seq = ++sequence;
  return new Promise((resolve, reject) => {
    pending.set(seq, { resolve, reject });
    worker.postMessage({ seq, command, input });
  });
}
worker.onmessage = ({ data }) => {
  const task = pending.get(data.seq);
  pending.delete(data.seq);
  if (data.error) task.reject(Error(data.error));
  else task.resolve(data);
};
worker.onerror = () => {
  for (const task of pending.values()) task.reject(Error("Document worker stopped. Reload the page."));
  pending.clear();
  status("Document worker stopped. Reload the page.");
};
const label = layer => layer.id.length === 36 ? layer.kind + " " + layer.id.slice(0, 4) : layer.id.replaceAll("-", " ");
const report = error => status(error.message);
async function edit(input) {
  const result = await request("edit", input);
  selected = result.edited;
  scene = result.scene;
  render();
  try {
    localStorage.setItem(storageKey, JSON.stringify({ scene, revision }));
    status(serverMode ? "Local draft saved · Save to server to publish this revision" : "Saved in this browser");
  } catch { status("Browser storage unavailable · Edits are in memory"); }
  return { board, scene, edited: selected };
}
function select(id) {
  selected = id;
  inspector();
  for (const item of el("layers").children) item.setAttribute("aria-pressed", item.dataset.id === id);
}
function inspector() {
  const layer = scene.layers.find(item => item.id === selected);
  if (!layer) return;
  el("name").textContent = label(layer);
  for (const field of el("fields").elements) field.value = layer[field.name];
  el("fill").value = el("hex").textContent = layer.fill;
}
function wireDrag(shape, layer) {
  shape.onpointerdown = e => {
    if (e.button !== 0) return;
    select(layer.id);
    shape.setPointerCapture(e.pointerId);
    const rect = el("art").getBoundingClientRect();
    const position = event => ({
      x: Math.min(board.width-layer.width, Math.max(0, Math.round(layer.x+(event.clientX-e.clientX)*board.width/rect.width))),
      y: Math.min(board.height-layer.height, Math.max(0, Math.round(layer.y+(event.clientY-e.clientY)*board.height/rect.height)))
    });
    shape.onpointermove = event => {
      const p = position(event);
      shape.style.left = p.x/board.width*100+"%";
      shape.style.top = p.y/board.height*100+"%";
    };
    shape.onpointerup = event => {
      shape.onpointermove = null;
      const p = position(event);
      if (p.x !== layer.x || p.y !== layer.y) edit({ id: layer.id, ...p }).catch(report);
    };
    shape.onpointercancel = render;
  };
}
function render() {
  el("shapes").replaceChildren();
  el("layers").replaceChildren();
  for (const layer of scene.layers) {
    const shape = document.createElement("button");
    shape.className = "shape";
    shape.setAttribute("aria-label", "Select " + label(layer));
    Object.assign(shape.style, { left: layer.x/board.width*100+"%", top: layer.y/board.height*100+"%",
      width: layer.width/board.width*100+"%", height: layer.height/board.height*100+"%",
      background: layer.fill, borderRadius: layer.kind === "ellipse" ? "50%" : "0" });
    shape.onclick = () => select(layer.id);
    wireDrag(shape, layer);
    el("shapes").append(shape);
    const item = document.createElement("button"), swatch = document.createElement("i");
    item.className = "layer";
    item.dataset.id = layer.id;
    swatch.style.background = layer.fill;
    swatch.style.borderRadius = shape.style.borderRadius;
    item.append(swatch, label(layer));
    item.onclick = () => select(layer.id);
    el("layers").prepend(item);
  }
  select(selected);
}
for (const name of ["x", "y", "width", "height"]) {
  const wrapper = document.createElement("label"), input = document.createElement("input");
  input.name = name;
  input.type = "number";
  input.setAttribute("aria-label", name);
  input.onchange = () => edit({ id: selected, [name]: input.valueAsNumber }).catch(error => { inspector(); report(error); });
  wrapper.append(name[0].toUpperCase(), input);
  el("fields").append(wrapper);
}
el("fields").onsubmit = e => e.preventDefault();
el("fill").onchange = e => edit({ id: selected, fill: e.target.value }).catch(report);
for (const [id, kind] of [["rect","rectangle"], ["ellipse","ellipse"]])
  el(id).onclick = () => edit({ kind }).catch(report);

async function registerTools() {
  const registry = document.modelContext ?? navigator.modelContext;
  if (!registry?.registerTool) { el("agent").textContent = "WebMCP unavailable in this browser"; return; }
  const properties = { id: { type: "string" }, kind: { enum: ["rectangle", "ellipse"], type: "string" },
    fill: { type: "string", pattern: "^#[0-9A-Fa-f]{6}$" } };
  for (const key of ["x","y","width","height"]) properties[key] = { type: "integer" };
  await registry.registerTool({ name: "read_design", description: "Read the artboard and its ordered layers.",
    inputSchema: { type: "object", properties: {}, additionalProperties: false }, annotations: { readOnlyHint: true },
    execute: async () => { const { scene, board } = await request("read"); return { scene, board }; } });
  await registry.registerTool({ name: "edit_layer", description: "Patch a layer by ID or omit ID to create a shape. Coordinates are artboard pixels.",
    inputSchema: { type: "object", properties, minProperties: 1, additionalProperties: false }, execute: edit });
  el("agent").textContent = "WebMCP tools ready";
}
async function start() {
  let saved, warning;
  try { saved = JSON.parse(localStorage.getItem(storageKey)); } catch { warning = "Browser draft unavailable"; }
  let result = await request("read");
  if (serverMode && !saved) {
    const response = await fetch("./api/document");
    if (!response.ok) throw Error("Document server unavailable");
    saved = await response.json();
  }
  if (saved?.scene) {
    try { result = await request("restore", saved.scene); } catch (error) { warning = error.message; }
  }
  revision = saved?.revision ?? 0;
  ({ scene, board } = result);
  selected = scene.layers.at(-1)?.id;
  render();
  status(warning ?? (serverMode ? "Server workspace · Revision " + revision : "Browser workspace · Ready"));
  if (serverMode) {
    const save = document.createElement("button");
    save.textContent = "Save to server";
    save.onclick = async () => {
      save.disabled = true;
      try {
        const submitted = scene;
        const response = await fetch("./api/document", { method: "PUT", headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ revision, scene }) });
        if (!response.ok) throw Error(response.status === 409 ? "Server changed · Local draft kept" : "Server save failed");
        revision = (await response.json()).revision;
        if (scene === submitted) localStorage.removeItem(storageKey);
        else localStorage.setItem(storageKey, JSON.stringify({ revision, scene }));
        status(scene === submitted ? "Saved server revision " + revision : "Newer edits remain in your local draft");
      } catch (error) { report(error); } finally { save.disabled = false; }
    };
    const reload = document.createElement("button");
    reload.textContent = "Load server copy";
    reload.onclick = async () => {
      if (!confirm("Replace the current draft with the saved server copy?")) return;
      try {
        const response = await fetch("./api/document");
        if (!response.ok) throw Error("Server unavailable");
        const saved = await response.json();
        if (!saved.scene) throw Error("No saved server copy yet");
        scene = (await request("restore", saved.scene)).scene;
        revision = saved.revision;
        selected = scene.layers.at(-1)?.id;
        localStorage.removeItem(storageKey);
        render();
        status("Loaded server revision " + revision);
      } catch (error) { report(error); }
    };
    el("server").append(save, reload);
  }
  await registerTools();
}
start().catch(report);
