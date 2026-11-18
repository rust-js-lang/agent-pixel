import { test, expect } from "@playwright/test";

test("agent batches are atomic, attributed, revision-checked and reversible", async ({ page }) => {
  await page.goto("/?design");
  await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
  const read = () => page.webmcp.callTool("read_design");
  const apply = (revision, edits) => page.webmcp.callTool("apply_edits", { revision, edits });
  const before = await read();
  const patches = [
    { op: "patch", layer: { kind: "rectangle", x: 20, y: 30, width: 200, height: 40 } },
    { op: "patch", layer: { kind: "text", x: 30, y: 30, width: 160, height: 30, text: "Checkout", fontSize: 20 } }
  ];

  expect(await apply(before.revision, [patches[0], { op: "delete", id: "missing" }])).toMatchObject({ isError: true });
  expect(await read()).toEqual(before);

  const committed = await apply(before.revision, patches);
  expect(committed.scene.layers).toHaveLength(2);
  expect(committed.history.undo).toEqual([{ label: "Edit layers", author: "agent" }]);
  await expect(page.getByLabel("Last edit")).toHaveText("Edit layers · Agent");
  expect(await apply(before.revision, patches)).toMatchObject({ isError: true });

  await page.getByRole("button", { name: "Undo", exact: true }).click();
  await expect(page.locator("#shapes button")).toHaveCount(0);
  const undone = await read();
  expect(undone.scene).toEqual(before.scene);
  expect(undone.revision).not.toBe(before.revision);
  const redone = await page.webmcp.callTool("redo_edit", { revision: undone.revision });
  expect(redone.scene).toEqual(committed.scene);

  const deletion = await apply(redone.revision, [{ op: "delete", id: committed.scene.layers[1].id }]);
  expect(deletion.scene.layers).toHaveLength(1);
  const restored = await page.webmcp.callTool("undo_edit", { revision: deletion.revision });
  expect(restored.scene).toEqual(committed.scene);
});

test("deleting a linked instance removes and restores the whole instance", async ({ page }) => {
  await page.goto("/?design");
  await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
  const placed = await page.webmcp.callTool("insert_component", { id: "primary-button", x: 40, y: 60 });
  await page.getByRole("button", { name: "Delete layer" }).click();
  await expect(page.locator("#shapes button")).toHaveCount(0);
  const empty = await page.webmcp.callTool("read_design");
  expect(empty.scene.instances ?? []).toEqual([]);
  expect(empty.history.undo.at(-1).author).toBe("person");
  await page.getByRole("button", { name: "Undo", exact: true }).click();
  await expect(page.locator("#shapes").getByText("Continue", { exact: true })).toBeVisible();
  expect((await page.webmcp.callTool("read_design")).scene).toEqual(placed.scene);
});
