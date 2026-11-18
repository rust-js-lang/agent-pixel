import { test, expect } from "@playwright/test";

async function dashboardTools(page) {
  await page.goto("/");
  await expect.poll(async () => (await page.webmcp.tools()).map(tool => tool.name)).toContain("create_project");
}

async function editorTools(page, id) {
  await page.goto(`/?project=${id}`);
  await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
}

test("resize an artboard from the inspector and edit desktop coordinates", async ({ page }) => {
  await dashboardTools(page);
  const project = await page.webmcp.callTool("create_project", { title: "Desktop layout" });
  await editorTools(page, project.id);
  const before = await page.webmcp.callTool("read_design");
  expect(before.board).toEqual({ width: 480, height: 600 });
  await page.getByText("Artboard size", { exact: true }).click();
  const form = page.getByRole("form", { name: "Resize artboard" });
  await form.getByLabel("Artboard width").fill("1440");
  await form.getByLabel("Artboard height").fill("900");
  await form.getByRole("button", { name: "Resize artboard" }).click();
  await expect(page.locator("#art")).toHaveCSS("aspect-ratio", "1440 / 900");

  await page.getByRole("button", { name: /Rectangle \+/ }).click();
  const inspector = page.getByRole("complementary", { name: "Inspector" });
  await inspector.getByRole("spinbutton", { name: "x", exact: true }).fill("1250");
  await inspector.getByRole("spinbutton", { name: "x", exact: true }).press("Tab");
  await expect.poll(async () => (await page.webmcp.callTool("read_design")).scene.layers[0].x).toBe(1250);
  await expect(page.locator("#status")).toHaveText("Saved in this browser");
  await page.reload();
  await expect(page.locator("#art")).toHaveCSS("aspect-ratio", "1440 / 900");
  await expect(page.getByText("1440 × 900", { exact: true })).toBeVisible();
});

test("an agent resizes, edits past legacy bounds, and restores dimensions in a fresh browser", async ({ page, browser, baseURL }) => {
  await editorTools(page, "orbit");
  const start = await page.webmcp.callTool("read_design");
  const target = { width: 1280, height: 800 };
  const created = await page.webmcp.callTool("apply_edits", {
    revision: start.revision,
    edits: [
      { op: "resize_artboard", ...target },
      { op: "patch", layer: { kind: "rectangle", x: 1150, y: 700, width: 100, height: 80 } }
    ]
  });
  expect(created.board).toEqual(target);
  expect(created.scene.board).toEqual(target);
  const invalid = await page.webmcp.callTool("apply_edits", {
    revision: created.revision,
    edits: [
      { op: "patch", layer: { id: created.edited, fill: "#111111" } },
      { op: "resize_artboard", width: 390, height: 844 }
    ]
  });
  expect(invalid).toMatchObject({ isError: true });
  const unchanged = await page.webmcp.callTool("read_design");
  expect(unchanged.scene).toEqual(created.scene);
  expect(unchanged.revision).toBe(created.revision);
  const undone = await page.webmcp.callTool("undo_edit", { revision: created.revision });
  expect(undone.scene).toEqual(start.scene);
  const redone = await page.webmcp.callTool("redo_edit", { revision: undone.revision });
  expect(redone.scene).toEqual(created.scene);

  await dashboardTools(page);
  const bundle = await page.webmcp.callTool("export_project", { id: "orbit" });
  expect(bundle.screens[0].scene.board).toEqual(target);
  const fresh = await browser.newContext({ baseURL });
  try {
    const restoredPage = await fresh.newPage();
    await dashboardTools(restoredPage);
    const restored = await restoredPage.webmcp.callTool("import_project", { bundle });
    await editorTools(restoredPage, restored.id);
    const roundTrip = await restoredPage.webmcp.callTool("read_design");
    expect(roundTrip.scene).toEqual(created.scene);
    await expect(restoredPage.locator("#art")).toHaveCSS("aspect-ratio", "1280 / 800");
  } finally {
    await fresh.close();
  }
});
