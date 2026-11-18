import { Scene, workerCommand } from "../core/src/scene.js";

const scene = Scene.poster();

self.onmessage = ({ data: { seq, command, input } }) => {
  const result = workerCommand(scene, command, JSON.stringify(input ?? null), crypto.randomUUID());
  if (result.TAG === "Err") self.postMessage({ seq, error: result._0 });
  else self.postMessage({ seq, ...JSON.parse(result._0) });
};
