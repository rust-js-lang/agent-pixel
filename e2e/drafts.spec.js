import { test, expect } from "@playwright/test";

async function ready(page, url) {
  await page.goto(url);
  await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
}

async function patch(page, x) {
  const before = await page.webmcp.callTool("read_design");
  return page.webmcp.callTool("apply_edits", { revision: before.revision, edits: [
    { op: "patch", layer: { id: "satellite", x } }
  ] });
}

async function satelliteX(page) {
  return (await page.webmcp.callTool("read_design")).scene.layers.find(layer => layer.id === "satellite").x;
}

async function allowInterruptedDraftWrites(page) {
  await page.addInitScript(() => {
    const put = IDBObjectStore.prototype.put;
    IDBObjectStore.prototype.put = function (...args) {
      if (globalThis.abortNextDraftWrite && this.name === "drafts") {
        globalThis.abortNextDraftWrite = false;
        this.transaction.abort();
        return;
      }
      return put.apply(this, args);
    };
  });
}

test("a stale tab saves a separate recovery project without overwriting the newer draft", async ({ page, context }) => {
  const stale = await context.newPage();
  await Promise.all([ready(page, "/?project=orbit"), ready(stale, "/?project=orbit")]);

  expect((await patch(page, 221)).isError).toBeUndefined();
  expect((await patch(stale, 155)).isError).toBe(true);
  const status = await stale.webmcp.callTool("get_save_status");
  expect(status).toMatchObject({ state: "conflict", recovery: { title: "Recovery — Orbit study" } });
  await Promise.all([
    stale.waitForURL(url => url.searchParams.get("project") === status.recovery.id),
    stale.getByRole("link", { name: "Open recovery project" }).click(),
  ]);
  await expect(stale.locator("#agent")).toHaveText("WebMCP tools ready");
  expect(await satelliteX(stale)).toBe(155);

  await ready(page, "/?project=orbit");
  expect(await satelliteX(page)).toBe(221);
  await page.goto("/");
  await expect(page.getByRole("link", { name: /Recovery — Orbit study/ })).toBeVisible();
});

test("a saved legacy draft moves to IndexedDB with its original retained", async ({ page }) => {
  const key = "agent-pixel:poster:v1";
  const legacy = JSON.stringify({ revision: 0, scene: { version: 1, layers: [
    { id: "satellite", kind: "rectangle", x: 123, y: 25, width: 40, height: 40, fill: "#247a69" }
  ] } });
  await page.goto("/");
  await page.evaluate(([key, value]) => localStorage.setItem(key, value), [key, legacy]);
  await ready(page, "/?project=orbit");
  expect(await satelliteX(page)).toBe(123);
  expect(await page.webmcp.callTool("get_save_status")).toMatchObject({ state: "saved", storageRevision: expect.any(String) });
  expect(await page.evaluate(key => localStorage.getItem(key), key)).toBe(legacy);
  await page.evaluate(key => localStorage.removeItem(key), key);
  await ready(page, "/?project=orbit");
  expect(await satelliteX(page)).toBe(123);
});

test("an interrupted commit leaves the previous saved draft intact and can be retried", async ({ page, context }) => {
  await allowInterruptedDraftWrites(page);
  await ready(page, "/?project=orbit");
  await patch(page, 221);
  await page.evaluate(() => { globalThis.abortNextDraftWrite = true; });
  expect((await patch(page, 155)).isError).toBe(true);
  expect(await page.webmcp.callTool("get_save_status")).toMatchObject({ state: "unsaved" });
  expect(await satelliteX(page)).toBe(155);
  const other = await context.newPage();
  await ready(other, "/?project=orbit");
  expect(await satelliteX(other)).toBe(221);

  expect(await page.webmcp.callTool("retry_draft_save")).toMatchObject({ state: "saved" });
  await ready(other, "/?project=orbit");
  expect(await satelliteX(other)).toBe(155);
});

test("opening the same legacy draft in two tabs migrates it once", async ({ page, context }) => {
  await page.goto("/");
  await page.evaluate(() => localStorage.setItem("agent-pixel:poster:v1", JSON.stringify({
    revision: 0, scene: { version: 1, layers: [
      { id: "satellite", kind: "rectangle", x: 95, y: 20, width: 40, height: 40, fill: "#247a69" }
    ] }
  })));
  const other = await context.newPage();
  await Promise.all([ready(page, "/?project=orbit"), ready(other, "/?project=orbit")]);
  const [first, second] = await Promise.all([
    page.webmcp.callTool("get_save_status"), other.webmcp.callTool("get_save_status")
  ]);
  expect(first.storageRevision).toEqual(second.storageRevision);
  expect(first.state).toBe("saved");
  expect(second.state).toBe("saved");
  await page.goto("/");
  await expect(page.getByRole("region", { name: "Projects" })).toBeVisible();
  await expect(page.getByRole("link", { name: /Recovery — Orbit study/ })).toHaveCount(0);
});

test("recover the previous checkpoint into a separate project through WebMCP and the editor", async ({ page, context }) => {
  await ready(page, "/?project=orbit");
  await patch(page, 221);
  const first = await page.webmcp.callTool("get_save_status");
  expect(first.hasPrevious).toBe(false);
  await patch(page, 155);
  const second = await page.webmcp.callTool("get_save_status");
  expect(second.hasPrevious).toBe(true);
  const recover = revision => page.webmcp.callTool("recover_checkpoint", { revision });
  expect(await recover(first.storageRevision)).toMatchObject({ isError: true });
  const checkpoint = await recover(second.storageRevision);
  expect(checkpoint.title).toBe("Checkpoint — Orbit study");
  expect(await satelliteX(page)).toBe(155);

  const copy = await context.newPage();
  await ready(copy, "/?project=" + checkpoint.id);
  expect(await satelliteX(copy)).toBe(221);

  await page.getByRole("button", { name: "Copy previous checkpoint" }).click();
  await expect(page.getByRole("heading", { name: "Checkpoint — Orbit study" })).toBeVisible();
  await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
  expect(await satelliteX(page)).toBe(221);
});

test("recover an unsaved multi-screen project after closing its editor", async ({ page, browser, baseURL }) => {
  const source = "/?project=current-reminders";
  await allowInterruptedDraftWrites(page);
  await ready(page, source);
  await page.webmcp.callTool("edit_layer", { id: "page-title", text: "Saved overview" });
  await ready(page, source + "&screen=schedule");
  expect(await page.webmcp.callTool("export_unsaved_project")).toMatchObject({ isError: true });
  const placed = await page.webmcp.callTool("insert_component", { id: "primary-button", x: 30, y: 470 });

  await page.evaluate(() => { globalThis.abortNextDraftWrite = true; });
  const rejected = await page.webmcp.callTool("apply_edits", {
    revision: placed.revision,
    edits: [{ op: "patch", layer: { id: "page-title", text: "Unsaved schedule" } }]
  });
  expect(rejected).toMatchObject({ isError: true });
  expect(await page.webmcp.callTool("get_save_status")).toMatchObject({ state: "unsaved", canExportUnsaved: true });
  const live = await page.webmcp.callTool("read_design");
  const bundle = await page.webmcp.callTool("export_unsaved_project");
  expect(bundle.screens).toHaveLength(5);
  expect(bundle.screens.find(screen => screen.info.id === "schedule").scene).toEqual(live.scene);
  expect(bundle.screens.find(screen => screen.info.id === "overview").scene.layers)
    .toContainEqual(expect.objectContaining({ id: "page-title", text: "Saved overview" }));
  expect(bundle.library).toContainEqual(expect.objectContaining({ id: "primary-button" }));
  expect(await page.webmcp.callTool("get_save_status")).toMatchObject({ state: "unsaved" });

  const [download] = await Promise.all([
    page.waitForEvent("download"),
    page.getByRole("button", { name: "Download unsaved project" }).click()
  ]);
  expect(download.suggestedFilename()).toBe("agent-pixel-unsaved-project.json");
  const portable = await Bun.file(await download.path()).json();
  expect(portable).toEqual(bundle);
  await page.close();

  const fresh = await browser.newContext({ baseURL });
  try {
    const restored = await fresh.newPage();
    await restored.goto("/");
    await expect.poll(async () => (await restored.webmcp.tools()).map(tool => tool.name)).toContain("import_project");
    const project = await restored.webmcp.callTool("import_project", { bundle: portable });
    await ready(restored, `/?project=${project.id}&screen=schedule`);
    expect((await restored.webmcp.callTool("read_design")).scene).toEqual(live.scene);
    await expect(restored.locator("#shapes").getByText("Unsaved schedule", { exact: true })).toBeVisible();
    await ready(restored, `/?project=${project.id}`);
    await expect(restored.locator("#shapes").getByText("Saved overview", { exact: true })).toBeVisible();
  } finally {
    await fresh.close();
  }
});
