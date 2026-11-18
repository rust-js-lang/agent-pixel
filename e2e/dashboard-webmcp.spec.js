import { test, expect } from "@playwright/test";

const dashboardTools = ["create_project", "export_project", "import_project", "list_projects", "open_project"];
const editorTools = ["apply_edits", "edit_layer", "export_unsaved_project", "get_save_status", "insert_component", "list_components", "read_design", "recover_checkpoint", "redo_edit", "retry_draft_save", "switch_component_variant", "undo_edit"];

async function expectTools(page, names) {
  await expect.poll(async () => (await page.webmcp.tools()).map(tool => tool.name).sort()).toEqual(names);
}

async function openProject(page, id) {
  await Promise.all([
    page.waitForURL("/?project=" + id),
    page.webmcp.callTool("open_project", { id }, { timeout: 5000 }).catch(error => {
      // Navigation can dispose the tool callback before Chromium returns its result.
      if (!error.message.includes("Execution context was destroyed")) throw error;
    })
  ]);
  await expectTools(page, editorTools);
}

test("an agent lists, creates, and opens a project, then edits its design", async ({ page }) => {
  await page.goto("/");
  await expectTools(page, dashboardTools);
  const list = () => page.webmcp.callTool("list_projects");
  const initial = await list();
  expect(initial).toEqual([
    { id: "orbit", title: "Orbit study" },
    { id: "current", title: "Current — Home energy" },
    { id: "canopy-ui", title: "Canopy UI — Design system" },
    { id: "current-reminders", title: "Current — Reminder journey" }
  ]);

  const project = await page.webmcp.callTool("create_project", { title: "Weather dashboard" });
  expect(project).toEqual({ id: expect.stringMatching(/^[a-f0-9-]{36}$/), title: "Weather dashboard" });
  expect(await list()).toEqual([...initial, project]);
  await expect(page.getByRole("link", { name: "Weather dashboard Empty artboard" })).toBeVisible();
  await expect(page).toHaveURL("/");

  await openProject(page, project.id);
  await expect(page.getByRole("heading", { name: "Weather dashboard" })).toBeVisible();
  expect((await page.webmcp.callTool("read_design")).scene.layers).toEqual([]);

  const layer = { kind: "text", text: "Weather today", x: 24, y: 24, width: 300, height: 50, fontSize: 32, fill: "#162d29" };
  const edited = await page.webmcp.callTool("edit_layer", layer);
  expect(edited.scene.layers).toEqual([{ ...layer, id: edited.edited }]);
  await expect(page.locator("#shapes").getByText("Weather today")).toBeVisible();

  await page.goto("/");
  await expectTools(page, dashboardTools);
  expect(await list()).toContainEqual(project);
  await expect(page.getByRole("link", { name: "Weather dashboard 1 layer" })).toBeVisible();
  await openProject(page, project.id);
  expect((await page.webmcp.callTool("read_design")).scene).toEqual(edited.scene);
});

test("invalid dashboard tool requests preserve the project list and route", async ({ page }) => {
  await page.goto("/");
  await expectTools(page, dashboardTools);
  const before = await page.webmcp.callTool("list_projects");
  expect(await page.webmcp.callTool("create_project", { title: "   " })).toEqual({
    isError: true, content: [{ type: "text", text: "Enter a name between 1 and 80 characters." }]
  });
  expect(await page.webmcp.callTool("open_project", { id: "missing" })).toEqual({
    isError: true, content: [{ type: "text", text: "Project not found in this browser." }]
  });
  await expect(page).toHaveURL("/");
  expect(await page.webmcp.callTool("list_projects")).toEqual(before);
});
