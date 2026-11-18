import { test, expect } from "@playwright/test";

test("designer creates and nests groups, expands them, and reopens the hierarchy", async ({ page }) => {
  await page.goto("/?project=orbit");
  await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
  const layers = page.getByRole("complementary", { name: "Layers panel" });
  const addGroup = async name => {
    await layers.getByLabel("New group name").fill(name);
    await layers.getByRole("button", { name: "Add layer group" }).click();
    await expect(layers.getByRole("button", { name, exact: true })).toBeEnabled();
    await expect(page.locator("#status")).toHaveText("Saved in this browser");
  };
  const parent = page.getByLabel("Parent group");
  await addGroup("Artwork");
  await addGroup("Orbit objects");
  await parent.selectOption({ label: "Artwork" });
  await expect(page.locator("#status")).toHaveText("Saved in this browser");

  await layers.getByRole("button", { name: "satellite", exact: true }).click();
  await parent.selectOption({ label: "Orbit objects" });
  await expect(page.locator("#status")).toHaveText("Saved in this browser");
  const tree = await page.webmcp.callTool("read_layer_tree");
  const artwork = tree.find(node => node.name === "Artwork");
  const orbit = artwork.children.find(node => node.name === "Orbit objects");
  expect(orbit.children).toEqual([{ id: "satellite", kind: "ellipse", name: "satellite", children: [] }]);

  const nested = page.locator("#tree-" + artwork.id).locator("#tree-" + orbit.id).locator("#tree-satellite");
  await expect(nested).toBeVisible();
  await layers.getByRole("button", { name: "Collapse Artwork", exact: true }).click();
  await expect(nested).toHaveCount(0);
  await layers.getByRole("button", { name: "Expand Artwork", exact: true }).click();
  await expect(nested).toBeVisible();
  await page.reload();
  await expect(nested).toBeVisible();
});

test("agent organizes component instances and rejects group cycles atomically", async ({ page }) => {
  await page.goto("/?project=orbit");
  await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
  const place = await page.webmcp.callTool("insert_component", { id: "primary-button", x: 30, y: 400 });
  const instance = place.scene.instances[0];
  const apply = async edits => {
    const design = await page.webmcp.callTool("read_design");
    return page.webmcp.callTool("apply_edits", { revision: design.revision, edits });
  };
  const outer = (await apply([{ op: "create_group", name: "UI" }])).edited;
  const inner = (await apply([{ op: "create_group", name: "Actions", parentId: outer }])).edited;
  const nested = await apply([{ op: "set_layer_parent", id: instance.id, parentId: inner }]);
  expect(nested.isError).toBeUndefined();
  const tree = await page.webmcp.callTool("read_layer_tree");
  const row = tree.find(node => node.id === outer).children[0].children[0];
  expect(row).toMatchObject({ id: instance.id, kind: "instance" });
  expect(row.children.map(node => node.id)).toEqual([...instance.layerIds].reverse());
  const invalid = await apply([
    { op: "rename_group", id: inner, name: "Would have changed" },
    { op: "set_layer_parent", id: outer, parentId: inner }
  ]);
  expect(invalid).toMatchObject({ isError: true });
  expect(await page.webmcp.callTool("read_layer_tree")).toEqual(tree);
  expect(await apply([{ op: "set_layer_parent", id: instance.layerIds[0], parentId: outer }]))
    .toMatchObject({ isError: true });
  const ungrouped = await apply([{ op: "ungroup", id: inner }]);
  expect(ungrouped.scene.groups).toEqual([{ id: outer, name: "UI", children: [instance.id] }]);
  await page.webmcp.callTool("undo_edit", { revision: ungrouped.revision });
  expect(await page.webmcp.callTool("read_layer_tree")).toEqual(tree);
});
