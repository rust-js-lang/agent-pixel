import { test, expect } from "@playwright/test";

test("the workspace has a file and asset rail, a fixed inspector, and floating canvas tools", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 780 });
  await page.goto("/?project=orbit");
  await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
  const rail = page.getByRole("navigation", { name: "Workspace panels" });
  const file = rail.getByRole("button", { name: "File" });
  const assets = rail.getByRole("button", { name: "Assets" });
  const layers = page.getByRole("complementary", { name: "Layers panel" });
  const inspector = page.getByRole("complementary", { name: "Inspector" });
  const canvas = page.getByRole("region", { name: "Canvas" });
  const toolbar = page.getByRole("group", { name: "Design tools" });
  const zoom = page.getByRole("group", { name: "Canvas zoom" });

  await expect(file).toHaveAttribute("aria-pressed", "true");
  await expect(layers.getByRole("list", { name: "Layer hierarchy" })).toBeVisible();
  const [railBox, left, art, right, tools, scale] = await Promise.all(
    [rail, layers, canvas, inspector, toolbar, zoom].map(item => item.boundingBox())
  );
  expect(railBox.x).toBeLessThan(left.x);
  expect(left.x).toBeLessThan(art.x);
  expect(art.x).toBeLessThan(right.x);
  expect(tools.x + tools.width / 2).toBeCloseTo(art.x + art.width / 2, 0);
  expect(art.y + art.height - tools.y - tools.height).toBeLessThan(24);
  expect(scale.y - art.y).toBeLessThan(20);

  await assets.click();
  await expect(assets).toHaveAttribute("aria-pressed", "true");
  const panel = page.getByRole("complementary", { name: "Assets panel" });
  await expect(panel.getByLabel("Place image")).toBeVisible();
  await expect(panel.getByText("Canopy UI components", { exact: true })).toBeVisible();
  expect(await inspector.boundingBox()).toEqual(right);
  expect(await page.webmcp.callTool("show_workspace_panel", { panel: "file" })).toEqual({ panel: "file" });
  await expect(layers).toBeVisible();
});

test("the square artboard cursor uses a hotspot at the arrow tip and stays off the panels", async ({ page }) => {
  await page.goto("/?project=orbit");
  const art = page.locator("#art");
  const cursor = await art.evaluate(node => getComputedStyle(node).cursor);
  expect(cursor).toContain("data:image/svg+xml");
  expect(cursor).toMatch(/4 4, default$/);
  const size = await art.evaluate(async node => {
    const source = getComputedStyle(node).cursor.split('"')[1];
    const image = new Image();
    image.src = source;
    await image.decode();
    return [image.naturalWidth, image.naturalHeight];
  });
  expect(size).toEqual([24, 24]);
  await expect(page.getByRole("button", { name: "Select satellite", exact: true })).toHaveCSS("cursor", cursor);
  await expect(page.getByRole("button", { name: "File", exact: true })).toHaveCSS("cursor", "pointer");
});
