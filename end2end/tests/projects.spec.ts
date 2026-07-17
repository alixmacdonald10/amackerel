import { test, expect } from "@playwright/test";

const BASE = "http://localhost:3000";

test.describe("home page", () => {
  test("has projects title and header", async ({ page }) => {
    await page.goto(`${BASE}/`);

    await expect(page).toHaveTitle("A Macdonald — Projects");
    await expect(
      page.locator('header a[href="/"] img[alt="A Macdonald"]'),
    ).toBeVisible();
    await expect(page.locator("header p").first()).toContainText("simple");
  });

  test("lists projects or shows empty state", async ({ page }) => {
    await page.goto(`${BASE}/`);

    await expect(page.locator(".section-title")).toHaveText("Projects");

    // Projects are fetched live from GitHub, so the page can land in three
    // states: populated list, empty state, or a load-error notice (no network
    // / rate limited in CI). Handle each without flaking.
    const emptyState = page.locator('img[alt="No projects yet"]');
    const loadError = page.locator('img[alt="Failed to load projects"]');

    if (await emptyState.count()) {
      await expect(emptyState).toBeVisible();
      await expect(page.locator("body")).toContainText(
        "Nothing here yet, I'm still fishing for ideas.",
      );
      await expect(page.locator(".post-card")).toHaveCount(0);
    } else if (await loadError.count()) {
      // Projects unreachable — show the error image + notice.
      await expect(loadError).toBeVisible();
      await expect(page.locator("body")).toContainText(
        "Couldn't reel in the projects — try again later.",
      );
      await expect(page.locator(".post-card")).toHaveCount(0);
    } else {
      await expect(page.locator(".post-card")).not.toHaveCount(0);

      // Each card links out to its GitHub repo in a new tab.
      const firstCard = page.locator(".post-card").first();
      const link = firstCard.locator("a.card-link");
      await expect(link).toHaveAttribute("target", "_blank");
      await expect(link).toHaveAttribute("href", /^https:\/\/github\.com\//);
      await expect(firstCard.locator("h3")).toBeVisible();
    }
  });

  test("nav has an external GitHub link", async ({ page }) => {
    await page.goto(`${BASE}/`);

    const gh = page.locator(
      'header nav a[href="https://github.com/alixmacdonald10"]',
    );
    await expect(gh).toBeVisible();
    await expect(gh).toHaveAttribute("target", "_blank");
  });
});

test("about page loads via nav", async ({ page }) => {
  await page.goto(`${BASE}/`);
  await page.locator("header nav a", { hasText: "About" }).click();

  await expect(page).toHaveURL(`${BASE}/about`);
  await expect(page.locator("article.about h1")).toContainText("Alix");
  await expect(page.locator("article.about")).toContainText("KISS");
});

test("unknown route shows 404 page", async ({ page }) => {
  await page.goto(`${BASE}/does-not-exist`);
  await expect(page.locator("h1")).toContainText("404");
  await expect(page.locator("body")).toContainText("This page swam away.");
  await expect(page.locator('img[alt="404 — page not found"]')).toBeVisible();
  await page.locator("a", { hasText: "Back to shore" }).click();
  await expect(page).toHaveURL(`${BASE}/`);
});
