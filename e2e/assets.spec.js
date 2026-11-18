import { test, expect } from "@playwright/test";

const image = await Bun.file(new URL("../core/tests/fixtures/raster.json", import.meta.url)).json();

async function openEditor(page, project = "orbit") {
  await page.goto(`/?project=${project}`);
  await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
}

test("an uploaded image decodes and survives reloading the editor", async ({ page }) => {
  await openEditor(page);
  const bytes = Buffer.from(image.dataUrl.split(",")[1], "base64");
  await page.getByLabel("Place image").setInputFiles({ name: image.name, mimeType: "image/png", buffer: bytes });
  const placed = page.locator("#shapes").getByRole("img", { name: image.name });
  await expect(placed).toBeVisible();
  await expect(page.locator("#status")).toHaveText("Saved in this browser");
  expect(await placed.evaluate(element => element.naturalWidth)).toBe(1);
  await page.reload();
  await expect(placed).toBeVisible();
  const saved = await page.webmcp.callTool("read_design");
  const layer = saved.scene.layers.find(layer => layer.kind === "image");
  expect(saved.scene.assets).toEqual([{ id: layer.imageId, image }]);

  for (const [mimeType, extension] of [["image/jpeg", "jpg"], ["image/webp", "webp"]]) {
    const dataUrl = await page.evaluate(mimeType => {
      const canvas = document.createElement("canvas");
      canvas.width = 80;
      canvas.height = 40;
      canvas.getContext("2d").fillRect(0, 0, 80, 40);
      return canvas.toDataURL(mimeType);
    }, mimeType);
    expect(dataUrl.startsWith("data:" + mimeType + ";base64,")).toBe(true);
    const filename = "sample." + extension;
    await page.getByLabel("Place image").setInputFiles({
      name: filename, mimeType, buffer: Buffer.from(dataUrl.split(",")[1], "base64")
    });
    const picture = page.locator("#shapes").getByRole("img", { name: filename });
    await expect.poll(() => picture.evaluate(item => item.naturalWidth)).toBe(80);
    await expect(page.locator("#status")).toHaveText("Saved in this browser");
  }
});

test("WebMCP places images in atomic edits and restores a self-contained backup", async ({ page, browser, baseURL }) => {
  await openEditor(page);
  const before = await page.webmcp.callTool("read_design");
  const placement = { image, x: 40, y: 40, width: 180, height: 100 };
  const place = { op: "place_image", placement };
  const change = { op: "patch", layer: { id: "satellite", x: 220 } };
  const edit = edits => page.webmcp.callTool("apply_edits", { revision: before.revision, edits });
  const badImage = { ...image, dataUrl: "data:image/png;base64,iVBORw0KGgoAAAANSUhE" };
  expect(await edit([change, { op: "place_image", placement: { ...placement, image: badImage } }]))
    .toMatchObject({ isError: true });
  expect(await page.webmcp.callTool("read_design")).toEqual(before);
  expect(await edit([place, { op: "delete", id: "missing-layer" }])).toMatchObject({ isError: true });
  expect(await page.webmcp.callTool("read_design")).toEqual(before);

  const placed = await edit([change, place, place]);
  expect(placed.scene.assets).toHaveLength(1);
  expect(placed.scene.layers.filter(layer => layer.imageId === placed.scene.assets[0].id)).toHaveLength(2);
  const undone = await page.webmcp.callTool("undo_edit", { revision: placed.revision });
  expect(undone.scene).toEqual(before.scene);
  const redone = await page.webmcp.callTool("redo_edit", { revision: undone.revision });
  expect(redone.scene).toEqual(placed.scene);

  await page.goto("/");
  await expect.poll(async () => (await page.webmcp.tools()).map(tool => tool.name)).toContain("export_project");
  const bundle = await page.webmcp.callTool("export_project", { id: "orbit" });
  expect(bundle.screens[0].scene).toEqual(placed.scene);

  const fresh = await browser.newContext({ baseURL });
  try {
    const restored = await fresh.newPage();
    await restored.goto("/");
    await expect.poll(async () => (await restored.webmcp.tools()).map(tool => tool.name)).toContain("import_project");
    const projects = await restored.webmcp.callTool("list_projects");
    const invalid = structuredClone(bundle);
    invalid.screens[0].scene.assets = [];
    expect(await restored.webmcp.callTool("import_project", { bundle: invalid })).toMatchObject({ isError: true });
    expect(await restored.webmcp.callTool("list_projects")).toEqual(projects);

    const project = await restored.webmcp.callTool("import_project", { bundle });
    expect(project.id).toBeDefined();
    await openEditor(restored, project.id);
    const recovered = await restored.webmcp.callTool("read_design");
    expect(recovered.scene).toEqual(placed.scene);
    const images = restored.locator("#shapes").getByRole("img", { name: image.name });
    await expect(images).toHaveCount(2);
    await expect.poll(() => images.evaluateAll(items => items.map(item => item.naturalWidth))).toEqual([1, 1]);
  } finally {
    await fresh.close();
  }
});
