import { board, initial, restore, validate } from "./scene.js";
let scene = initial;

function edit(input) {
  const { id, ...fields } = input;
  if (!Object.keys(fields).length || Object.keys(fields).some(key => !["kind","x","y","width","height","fill"].includes(key)))
    throw Error("Specify shape properties.");
  const index = scene.layers.findIndex(layer => layer.id === id);
  if (id !== undefined && index < 0) throw Error("Unknown layer.");
  const isNew = id === undefined;
  if (isNew && scene.layers.length >= 1000) throw Error("Layer limit reached.");
  const layer = isNew ? { id: crypto.randomUUID(), kind: "rectangle", x: 40, y: 40, width: 80, height: 80, fill: "#ff5636", ...fields }
    : { ...scene.layers[index], ...fields };
  validate(layer);
  if (isNew) scene.layers.push(layer);
  else scene.layers[index] = layer;
  return layer.id;
}

self.onmessage = ({ data: { seq, command, input } }) => {
  try {
    let edited;
    switch (command) {
      case "restore": scene = restore(input); break;
      case "edit": edited = edit(input); break;
      case "read": break;
      default: throw Error("Unknown command.");
    }
    self.postMessage({ seq, scene, board, edited });
  } catch (error) {
    self.postMessage({ seq, error: error.message });
  }
};
