import { test, expect } from "@playwright/test";

test("frames nest, clip overlapping layers, move as a unit, and survive backup", async ({ page, browser, baseURL }) => {
  await page.goto("/?project=orbit");
  await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
  await page.getByRole("button", { name: /Frame \+/ }).click();
  await expect(page.locator("#status")).toHaveText("Saved in this browser");
  const scene = () => page.webmcp.callTool("read_design");
  const batch = async edits => page.webmcp.callTool("apply_edits", { revision: (await scene()).revision, edits });
  const outer = (await scene()).scene.groups[0];
  const frame = { bounds: { x: 200, y: 60, width: 140, height: 120 }, fill: "#bfd2c0", clip: true };
  const inner = await batch([{ op: "create_frame", name: "Inner", parentId: outer.id, frame }]);
  const leaf = await batch([{ op: "patch", layer: { kind: "text", text: "Clipped", x: 250, y: 80, width: 100, height: 30, fontSize: 18 } }]);
  await batch([{ op: "set_layer_parent", id: leaf.edited, parentId: inner.edited }]);
  const tree = await page.webmcp.callTool("read_layer_tree");
  expect(tree.find(node => node.id === outer.id).children[0]).toMatchObject({
    id: inner.edited, kind: "frame", children: [expect.objectContaining({ id: leaf.edited })]
  });
  const clipped = page.getByRole("button", { name: "Select Clipped", exact: true });
  await expect(clipped).toHaveCSS("clip-path", "inset(0% 70% 0% 0%)");

  await page.getByRole("button", { name: "Frame", exact: true }).click();
  const geometry = page.getByRole("form", { name: "Frame geometry" });
  await geometry.getByLabel("X", { exact: true }).fill("50");
  await geometry.getByRole("button", { name: "Update frame" }).click();
  await expect.poll(async () => (await scene()).scene.layers.find(layer => layer.id === leaf.edited).x).toBe(260);
  await expect(clipped).toHaveCSS("clip-path", "inset(0% 70% 0% 0%)");

  const before = await scene();
  const invalid = { ...outer.frame, bounds: { ...outer.frame.bounds, x: 240 } };
  expect(await batch([{ op: "set_frame", id: outer.id, frame: invalid }])).toMatchObject({ isError: true });
  expect(await scene()).toEqual(before);
  await page.reload();
  await expect(clipped).toHaveCSS("clip-path", "inset(0% 70% 0% 0%)");

  await page.goto("/");
  await expect.poll(async () => (await page.webmcp.tools()).map(tool => tool.name)).toContain("export_project");
  const bundle = await page.webmcp.callTool("export_project", { id: "orbit" });
  const fresh = await browser.newContext({ baseURL });
  try {
    const restored = await fresh.newPage();
    await restored.goto("/");
    await expect.poll(async () => (await restored.webmcp.tools()).map(tool => tool.name)).toContain("import_project");
    const project = await restored.webmcp.callTool("import_project", { bundle });
    await restored.goto("/?project=" + project.id);
    await expect(restored.getByRole("button", { name: "Select Clipped", exact: true }))
      .toHaveCSS("clip-path", "inset(0% 70% 0% 0%)");
  } finally {
    await fresh.close();
  }
});
