import { test, expect } from "@playwright/test";

test("open projects from the dashboard and keep their drafts separate", async ({ page }) => {
  await page.goto("/");
  await expect(page.getByRole("heading", { name: "Design projects" })).toBeVisible();
  const orbit = page.getByRole("link", { name: "Orbit study 7 layers" });
  const current = page.getByRole("link", { name: "Current — Home energy 50 layers" });
  await expect(page.getByRole("region", { name: "Projects" }).getByRole("link")).toHaveCount(2);
  await expect(orbit).toBeVisible();
  await current.click();
  await expect(page).toHaveURL(/\?project=current$/);
  await expect(page.getByRole("heading", { name: "Current — Home energy" })).toBeVisible();
  await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
  const currentDesign = await page.webmcp.callTool("read_design");
  expect(currentDesign.scene.layers).toHaveLength(50);
  await expect(page.locator("#shapes").getByText("Home energy", { exact: true })).toBeVisible();
  await page.webmcp.callTool("edit_layer", { kind: "rectangle", fill: "#247a69" });

  await page.getByRole("link", { name: "Projects", exact: false }).click();
  await expect(page.getByRole("link", { name: "Current — Home energy 51 layers" })).toBeVisible();
  await orbit.click();
  await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
  const orbitDesign = await page.webmcp.callTool("read_design");
  expect(orbitDesign.scene.layers).toHaveLength(7);

  await page.goto("/?design");
  await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
  expect((await page.webmcp.callTool("read_design")).scene.layers).toHaveLength(0);
  await page.webmcp.callTool("edit_layer", { kind: "ellipse" });
  await page.getByRole("link", { name: "Projects", exact: false }).click();
  await expect(page.getByRole("link", { name: "Web app mockup 1 layer" })).toBeVisible();
  await page.getByRole("link", { name: "Current — Home energy 51 layers" }).click();
  await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
  expect((await page.webmcp.callTool("read_design")).scene.layers).toHaveLength(51);
});

test("create a named project and reopen it on a narrow viewport", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 820 });
  await page.goto("/");
  await page.getByLabel("Project name").fill("Kitchen planner");
  await page.getByRole("button", { name: /New project/ }).click();
  await expect(page).toHaveURL(/\?project=[\da-f-]{36}$/);
  await expect(page.getByRole("link", { name: "Projects", exact: false })).toBeVisible();
  await page.getByRole("button", { name: /Ellipse \+/ }).click();
  await expect(page.locator("#shapes button")).toHaveCount(1);
  await page.getByRole("link", { name: "Projects", exact: false }).click();
  const card = page.getByRole("link", { name: "Kitchen planner 1 layer" });
  await expect(card).toBeVisible();
  await page.reload();
  await card.click();
  await expect(page.locator("#shapes button")).toHaveCount(1);
});

test("an unknown project offers a way back to the dashboard", async ({ page }) => {
  await page.goto("/?project=missing");
  await expect(page.getByRole("heading", { name: "Project unavailable" })).toBeVisible();
  await page.getByRole("link", { name: "Back to projects" }).click();
  await expect(page.getByRole("heading", { name: "Design projects" })).toBeVisible();
});
