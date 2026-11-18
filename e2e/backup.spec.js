import { test, expect } from "@playwright/test";

test("an agent restores every journey screen, link and linked component in a fresh browser", async ({ page, browser, baseURL }) => {
  const source = await page.context().newPage();
  await source.goto("/?project=canopy-ui");
  await expect(source.locator("#agent")).toHaveText("WebMCP tools ready");
  await source.webmcp.callTool("edit_layer", { id: "primary-button", fill: "#247a69" });

  await page.goto("/?project=current-reminders&screen=schedule");
  await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
  await page.webmcp.callTool("edit_layer", { id: "page-title", text: "Your weekday schedule" });
  const placed = await page.webmcp.callTool("insert_component", { id: "primary-button", x: 30, y: 465 });
  expect(placed.scene.instances).toHaveLength(1);
  await page.goto("/");
  await expect.poll(async () => (await page.webmcp.tools()).map(tool => tool.name)).toContain("export_project");
  const bundle = await page.webmcp.callTool("export_project", { id: "current-reminders" });
  expect(bundle.screens).toHaveLength(5);
  expect(bundle.library.find(item => item.id === "primary-button").layers[0].fill).toBe("#247a69");

  const recovery = await browser.newContext({ baseURL });
  try {
    const restored = await recovery.newPage();
    await restored.goto("/");
    await expect.poll(async () => (await restored.webmcp.tools()).map(tool => tool.name)).toContain("import_project");
    const before = await restored.webmcp.callTool("list_projects");
    const invalid = { ...bundle, library: [] };
    expect(await restored.webmcp.callTool("import_project", { bundle: invalid })).toMatchObject({ isError: true });
    expect(await restored.webmcp.callTool("list_projects")).toEqual(before);

    const project = await restored.webmcp.callTool("import_project", { bundle });
    expect(project.title).toBe("Current — Reminder journey");
    expect(await restored.webmcp.callTool("list_projects")).toContainEqual(project);
    await restored.locator(`a[href="?project=${project.id}"]`).click();
    const screens = restored.getByRole("navigation", { name: "Journey screens" });
    await expect(screens.getByRole("link", { name: "Set schedule" })).toBeVisible();
    await screens.getByRole("link", { name: "Set schedule" }).click();
    await expect(restored.locator("#agent")).toHaveText("WebMCP tools ready");
    const design = await restored.webmcp.callTool("read_design");
    expect(design.scene).toEqual(placed.scene);
    await expect(restored.locator("#shapes").getByText("Your weekday schedule")).toBeVisible();
    await screens.getByRole("link", { name: "Play prototype" }).click();
    await restored.getByRole("link", { name: "Review reminder", exact: true }).click();
    await expect(restored.getByRole("heading", { name: "Review" })).toBeVisible();
    await expect(restored).toHaveURL(new RegExp(`project=${project.id}&screen=review&preview`));
  } finally {
    await recovery.close();
  }
});

test("download and upload a project bundle from the dashboard", async ({ page }) => {
  await page.goto("/");
  await page.getByText("Project backup", { exact: true }).click();
  await page.getByLabel("Project to export").selectOption("orbit");
  await page.getByRole("button", { name: "Prepare backup" }).click();
  const [download] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("link", { name: "Download project bundle" }).click()
  ]);
  await page.getByLabel("Project bundle to restore").setInputFiles({
    name: "agent-pixel-project.json",
    mimeType: "application/json",
    buffer: Buffer.from(await Bun.file(await download.path()).arrayBuffer())
  });
  await page.getByRole("button", { name: "Restore project" }).click();
  await expect(page).toHaveURL(/\?project=[\da-f-]{36}$/);
  await expect(page.getByRole("heading", { name: "Orbit study" })).toBeVisible();
  await expect(page.locator("#shapes").getByRole("button")).toHaveCount(7);
});
