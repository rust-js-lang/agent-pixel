import { BOARD_WIDTH, BOARD_HEIGHT, fitsBoard } from "../core/src/lib.js";

export const board = { width: BOARD_WIDTH, height: BOARD_HEIGHT };
const ink = "#20231f", red = "#ff5636", paper = "#f5edda";
const orbit = (id, x, y, size, fill) => ({ id, kind: "ellipse", x, y, width: size, height: size, fill });
export const initial = { version: 1, layers: [
  orbit("outer-disc", 44, 68, 392, ink),
  orbit("orange-disc", 64, 88, 352, red),
  orbit("middle-disc", 84, 108, 312, ink),
  orbit("inner-disc", 104, 128, 272, red),
  orbit("core", 124, 148, 232, ink),
  orbit("moon", 150, 175, 178, paper),
  orbit("satellite", 305, 312, 74, red)
] };

export function validate(layer) {
  if (!["ellipse", "rectangle"].includes(layer.kind) || !/^#[0-9a-f]{6}$/i.test(layer.fill)
      || typeof layer.fill !== "string") throw Error("Choose a shape and a six-digit hex fill.");
  if (![layer.x, layer.y, layer.width, layer.height].every(Number.isSafeInteger)
      || !fitsBoard(layer.x, layer.y, layer.width, layer.height)) throw Error("Keep the entire shape inside the artboard.");
  if (typeof layer.id !== "string" || !/^[a-z0-9-]{1,64}$/.test(layer.id)) throw Error("Invalid layer ID.");
}

export function restore(scene) {
  if (scene?.version !== 1 || !Array.isArray(scene.layers) || scene.layers.length > 1000)
    throw Error("Unsupported document.");
  scene.layers.forEach(validate);
  if (new Set(scene.layers.map(layer => layer.id)).size !== scene.layers.length) throw Error("Duplicate layer ID.");
  return scene;
}
