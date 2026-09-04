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

    await expect(page.getByRole("main")).toContainText(
      "Trawl through the shoal of projects",
    );

    // Projects are fetched live from GitHub, so the page can land in three
    // states: populated list, empty state, or a load-error notice (no network
    // / rate limited in CI). Handle each without flaking.
    const emptyState = page.locator('img[alt="No projects yet"]');
    const loadError = page.locator('img[alt="Failed to load projects"]');
    const cards = page.locator("main ul > li");

    if (await emptyState.count()) {
      await expect(emptyState).toBeVisible();
      await expect(page.locator("body")).toContainText(
        "Nothing here yet, I'm still fishing for ideas.",
      );
      await expect(cards).toHaveCount(0);
    } else if (await loadError.count()) {
      // Projects unreachable — show the error image + notice.
      await expect(loadError).toBeVisible();
      await expect(page.locator("body")).toContainText(
        "Couldn't reel in the projects — try again later.",
      );
      await expect(cards).toHaveCount(0);
    } else {
      await expect(cards).not.toHaveCount(0);

      // Each card links out to its GitHub repo in a new tab.
      const firstCard = cards.first();
      const link = firstCard.locator('a[href^="https://github.com/"]');
      await expect(link).toHaveAttribute("target", "_blank");
      await expect(link).toHaveAttribute("href", /^https:\/\/github\.com\//);

      // The project name is the first <span> inside the card link; it carries
      // no heading tag, so assert it renders rather than pinning the element.
      const title = link.locator("span").first();
      await expect(title).toBeVisible();
      await expect(title).not.toBeEmpty();
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

  const about = page.locator("main article");
  await expect(about.getByRole("heading", { level: 1 })).toContainText("Alix");
  await expect(about).toContainText("KISS");
});

test("unknown route shows 404 page", async ({ page }) => {
  await page.goto(`${BASE}/does-not-exist`);
  await expect(page.locator("body")).toContainText("This page swam away.");
  await page.locator("a", { hasText: "Back to shore" }).click();
  await expect(page).toHaveURL(`${BASE}/`);
});
