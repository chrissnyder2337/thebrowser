// Lantern's internal pages (ui/pages/), as written by
// `cargo run --example dump-pages`: no script errors, untrusted text shown as
// text, links back into Rust only through lantern.invalid, and a golden each.
const { test, expect } = require("@playwright/test");
const { pageUrl, trackErrors } = require("./helpers");

// Links the pages show as content (dump-pages' bookmarks, error and credits).
const CONTENT_LINKS = [
  "https://github.com/chrissnyder2337/thebrowser",
  "https://example.com/",
  "https://example.org/?q=%3Cscript%3E&x=1",
  "https://no-such-host.invalid/%3Cx%3E",
  "https://bitbucket.org/chromiumembedded/cef",
  "https://github.com/tauri-apps/cef-rs",
  "https://duckduckgo.com/",
];

const PAGES = ["home", "home-empty", "bookmarks", "about", "settings", "settings-saved", "error"];

for (const name of PAGES) {
  test(`${name} page`, async ({ page }) => {
    const errors = trackErrors(page);
    await page.goto(pageUrl(`${name}.html`));
    // Links back into Rust go only to lantern.invalid; every other link is
    // page content: a bookmark, the failed URL, a credit or the search box.
    const hrefs = await page.evaluate(() => [...document.querySelectorAll("a[href], form[action]")]
      .map((e) => e.href || e.action));
    for (const href of hrefs) {
      const url = new URL(href);
      if (url.hostname === "lantern.invalid") {
        expect(url.protocol, href).toBe("https:");
        continue;
      }
      expect(CONTENT_LINKS, `unexpected link ${href}`).toContain(href);
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
  // Every retro look (settings.rs RetroStyle::ALL), Off checked.
  await expect(page.locator("input[name=retro]")).toHaveCount(6);
  await expect(page.locator("input[name=retro][value=off]")).toBeChecked();
  // Every UI mode, Lantern's own checked, and the three page styles.
  const modes = JSON.parse(require("fs").readFileSync(require("url").fileURLToPath(pageUrl("ui-modes.json")), "utf8"));
  await expect(page.locator("input[name=ui]")).toHaveCount(modes.length);
  await expect(page.locator("input[name=ui][value=lantern]")).toBeChecked();
  await expect(page.locator("input[name=pages]")).toHaveCount(3);
});

test("error page escapes the failed URL", async ({ page }) => {
  await page.goto(pageUrl("error.html"));
  await expect(page.getByText("https://no-such-host.invalid/<x>").first()).toBeVisible();
  await expect(page.locator("x")).toHaveCount(0);
});
