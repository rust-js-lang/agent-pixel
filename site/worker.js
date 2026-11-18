import { Scene, workerCommand } from "../core/src/scene.js";

const scene = Scene.poster();

self.onmessage = ({ data: { seq, command, json } }) => {
  const result = workerCommand(scene, command, json, crypto.randomUUID());
  if (result.TAG === "Err") self.postMessage({ seq, error: result._0 });
  else self.postMessage({ seq, json: result._0 });
};
