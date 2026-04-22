import { test, expect } from "@playwright/test";

test.describe("Overview page", () => {
  test("renders page heading and disclaimer", async ({ page }) => {
    await page.goto("/");
    await expect(page.getByRole("heading", { name: "Overview" })).toBeVisible();
    await expect(page.getByText("labor-equivalent estimates")).toBeVisible();
  });
});

test.describe("Test Orchestration page", () => {
  test("renders page heading and fixture list", async ({ page }) => {
    await page.goto("/test-orchestration");
    await expect(
      page.getByRole("heading", { name: "Test Orchestration" })
    ).toBeVisible();
    await expect(
      page.getByText("valid/01-prd-canonical.json")
    ).toBeVisible();
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
