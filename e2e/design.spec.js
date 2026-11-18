import { test, expect } from "@playwright/test";

test("an agent designs a task dashboard from a blank artboard", async ({ page }) => {
  await page.goto("/?design");
  await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
  const read = () => page.webmcp.callTool("read_design");
  const place = layer => page.webmcp.callTool("edit_layer", layer);
  const blank = await read();
  expect(blank.board).toEqual({ width: 480, height: 600 });
  expect(blank.scene.layers).toEqual([]);

  const rectangle = (x, y, width, height, fill) => ({ kind: "rectangle", x, y, width, height, fill });
  const text = (text, x, y, width, height, fontSize, fill) => ({
    kind: "text", text, x, y, width, height, fontSize, fill
  });
  const layers = [
    rectangle(0, 0, 480, 600, "#f6f3ed"),
    rectangle(0, 0, 480, 146, "#202b29"),
    text("FIELD NOTES", 24, 24, 210, 20, 13, "#b7d7c2"),
    text("Your tasks", 24, 59, 380, 52, 38, "#ffffff"),
    rectangle(24, 166, 432, 88, "#d9e8d7"),
    text("3 tasks in progress", 42, 185, 370, 36, 25, "#202b29"),
    text("TODAY", 24, 276, 120, 26, 16, "#47544e"),
    rectangle(24, 316, 432, 72, "#ffffff"),
    rectangle(24, 402, 432, 72, "#ffffff"),
    text("Review onboarding flow", 42, 328, 378, 44, 21, "#202b29"),
    text("Draft new settings screen", 42, 414, 378, 44, 21, "#202b29"),
    rectangle(24, 510, 432, 58, "#286249"),
    text("+  New task", 42, 517, 365, 42, 23, "#ffffff")
  ];
  const ids = [];
  for (const layer of layers) {
    const result = await place(layer);
    ids.push(result.edited);
    expect(result.scene.layers.at(-1)).toEqual({ ...layer, id: result.edited });
  }
  await expect(page.locator("#shapes button")).toHaveCount(layers.length);
  await expect(page.locator("#shapes").getByText("Your tasks", { exact: true })).toBeVisible();
  await expect(page.locator("#shapes").getByText("Review onboarding flow", { exact: true })).toBeVisible();

  const id = ids[3];
  const updated = await place({ id, text: "Today’s tasks" });
  await expect(page.locator("#shapes").getByText("Today’s tasks", { exact: true })).toBeVisible();
  await expect(place({ id, fontSize: 97 })).rejects.toThrow();
  expect(await read()).toEqual({ board: updated.board, scene: updated.scene });

  await page.reload();
  await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
  expect((await read()).scene).toEqual(updated.scene);
  await expect(page.locator("#shapes").getByText("Today’s tasks", { exact: true })).toBeVisible();

  await page.goto("/");
  await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
  expect((await read()).scene.layers.map(layer => layer.id)).toContain("satellite");
});
