import { test, expect } from "@playwright/test";

const BASE = "http://localhost:3000";

test.describe("home page", () => {
  test("has blog title and header", async ({ page }) => {
    await page.goto(`${BASE}/`);

    await expect(page).toHaveTitle("A Macdonald — Blog");
    await expect(
      page.locator('header a[href="/"] img[alt="A Macdonald"]'),
    ).toBeVisible();
    await expect(page.locator("header p")).toContainText("Stupidly Simple");
  });

  test("lists posts", async ({ page }) => {
    await page.goto(`${BASE}/`);

    await expect(page.locator(".section-title")).toHaveText("Posts");
    // Sample post ships with the repo.
    await expect(page.locator(".post-card")).not.toHaveCount(0);
    await expect(
      page.locator('.post-card a[href="/posts/hello-world"]'),
    ).toBeVisible();
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

test("post page renders markdown", async ({ page }) => {
  await page.goto(`${BASE}/`);
  await page.locator('.post-card a[href="/posts/hello-world"]').click();

  await expect(page).toHaveURL(`${BASE}/posts/hello-world`);
  await expect(page.locator("article.post > h1")).toBeVisible();
  // Frontmatter title + rendered markdown body.
  await expect(page.locator(".post-body")).not.toBeEmpty();
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
