import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test.describe("Overview page", () => {
  test("renders page heading and disclaimer", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByRole("heading", { name: "Overview" })).toBeVisible();
    await expect(page.getByText("labor-equivalent estimates")).toBeVisible();
    await expect(page.getByText("Total Tasks (7d)")).toBeVisible();
    await expect(page.getByText("Financial Equivalent Cost")).toBeVisible();
  });

  test("overview passes accessibility smoke", async ({ page }) => {
    await page.goto("/");
    const results = await new AxeBuilder({ page }).analyze();
    expect(results.violations).toEqual([]);
  });

  test("filter links and drill-down expose explainability", async ({
    page,
  }) => {
    await page.goto("/trends?range=30d");
    await expect(page.getByRole("heading", { name: "Trends" })).toBeVisible();
    await expect(page.getByRole("link", { name: "30d" })).toBeVisible();
    await page.getByRole("link", { name: "PRD section synthesis" }).click();
    await expect(
      page.getByRole("heading", { name: "PRD section synthesis" }),
    ).toBeVisible();
    await expect(
      page.getByRole("heading", { name: "Explainability Trace" }),
    ).toBeVisible();
    await expect(page.getByText("Base cognitive units")).toBeVisible();
  });

  test("settings persist across reload", async ({ page }) => {
    await page.goto("/settings");
    await page.getByLabel("Date range").selectOption("30d");
    await page.getByLabel("Table density").selectOption("compact");
    await page.reload();
    await expect(page.getByText("Saved: 30d / compact")).toBeVisible();
  });

  test("admin actions deny non-admin role", async ({ page }) => {
    await page.goto("/admin");
    await expect(
      page.getByRole("heading", { name: "Admin Console" }),
    ).toBeVisible();
    await expect(page.getByText("Admin role required")).toBeVisible();
    await expect(
      page.getByRole("button", { name: "Open" }).first(),
    ).toBeDisabled();
  });

  test("admin calibration help page renders denied state", async ({ page }) => {
    await page.goto("/admin/calibration");
    await expect(
      page.getByRole("heading", { name: "Calibration Guide" }),
    ).toBeVisible();
    await expect(page.getByText("Admin or owner role required")).toBeVisible();
    await expect(page.getByText("Run the reference pack")).toBeVisible();
  });

  test("queue health page renders operational counters", async ({ page }) => {
    await page.goto("/queue");
    await expect(
      page.getByRole("heading", { name: "Queue Health" }),
    ).toBeVisible();
    await expect(page.getByText("Pending")).toBeVisible();
    await expect(page.getByText("DLQ")).toBeVisible();
  });

  test("visual regression: overview", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("main")).toHaveScreenshot("overview-page.png");
  });

  test("visual regression: explainability", async ({ page }) => {
    await page.goto("/events/00000000-0000-4000-8000-000000000101");
    await expect(page.locator("main")).toHaveScreenshot(
      "explainability-page.png",
    );
  });
});

test.describe("Test Orchestration page", () => {
  test("renders page heading and fixture list", async ({ page }) => {
    await page.goto("/test-orchestration");
    await expect(
      page.getByRole("heading", { name: "Test Orchestration" }),
    ).toBeVisible();
    await expect(page.getByText("valid/01-prd-canonical.json")).toBeVisible();
  });

  test("Run button is disabled when no fixture selected", async ({ page }) => {
    await page.goto("/test-orchestration");
    const runButton = page.getByRole("button", { name: "Run" });
    await expect(runButton).toBeDisabled();
  });

  test("Run button enables after fixture selection", async ({ page }) => {
    await page.goto("/test-orchestration");
    await page.getByText("valid/01-prd-canonical.json").click();
    const runButton = page.getByRole("button", { name: "Run" });
    await expect(runButton).toBeEnabled();
  });
});
