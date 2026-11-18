import { expect, test } from "bun:test";
import { fitsBoard } from "../core/src/lib.js";
import { Scene, Session } from "../core/src/scene.js";

test("compiled bounds include edges and reject overflow", () => {
  expect(fitsBoard(0, 0, 480, 600)).toBe(true);
  expect(fitsBoard(479, 599, 1, 1)).toBe(true);
  expect(fitsBoard(480, 599, 1, 1)).toBe(false);
  expect(fitsBoard(0, 0, 0, 1)).toBe(false);
  expect(fitsBoard(2147483647, 0, 2, 2)).toBe(false);
});

test("the poster round-trips and duplicate identities are rejected", () => {
  const scene = Scene.poster();
  const session = Session.new(scene);
  const restore = input => Session.command(session, "restore", JSON.stringify(input), "r1", "person");
  const legacy = JSON.parse(JSON.stringify({ version: scene.version, layers: scene.layers }));
  expect(JSON.parse(restore(legacy)._0).scene).toEqual(legacy);
  expect(restore({ version: 1, layers: [scene.layers[0], scene.layers[0]] }))
    .toEqual({ TAG: "Err", _0: "Duplicate layer ID." });
});

test("compiled history rejects partial edits and reverses a batch in one step", async () => {
  const fixture = await Bun.file(new URL("../core/tests/fixtures/history.json", import.meta.url)).json();
  const session = Session.new(Scene.blank());
  const run = (command, input, revision) => Session.command(session, command, JSON.stringify(input), revision, "agent");
  const initial = JSON.parse(run("restore", fixture.initial, "r1")._0);
  expect(run("batch", fixture.invalid, "r2").TAG).toBe("Err");
  expect(JSON.parse(run("read", null, "ignored")._0)).toEqual(initial);
  const edited = JSON.parse(run("batch", fixture.valid, "r3")._0);
  expect(edited.scene.layers).toHaveLength(2);
  expect(edited.history.undo).toEqual([{ label: "Edit layers", author: "agent" }]);
  const undone = JSON.parse(run("undo", "r3", "r4")._0);
  expect(undone.scene).toEqual(initial.scene);
  expect(undone.history.redo).toHaveLength(1);
  expect(run("redo", "r3", "stale").TAG).toBe("Err");
  expect(JSON.parse(run("redo", "r4", "r5")._0).scene).toEqual(edited.scene);
});
