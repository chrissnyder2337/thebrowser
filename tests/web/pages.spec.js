// Lantern's internal pages (ui/pages/), as written by
// `cargo run --example dump-pages`: no script errors, untrusted text shown as
// text, links back into Rust only through lantern.invalid, and a golden each.
const { test, expect } = require("@playwright/test");
const { pageUrl, trackErrors } = require("./helpers");

const PAGES = ["home", "home-empty", "bookmarks", "about", "settings", "settings-saved", "error"];

for (const name of PAGES) {
  test(`${name} page`, async ({ page }) => {
    const errors = trackErrors(page);
    await page.goto(pageUrl(`${name}.html`));
    // The only http(s) links that aren't bookmarks go to lantern.invalid.
    const hrefs = await page.evaluate(() => [...document.querySelectorAll("a[href], form[action]")]
      .map((e) => e.getAttribute("href") || e.getAttribute("action")));
    for (const href of hrefs) {
      expect(href, "no script URLs").not.toMatch(/^\s*javascript:/i);
    }
    await expect(page.locator("b", { hasText: "Not bold" })).toHaveCount(0);
    await expect(page.locator("script[src]")).toHaveCount(0);
    await expect(page).toHaveScreenshot(`${name}.png`, { fullPage: true });
    expect(errors).toEqual([]);
  });
}

test("bookmarks page links each row to its delete action", async ({ page }) => {
  await page.goto(pageUrl("bookmarks.html"));
  const deletes = await page.locator("a", { hasText: "Delete" }).evaluateAll((links) => links.map((a) => a.href));
  expect(deletes).toEqual([
    "https://lantern.invalid/delete?url=" + encodeURIComponent("https://github.com/chrissnyder2337/thebrowser"),
    "https://lantern.invalid/delete?url=" + encodeURIComponent("https://example.com/"),
    "https://lantern.invalid/delete?url=" + encodeURIComponent("https://example.org/?q=<script>&x=1"),
  ]);
  await expect(page.getByText('<b>Not bold</b> & "quoted"')).toHaveCount(1);
});

test("settings form posts to lantern.invalid", async ({ page }) => {
  await page.goto(pageUrl("settings.html"));
  expect(await page.locator("form").getAttribute("action")).toBe("https://lantern.invalid/settings");
  await expect(page.locator("input[name=retro]")).toHaveCount(4);
  await expect(page.locator("input[name=retro][value=off]")).toBeChecked();
});

test("error page escapes the failed URL", async ({ page }) => {
  await page.goto(pageUrl("error.html"));
  await expect(page.getByText("https://no-such-host.invalid/<x>").first()).toBeVisible();
  await expect(page.locator("x")).toHaveCount(0);
});
