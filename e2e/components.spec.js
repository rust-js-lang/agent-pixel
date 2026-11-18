import { test, expect } from "@playwright/test";

test("reuse a library variant in another project from the editor", async ({ page }) => {
  await page.goto("/?project=canopy-ui");
  await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
  await page.webmcp.callTool("edit_layer", { id: "primary-button", fill: "#247a69" });

  await page.getByRole("link", { name: "Projects", exact: false }).click();
  await page.getByLabel("Project name").fill("Checkout");
  await page.getByRole("button", { name: /New project/ }).click();
  await page.getByText("Canopy UI components", { exact: true }).click();
  await page.getByLabel("Component variant").selectOption("primary-button");
  await page.getByLabel("Component X").fill("32");
  await page.getByLabel("Component Y").fill("80");
  await page.getByRole("button", { name: "Insert component" }).click();
  await expect(page.locator("#shapes").getByText("Continue", { exact: true })).toBeVisible();
  await expect(page.locator("#status")).toHaveText("Saved in this browser");
  const { scene } = await page.webmcp.callTool("read_design");
  expect(scene.layers).toHaveLength(2);
  expect(scene.layers[0]).toMatchObject({ kind: "rectangle", x: 32, y: 80, width: 130, height: 35, fill: "#247a69" });
  expect(scene.layers[1]).toMatchObject({ kind: "text", x: 68, y: 84, text: "Continue" });

  await page.reload();
  await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
  expect((await page.webmcp.callTool("read_design")).scene).toEqual(scene);
});

test("an agent discovers variants, inserts a copy, and rejects a partial placement", async ({ page }) => {
  await page.goto("/?design");
  await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
  const catalog = await page.webmcp.callTool("list_components");
  expect(catalog).toHaveLength(12);
  const variant = catalog.find(item => item.family === "Switch" && item.name === "On");
  expect(variant.id).toBe("on-switch");
  const insert = (x, y) => page.webmcp.callTool("insert_component", { id: variant.id, x, y });
  const first = await insert(30, 100);
  const second = await insert(30, 180);
  expect(first.inserted).toHaveLength(4);
  expect(second.inserted).toHaveLength(4);
  expect(second.scene.layers.map(layer => layer.id)).toEqual([...first.inserted, ...second.inserted]);
  expect(Math.min(...second.scene.layers.slice(4).map(layer => layer.y))).toBe(180);
  expect(await insert(470, 100)).toMatchObject({ isError: true });
  expect((await page.webmcp.callTool("read_design")).scene).toEqual(second.scene);
});
