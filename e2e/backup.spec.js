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

test("migrates the legacy catalog and bundle while preserving version one drafts", async ({ page }) => {
  const project = { id: "2f407ea0-ea8f-430e-9957-c92c1614c01d", title: "Migrated journey" };
  await page.goto("/");
  await expect.poll(async () => (await page.webmcp.tools()).map(tool => tool.name)).toContain("export_project");
  const bundle = await page.webmcp.callTool("export_project", { id: "current-reminders" });
  const legacy = { ...bundle, title: project.title };
  const seed = "**/migration-fixture";
  await page.route(seed, route => route.fulfill({ contentType: "text/html", body: "<html></html>" }));
  await page.goto("/migration-fixture");
  const originalRecords = await page.evaluate(async ({ project, legacy }) => {
    const complete = request => new Promise((resolve, reject) => {
      request.onsuccess = () => resolve(request.result);
      request.onerror = () => reject(request.error);
    });
    await complete(indexedDB.deleteDatabase("agent-pixel"));
    const open = indexedDB.open("agent-pixel", 1);
    open.onupgradeneeded = () => open.result.createObjectStore("drafts");
    const db = await complete(open);
    const key = `agent-pixel:project:${project.id}:v1:schedule`;
    const scene = structuredClone(legacy.screens.find(screen => screen.info.id === "schedule").scene);
    scene.layers.find(layer => layer.id === "page-title").text = "An existing saved revision";
    const tx = db.transaction("drafts", "readwrite");
    tx.objectStore("drafts").add(JSON.stringify({
      key, revision: "legacy-revision", document: { revision: 1, scene }, previous: null, recovery: null
    }), key);
    await new Promise((resolve, reject) => {
      tx.oncomplete = resolve;
      tx.onabort = () => reject(tx.error);
    });
    db.close();
    const catalog = JSON.stringify([project]);
    const bundle = JSON.stringify(legacy);
    localStorage.setItem("agent-pixel:projects:v1", catalog);
    localStorage.setItem(`agent-pixel:bundle:${project.id}`, bundle);
    return [catalog, bundle];
  }, { project, legacy });

  await page.goto("/");
  await expect(page.getByRole("link", { name: new RegExp(project.title) })).toBeVisible();
  expect(await page.evaluate(id => [
    localStorage.getItem("agent-pixel:projects:v1"),
    localStorage.getItem(`agent-pixel:bundle:${id}`)
  ], project.id)).toEqual(originalRecords);
  await page.evaluate(id => {
    localStorage.removeItem("agent-pixel:projects:v1");
    localStorage.removeItem(`agent-pixel:bundle:${id}`);
  }, project.id);

  await page.goto(`/?project=${project.id}&screen=schedule`);
  await expect(page.locator("#shapes").getByText("An existing saved revision", { exact: true })).toBeVisible();
  await expect(page.getByRole("navigation", { name: "Journey screens" }).getByRole("link", { name: "Set schedule" })).toBeVisible();
});

test("concurrent WebMCP creates persist both catalog entries and refresh both dashboards", async ({ page, context }) => {
  const other = await context.newPage();
  const dashboards = [page, other];
  await Promise.all(dashboards.map(async dashboard => {
    await dashboard.goto("/");
    await expect.poll(async () => (await dashboard.webmcp.tools()).map(tool => tool.name)).toContain("create_project");
  }));
  const created = await Promise.all(dashboards.map((dashboard, index) => dashboard.webmcp.callTool("create_project", {
    title: `Concurrent project ${index + 1}`
  })));
  expect(new Set(created.map(project => project.id)).size).toBe(2);
  for (const dashboard of dashboards) {
    for (const project of created) {
      await expect(dashboard.locator(`a[href="?project=${project.id}"]`)).toBeVisible();
    }
  }
  await page.reload();
  await expect.poll(async () => (await page.webmcp.tools()).map(tool => tool.name)).toContain("list_projects");
  expect(await page.webmcp.callTool("list_projects")).toEqual(expect.arrayContaining(created));
});

test("an aborted import publishes neither a project nor its bundle and can be retried", async ({ page }) => {
  await page.goto("/");
  await expect.poll(async () => (await page.webmcp.tools()).map(tool => tool.name)).toContain("import_project");
  const before = await page.webmcp.callTool("list_projects");
  const bundle = await page.webmcp.callTool("export_project", { id: "current-reminders" });
  const input = { bundle: { ...bundle, title: "Atomic import" } };
  await page.evaluate(() => {
    const original = IDBObjectStore.prototype.add;
    IDBObjectStore.prototype.add = function (...args) {
      const request = original.apply(this, args);
      if (this.name === "projects") {
        IDBObjectStore.prototype.add = original;
        this.transaction.abort();
      }
      return request;
    };
  });
  expect(await page.webmcp.callTool("import_project", input)).toMatchObject({ isError: true });
  await page.reload();
  await expect.poll(async () => (await page.webmcp.tools()).map(tool => tool.name)).toContain("import_project");
  expect(await page.webmcp.callTool("list_projects")).toEqual(before);
  const project = await page.webmcp.callTool("import_project", input);
  expect(project.title).toBe("Atomic import");
  expect(await page.webmcp.callTool("export_project", { id: project.id })).toEqual(input.bundle);
});
