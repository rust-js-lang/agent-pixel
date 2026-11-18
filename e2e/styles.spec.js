import { test, expect } from "@playwright/test";

test("dashboard, editor and prototype styles are scoped", async ({ page }) => {
  const globalRules = ".project-card, .workspace, .prototype-hotspot { display: none !important; }";

  await page.goto("/");
  const card = page.getByRole("link", { name: /Orbit study/ });
  await page.addStyleTag({ content: globalRules });
  await expect(card).toBeVisible();
  await expect(page.getByRole("region", { name: "Projects" })).toHaveCSS("display", "grid");

  await card.click();
  await page.addStyleTag({ content: globalRules });
  await expect(page.getByRole("region", { name: "Canvas" })).toHaveCSS("display", "flex");
  await expect(page.locator("#art")).toHaveCSS("aspect-ratio", "4 / 5");

  await page.setViewportSize({ width: 390, height: 850 });
  await page.goto("/?project=current-reminders");
  const screens = page.getByRole("navigation", { name: "Journey screens" });
  await expect(screens).toBeVisible();
  await screens.getByRole("link", { name: "Play prototype" }).click();
  await page.addStyleTag({ content: globalRules });
  const action = page.getByRole("link", { name: "Set off-peak reminder" });
  await expect(action).toBeVisible();
  await expect(action).toHaveCSS("position", "absolute");
  await action.click();
  await expect(page.getByRole("heading", { name: "Set schedule" })).toBeVisible();
});
