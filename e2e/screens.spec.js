import { test, expect } from "@playwright/test";

async function editor(page, path) {
  await page.goto(path);
  await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
}

async function list(page) {
  return page.webmcp.callTool("list_screens");
}

async function edit(page, operation, revision) {
  return page.webmcp.callTool("edit_screens", { edit: operation, revision: revision ?? (await list(page)).revision });
}

async function createProject(page, title) {
  await page.goto("/");
  await expect.poll(async () => (await page.webmcp.tools()).map(tool => tool.name)).toContain("create_project");
  return page.webmcp.callTool("create_project", { title });
}

test("a designer adds, renames, duplicates, reorders, and removes a separate screen", async ({ page }) => {
  const project = await createProject(page, "New product");
  await editor(page, "?project=" + project.id);
  await page.getByText("Manage screens", { exact: true }).click();
  await page.getByLabel("New screen name").fill("Details");
  await page.getByRole("button", { name: "Add screen", exact: true }).click();
  await expect(page).toHaveURL(/&screen=[\da-f-]{36}$/);
  await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
  const details = (await list(page)).activeScreenId;
  await page.getByText("Manage screens", { exact: true }).click();
  await page.getByRole("form", { name: "Rename screen" }).getByLabel("Screen name").fill("Account details");
  await page.getByRole("button", { name: "Rename screen", exact: true }).click();
  const navigation = page.getByRole("navigation", { name: "Screens", exact: true });
  await expect(navigation.getByRole("link", { name: "Account details", exact: true })).toBeVisible();

  await page.getByRole("button", { name: /Rectangle \+/ }).click();
  await expect(page.locator("#status")).toHaveText("Saved in this browser");
  await page.getByRole("button", { name: "Duplicate screen", exact: true }).click();
  await expect(page).not.toHaveURL(new RegExp(`screen=${details}$`));
  await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
  const copy = (await list(page)).activeScreenId;
  expect(copy).not.toBe(details);
  expect((await page.webmcp.callTool("read_design")).scene.layers).toHaveLength(1);

  await page.getByText("Manage screens", { exact: true }).click();
  await page.getByRole("button", { name: "Move up", exact: true }).click();
  await expect.poll(async () => (await list(page)).screens.map(screen => screen.id)).toEqual(["main", copy, details]);
  page.once("dialog", dialog => dialog.accept());
  await page.getByRole("button", { name: "Remove screen", exact: true }).click();
  await expect(page).toHaveURL(new RegExp(`screen=${details}$`));
  await expect(navigation.getByRole("link", { name: "Account details copy", exact: true })).toHaveCount(0);
  await page.goto(`/?project=${project.id}&screen=${copy}`);
  await expect(page.getByText("This screen is no longer in the project.")).toBeVisible();
});

test("WebMCP preserves screen identity, copies committed artboards, and exports a reordered project", async ({ page, context, browser, baseURL }) => {
  const project = await createProject(page, "Agent product");
  const url = `/?project=${project.id}`;
  await editor(page, url);
  await page.webmcp.callTool("edit_layer", { kind: "text", text: "Main scene", fontSize: 20, fill: "#242423" });
  const staleTab = await context.newPage();
  await editor(staleTab, url);
  const baseline = await list(staleTab);
  const checkout = await edit(page, { op: "create", title: "Checkout" });
  const checkoutId = checkout.screenId;
  expect(await edit(staleTab, { op: "create", title: "Superseded" }, baseline.revision)).toMatchObject({ isError: true });

  await Promise.all([
    page.waitForURL(url + "&screen=" + checkoutId),
    page.webmcp.callTool("open_screen", { id: checkoutId }).catch(error => {
      if (!error.message.includes("Execution context was destroyed")) throw error;
    })
  ]);
  await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
  const blank = await page.webmcp.callTool("read_design");
  await page.webmcp.callTool("apply_edits", {
    revision: blank.revision,
    edits: [
      { op: "resize_artboard", width: 1440, height: 900 },
      { op: "patch", layer: { kind: "text", text: "Checkout scene", fontSize: 20, x: 1200, y: 30 } }
    ]
  });
  const duplicated = await edit(page, { op: "duplicate", id: checkoutId, title: "Confirmation" });
  const copyId = duplicated.screenId;
  await edit(page, { op: "reorder", id: copyId, index: 0 });
  await editor(page, url);
  expect((await list(page)).activeScreenId).toBe(copyId);
  expect((await page.webmcp.callTool("read_design")).scene).toMatchObject({
    board: { width: 1440, height: 900 },
    layers: [expect.objectContaining({ text: "Checkout scene" })]
  });
  await editor(page, url + "&screen=main");
  await expect(page.locator("#shapes").getByText("Main scene", { exact: true })).toBeVisible();

  await page.goto("/");
  await expect.poll(async () => (await page.webmcp.tools()).map(tool => tool.name)).toContain("export_project");
  const bundle = await page.webmcp.callTool("export_project", { id: project.id });
  expect(bundle.screens.map(screen => screen.info.id)).toEqual([copyId, "main", checkoutId]);
  const fresh = await browser.newContext({ baseURL });
  try {
    const restored = await fresh.newPage();
    await restored.goto("/");
    await expect.poll(async () => (await restored.webmcp.tools()).map(tool => tool.name)).toContain("import_project");
    const imported = await restored.webmcp.callTool("import_project", { bundle });
    await editor(restored, `/?project=${imported.id}&screen=main`);
    await expect(restored.locator("#shapes").getByText("Main scene", { exact: true })).toBeVisible();
  } finally {
    await fresh.close();
  }
});

test("an interrupted duplicate writes neither the new screen nor its draft", async ({ page }) => {
  await editor(page, "/?project=orbit");
  const before = await list(page);
  await page.evaluate(() => {
    const original = IDBObjectStore.prototype.add;
    IDBObjectStore.prototype.add = function (...args) {
      const request = original.apply(this, args);
      if (this.name === "drafts") {
        IDBObjectStore.prototype.add = original;
        this.transaction.abort();
      }
      return request;
    };
  });
  expect(await edit(page, { op: "duplicate", id: "main", title: "Interrupted copy" })).toMatchObject({ isError: true });
  await editor(page, "/?project=orbit");
  expect(await list(page)).toEqual(before);
  const retried = await edit(page, { op: "duplicate", id: "main", title: "Completed copy" });
  expect(retried.screens).toHaveLength(2);
});

test("a stale tab saves edits as a recovery project after another tab removes its screen", async ({ page, context }) => {
  await editor(page, "/?project=orbit");
  const created = await edit(page, { op: "create", title: "Temporary screen" });
  const stale = await context.newPage();
  await stale.addInitScript(() => {
    window.addEventListener("storage", event => {
      if (event.key === "agent-pixel:screens-changed") event.stopImmediatePropagation();
    });
  });
  await editor(stale, "/?project=orbit&screen=" + created.screenId);
  const initial = await stale.webmcp.callTool("read_design");
  const deleted = await edit(page, { op: "delete", id: created.screenId });
  expect(deleted.screens).toHaveLength(1);

  const result = await stale.webmcp.callTool("apply_edits", {
    revision: initial.revision,
    edits: [{ op: "patch", layer: { kind: "text", text: "Recovered screen", fontSize: 20 } }]
  });
  expect(result).toMatchObject({ isError: true });
  const status = await stale.webmcp.callTool("get_save_status");
  expect(status).toMatchObject({ state: "conflict", recovery: { title: "Recovery — Orbit study · Temporary screen" } });
  await stale.getByRole("link", { name: "Open recovery project" }).click();
  await expect(stale.locator("#shapes").getByText("Recovered screen", { exact: true })).toBeVisible();
});
