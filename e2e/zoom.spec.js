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

test("canvas styles load from the bundled CSS Module with scoped class names", async ({ page }) => {
  await page.goto("/?project=orbit");
  const controls = page.getByRole("group", { name: "Canvas zoom" });
  await expect(controls).toHaveCSS("display", "flex");
  const classNames = await controls.evaluate(node => Array.from(node.classList));
  expect(classNames).toHaveLength(1);
  expect(classNames).not.toContain("zoom-tools");
  await page.addStyleTag({ content: ".zoom-tools { display: none; }" });
  await expect(controls).toBeVisible();
  await expect(page.locator("#canvas-scroll")).toHaveCSS("overflow", "auto");
});

for (const viewport of [{ width: 1100, height: 720 }, { width: 390, height: 820 }]) {
  test(`vertical wheel stays inside the canvas at ${viewport.width}px`, async ({ page }) => {
    await page.setViewportSize(viewport);
    await page.goto("/?project=current-reminders");
    await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
    const layerPanel = page.getByRole("complementary", { name: "Layers panel" });
    const inspector = page.getByRole("complementary", { name: "Inspector" });
    const controls = page.getByRole("group", { name: "Canvas zoom" });
    const zoom = controls.getByRole("button", { name: "Zoom in" });
    for (let i = 0; i < 6; i++) await zoom.click();
    const controlsOnScreen = [page.getByRole("banner"), layerPanel, inspector, controls];
    const positions = await Promise.all(controlsOnScreen.map(item => item.boundingBox()));
    const canvas = page.locator("#canvas-scroll");
    await canvas.hover();
    await canvas.evaluate(node => { node.scrollTop = 0; });
    await page.mouse.wheel(0, 140);
    await expect.poll(() => canvas.evaluate(node => node.scrollTop)).toBeGreaterThan(0);
    const belowTop = await canvas.evaluate(node => node.scrollTop);
    await page.mouse.wheel(0, -75);
    await expect.poll(() => canvas.evaluate(node => node.scrollTop)).toBeLessThan(belowTop);
    await canvas.evaluate(node => { node.scrollTop = node.scrollHeight; });
    const atBottom = await canvas.evaluate(node => node.scrollTop);
    await page.mouse.wheel(0, 140);
    await expect.poll(() => canvas.evaluate(node => node.scrollTop)).toBe(atBottom);
    expect(await page.evaluate(() => ({ y: window.scrollY, overflow: document.documentElement.scrollHeight - innerHeight })))
      .toEqual({ y: 0, overflow: 0 });
    expect(await Promise.all(controlsOnScreen.map(item => item.boundingBox()))).toEqual(positions);
  });
}
