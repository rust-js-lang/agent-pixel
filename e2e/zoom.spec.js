import { test, expect } from "@playwright/test";

test("canvas zoom scales the artboard and keeps editor controls fixed", async ({ page }) => {
  await page.goto("/?project=orbit");
  const art = page.locator("#art");
  const zoom = page.getByRole("group", { name: "Canvas zoom" });
  const original = await art.boundingBox();
  const toolbar = page.getByRole("button", { name: /Rectangle \+/ });
  const toolSize = await toolbar.boundingBox();
  await zoom.getByRole("button", { name: "Zoom in" }).click();
  await expect(zoom.getByRole("button", { name: "Reset canvas zoom, current 125%" })).toBeVisible();
  expect((await art.boundingBox()).width).toBeCloseTo(original.width * 1.25, 0);
  expect((await toolbar.boundingBox()).width).toBeCloseTo(toolSize.width, 0);
  await zoom.getByRole("button", { name: "Zoom out" }).click();
  expect((await art.boundingBox()).width).toBeCloseTo(original.width, 0);

  await page.keyboard.press("Control+-");
  await expect(zoom.getByRole("button", { name: "Reset canvas zoom, current 75%" })).toBeVisible();
  await page.keyboard.press("Control+0");
  await expect(zoom.getByRole("button", { name: "Reset canvas zoom, current 100%" })).toBeVisible();
  expect((await art.boundingBox()).width).toBeCloseTo(original.width, 0);
});

test("pinch-style wheel zoom stays in the canvas, and the workspace scrolls at high zoom", async ({ page }) => {
  await page.setViewportSize({ width: 950, height: 720 });
  await page.goto("/?project=orbit");
  await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
  const art = page.locator("#art");
  const canvas = page.locator("#canvas-scroll");
  const start = await art.boundingBox();
  const browserScale = await page.evaluate(() => window.visualViewport.scale);
  await page.mouse.move(start.x + start.width / 2, start.y + start.height / 2);
  await page.keyboard.down("Control");
  await page.mouse.wheel(0, -280);
  await page.keyboard.up("Control");
  await expect.poll(async () => (await art.boundingBox()).width).toBeGreaterThan(start.width * 2);
  expect(await page.evaluate(() => window.visualViewport.scale)).toBe(browserScale);
  expect(await canvas.evaluate(node => node.scrollWidth > node.clientWidth)).toBe(true);

  const oldZoom = await art.boundingBox();
  const before = await canvas.evaluate(node => node.scrollTop);
  await page.mouse.wheel(0, 200);
  await expect.poll(() => canvas.evaluate(node => node.scrollTop)).toBeGreaterThan(before);
  expect((await art.boundingBox()).width).toBe(oldZoom.width);
  await page.getByRole("button", { name: /Reset canvas zoom, current/ }).click();
  await expect(page.getByRole("button", { name: "Reset canvas zoom, current 100%" })).toBeVisible();
});
