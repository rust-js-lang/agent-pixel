import { test, expect } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await page.goto("/?project=orbit");
  await expect.poll(async () => (await page.webmcp.tools()).map(tool => tool.name).sort())
    .toEqual(["apply_edits", "edit_layer", "get_save_status", "insert_component", "list_components", "read_design", "recover_checkpoint", "redo_edit", "retry_draft_save", "switch_component_variant", "undo_edit"]);
});

test("an agent creates a layer and reads back a human's edit after reload", async ({ page }) => {
  const design = await page.webmcp.callTool("read_design");
  expect(design.board).toEqual({ width: 480, height: 600 });
  const layer = { kind: "ellipse", x: 30, y: 34, width: 57, height: 63, fill: "#aabbcc" };
  const created = await page.webmcp.callTool("edit_layer", layer);
  expect(created.scene.layers.at(-1)).toEqual({ ...layer, id: created.edited });
  const shape = page.getByRole("button", { name: /^Select ellipse / });
  await expect(shape).toHaveCSS("background-color", "rgb(170, 187, 204)");
  const x = page.getByRole("spinbutton", { name: "x", exact: true });
  await x.fill("71");
  await x.press("Tab");
  await expect.poll(async () => (await page.webmcp.callTool("read_design")).scene.layers.at(-1).x).toBe(71);
  await page.reload();
  await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
  const restored = await page.webmcp.callTool("read_design");
  expect(restored.scene.layers.at(-1)).toEqual({ ...layer, id: created.edited, x: 71 });
});

test("a patch targets its ID and rejected edits leave the entire design intact", async ({ page }) => {
  const before = await page.webmcp.callTool("read_design");
  const id = "satellite";
  const edited = await page.webmcp.callTool("edit_layer", { id, x: 222 });
  const expected = before.scene.layers.map(layer => layer.id === id ? { ...layer, x: 222 } : layer);
  expect(edited.scene.layers).toEqual(expected);
  await expect(page.getByRole("spinbutton", { name: "x", exact: true })).toHaveValue("222");
  for (const patch of [{ id, x: -1 }, { id: "missing-layer", x: 30 }]) {
    await expect(page.webmcp.callTool("edit_layer", patch)).rejects.toThrow();
    expect((await page.webmcp.callTool("read_design")).scene.layers).toEqual(expected);
  }
});
