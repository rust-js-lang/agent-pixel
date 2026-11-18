import { test, expect } from "@playwright/test";

test("play the reminder journey through confirmation and save recovery", async ({ page }) => {
  await page.goto("/");
  await page.getByRole("link", { name: /Current — Reminder journey/ }).click();
  const journey = page.getByRole("navigation", { name: "Journey screens" });
  await expect(journey.getByRole("link", { name: /Developer handoff/ })).toHaveAttribute("href", /handoff\/CURRENT-REMINDERS\.md$/);
  await journey.getByRole("link", { name: "Play prototype" }).click();
  const art = page.getByRole("region", { name: "Prototype artboard" });
  const step = label => art.getByRole("link", { name: label, exact: true }).click();
  await step("Set off-peak reminder");
  await expect(art.getByRole("heading", { name: "Set schedule" })).toBeVisible();
  await step("Review reminder");
  await page.getByRole("link", { name: "Simulate save failure" }).click();
  await expect(art.getByRole("heading", { name: "Save failure" })).toBeVisible();
  await step("Try again");
  await step("Save reminder");
  await expect(art.getByRole("heading", { name: "Confirmation" })).toBeVisible();
  await step("Back to Home energy");
  await expect(art.getByRole("heading", { name: "Energy overview" })).toBeVisible();
});

test("journey screens keep separate editable drafts and share them with preview", async ({ page }) => {
  await page.goto("/?project=current-reminders");
  await expect(page.locator("#agent")).toHaveText("WebMCP tools ready");
  await page.webmcp.callTool("edit_layer", { id: "page-title", text: "Energy at home" });
  const navigator = page.getByRole("navigation", { name: "Journey screens" });
  await navigator.getByRole("link", { name: "Set schedule" }).click();
  await expect(page.locator("#shapes").getByText("Set your schedule")).toBeVisible();
  await navigator.getByRole("link", { name: "Energy overview" }).click();
  await expect(page.locator("#shapes").getByText("Energy at home", { exact: true })).toBeVisible();
  await navigator.getByRole("link", { name: "Play prototype" }).click();
  await expect(page.locator("#art").getByText("Energy at home", { exact: true })).toBeVisible();
});
