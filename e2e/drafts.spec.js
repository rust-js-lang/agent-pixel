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

test("a stale tab saves a separate recovery project without overwriting the newer draft", async ({ page, context }) => {
  const stale = await context.newPage();
  await Promise.all([ready(page, "/?project=orbit"), ready(stale, "/?project=orbit")]);

  expect((await patch(page, 221)).isError).toBeUndefined();
  expect((await patch(stale, 155)).isError).toBe(true);
  const status = await stale.webmcp.callTool("get_save_status");
  expect(status).toMatchObject({ state: "conflict", recovery: { title: "Recovery — Orbit study" } });
  await stale.getByRole("link", { name: "Open recovery project" }).click();
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
