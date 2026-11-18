import { test, expect } from "@playwright/test";

test("React Router keeps navigation in one document and scopes WebMCP to the active view", async ({ page }) => {
  await page.goto("/");
  const names = async () => (await page.webmcp.tools()).map(tool => tool.name);
  await expect.poll(names).toContain("open_project");
  await page.evaluate(() => { globalThis.navigationSentinel = "same document"; });
  await page.getByRole("link", { name: /Current — Reminder journey/ }).click();
  await expect(page).toHaveURL(/\?project=current-reminders$/);
  await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
  expect(await page.evaluate(() => globalThis.navigationSentinel)).toBe("same document");
  expect(await names()).not.toContain("open_project");

  await page.getByRole("navigation", { name: "Journey screens" }).getByRole("link", { name: "Set schedule" }).click();
  await expect(page).toHaveURL(/screen=schedule$/);
  await expect(page.locator("#shapes").getByText("Set your schedule", { exact: true })).toBeVisible();
  expect(await page.evaluate(() => globalThis.navigationSentinel)).toBe("same document");
  expect((await page.webmcp.callTool("list_screens")).activeScreenId).toBe("schedule");

  await page.goBack();
  await expect(page).toHaveURL(/\?project=current-reminders$/);
  await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
  expect((await page.webmcp.callTool("list_screens")).activeScreenId).toBe("overview");
  await page.goForward();
  await expect(page.locator("#shapes").getByText("Set your schedule", { exact: true })).toBeVisible();
  await page.getByRole("link", { name: "← Projects" }).click();
  await expect.poll(names).toContain("open_project");
  expect(await names()).not.toContain("read_design");
  expect(await page.evaluate(() => globalThis.navigationSentinel)).toBe("same document");
});

test("query routes and links stay under a GitHub Pages project prefix", async ({ page, baseURL }) => {
  await page.route("**/agent-pixel/**", async route => {
    const url = new URL(route.request().url());
    const response = await route.fetch({ url: new URL(url.pathname.slice("/agent-pixel".length) + url.search, baseURL).href });
    await route.fulfill({ response });
  });
  await page.goto("/agent-pixel/?project=orbit");
  await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
  await page.getByRole("link", { name: "← Projects" }).click();
  await expect(page).toHaveURL(/\/agent-pixel\/$/);
  await expect(page.getByRole("heading", { name: "Design projects" })).toBeVisible();
  await page.getByRole("link", { name: /Orbit study/ }).click();
  await expect(page).toHaveURL(/\/agent-pixel\/\?project=orbit$/);
  await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
});
