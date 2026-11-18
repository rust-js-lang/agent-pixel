import { expect, test } from "bun:test";
import { fitsBoard } from "../core/src/lib.js";
import { initial, restore } from "./scene.js";

test("compiled bounds include edges and reject overflow", () => {
  expect(fitsBoard(0, 0, 480, 600)).toBe(true);
  expect(fitsBoard(479, 599, 1, 1)).toBe(true);
  expect(fitsBoard(480, 599, 1, 1)).toBe(false);
  expect(fitsBoard(0, 0, 0, 1)).toBe(false);
  expect(fitsBoard(2147483647, 0, 2, 2)).toBe(false);
});

test("the poster round-trips and duplicate identities are rejected", () => {
  expect(restore(JSON.parse(JSON.stringify(initial)))).toEqual(initial);
  expect(() => restore({version: 1, layers: [initial.layers[0], initial.layers[0]]})).toThrow("Duplicate");
});
