import { test, expect } from "@playwright/test";

test("create, resize and recolor a rectangle, then reopen the draft", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("button", { name: /Rectangle \+/ }).click();
  const rectangle = page.getByRole("button", { name: /^Select rectangle / });
  await expect(rectangle).toHaveCount(1);
  const width = page.getByRole("spinbutton", { name: "width" });
  await width.fill("120");
  await width.press("Tab");
  await expect.poll(() => rectangle.evaluate(shape => shape.style.width)).toBe("25%");
  await page.locator("#fill").fill("#247a69");
  await expect(rectangle).toHaveCSS("background-color", "rgb(36, 122, 105)");
  await page.reload();
  await rectangle.click();
  await expect(width).toHaveValue("120");
  await expect(rectangle).toHaveCSS("background-color", "rgb(36, 122, 105)");
  await width.fill("481");
  await width.press("Tab");
  await expect(page.locator("#status")).toContainText("inside the artboard");
  await expect(width).toHaveValue("120");
});

test("dragging to the artboard edge clamps and persists the position", async ({ page }) => {
  await page.goto("/");
  const satellite = page.getByRole("button", { name: "Select satellite" });
  const start = await satellite.boundingBox();
  const board = await page.locator("#art").boundingBox();
  await page.mouse.move(start.x + start.width / 2, start.y + start.height / 2);
  await page.mouse.down();
  await page.mouse.move(board.x + board.width, board.y);
  await page.mouse.up();
  await expect(page.getByRole("spinbutton", { name: "x", exact: true })).toHaveValue("406");
  await page.reload();
  await expect(page.getByRole("spinbutton", { name: "y", exact: true })).toHaveValue("0");
});

test("a stale server save preserves the local draft until the user loads the server copy", async ({ browser, page }) => {
  const other = await browser.newContext({ baseURL: test.info().project.use.baseURL });
  try {
    const stale = await other.newPage();
    await Promise.all([page.goto("/?server"), stale.goto("/?server")]);
    await expect(stale.getByRole("button", { name: "Save to server" })).toBeVisible();
    await page.getByRole("button", { name: /Rectangle \+/ }).click();
    await expect(page.locator("#status")).toContainText("Local draft saved");
    await page.getByRole("button", { name: "Save to server" }).click();
    await expect(page.locator("#status")).toContainText("Saved server revision");
    await stale.getByRole("button", { name: /Ellipse \+/ }).click();
    await expect(stale.locator("#status")).toContainText("Local draft saved");
    await stale.getByRole("button", { name: "Save to server" }).click();
    await expect(stale.locator("#status")).toContainText("Server changed");
    await stale.reload();
    await expect(stale.getByRole("button", { name: /^Select ellipse / })).toHaveCount(1);
    stale.once("dialog", dialog => dialog.accept());
    await stale.getByRole("button", { name: "Load server copy" }).click();
    await expect(stale.getByRole("button", { name: /^Select rectangle / })).toHaveCount(1);
    await expect(stale.getByRole("button", { name: /^Select ellipse / })).toHaveCount(0);
  } finally {
    await other.close();
  }
});
