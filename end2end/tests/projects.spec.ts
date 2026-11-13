import { test, expect } from "@playwright/test";

const BASE = process.env.BASE_URL ?? "http://localhost:3000";

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
    // states: populated list, empty state (`empty` component text), or the
    // "Failed to fetch data" alert (GitHub unreachable / rejected). Wait for
    // one to render, then assert on whichever it is.
    const cards = page.locator("main ul > li");
    const emptyState = page.getByText(
      "Nothing here yet, I'm still fishing for ideas.",
    );
    const loadError = page.getByText("Failed to fetch data");

    await expect(cards.first().or(emptyState).or(loadError)).toBeVisible();

    if ((await loadError.isVisible()) || (await emptyState.isVisible())) {
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
