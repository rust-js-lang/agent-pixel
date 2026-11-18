import { expect, test } from "bun:test";
import { fitsBoard } from "../core/src/lib.js";
import { Scene, workerCommand } from "../core/src/scene.js";

test("compiled bounds include edges and reject overflow", () => {
  expect(fitsBoard(0, 0, 480, 600)).toBe(true);
  expect(fitsBoard(479, 599, 1, 1)).toBe(true);
  expect(fitsBoard(480, 599, 1, 1)).toBe(false);
  expect(fitsBoard(0, 0, 0, 1)).toBe(false);
  expect(fitsBoard(2147483647, 0, 2, 2)).toBe(false);
});

test("the poster round-trips and duplicate identities are rejected", () => {
  const scene = Scene.poster();
  const restore = input => workerCommand(scene, "restore", JSON.stringify(input), "");
  const legacy = JSON.parse(JSON.stringify({ version: scene.version, layers: scene.layers }));
  expect(JSON.parse(restore(legacy)._0).scene).toEqual(legacy);
  expect(restore({ version: 1, layers: [scene.layers[0], scene.layers[0]] }))
    .toEqual({ TAG: "Err", _0: "Duplicate layer ID." });
});
