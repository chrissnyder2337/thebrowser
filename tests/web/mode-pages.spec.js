// A UI mode's browser defaults for pages (ui/chrome/<mode>/pages.css, put in
// by ui/chrome/page-defaults.js): they style what a page leaves to the
// browser and nothing a site sets itself. Scripts come from dump-pages'
// `mode-pages-<mode>.js`, the exact code Rust runs in content frames.
const fs = require("fs");
const { fileURLToPath } = require("url");
const { test, expect } = require("@playwright/test");
const { FIXTURE_ORIGIN, pageUrl, serveFixtures, trackErrors } = require("./helpers");

const script = (name) => fs.readFileSync(fileURLToPath(pageUrl(`mode-pages-${name}.js`)), "utf8");

/** Open a fixture with the script run as a load starts and once it has loaded. */
async function openWith(page, fixture, code) {
  await page.addInitScript({ content: code });
  await page.goto(`${FIXTURE_ORIGIN}/${fixture}.html`, { waitUntil: "load" });
  await page.evaluate(code);
}

const computed = (page, selector, prop) =>
  page.evaluate(([s, p]) => getComputedStyle(document.querySelector(s))[p], [selector, prop]);

test.beforeEach(async ({ context }) => serveFixtures(context));

test.describe("Workstation (1993) page defaults", () => {
  test("a plain page gets the era's grey page and Times", async ({ page }) => {
    const errors = trackErrors(page);
    await openWith(page, "plain", script("workstation-1993"));
    expect(await computed(page, "html", "backgroundColor")).toBe("rgb(191, 191, 191)");
    expect(await computed(page, "p", "fontFamily")).toContain("Times");
    // Code keeps the browser's monospace font (and its smaller size).
    expect(await computed(page, "code", "fontFamily")).toBe("monospace");
    await expect(page).toHaveScreenshot("workstation-1993-plain.png");
    expect(errors).toEqual([]);
  });

  test("a framed plain page gets the fonts but not the page color", async ({ page }) => {
    await page.route(`${FIXTURE_ORIGIN}/framed.html`, (route) =>
      route.fulfill({ contentType: "text/html", body: `<iframe src="plain.html" width="400" height="200"></iframe>` }));
    const code = script("workstation-1993");
    await page.addInitScript({ content: code });
    await page.goto(`${FIXTURE_ORIGIN}/framed.html`, { waitUntil: "load" });
    for (const frame of page.frames()) await frame.evaluate(code);
    const child = page.frames().find((f) => f.url().endsWith("/plain.html"));
    const style = (sel, prop) => child.evaluate(([s, p]) => getComputedStyle(document.querySelector(s))[p], [sel, prop]);
    expect(await style("html", "backgroundColor")).toBe("rgba(0, 0, 0, 0)");
    expect(await style("p", "fontFamily")).toContain("Times");
  });

  test("a site's own styles win", async ({ page }) => {
    const errors = trackErrors(page);
    await openWith(page, "tables", script("workstation-1993"));
    // tables.html sets its own background and font on body.
    expect(await computed(page, "html", "backgroundColor")).toBe("rgba(0, 0, 0, 0)");
    expect(await computed(page, "body", "backgroundColor")).toBe("rgb(250, 250, 250)");
    expect(await computed(page, "p", "fontFamily")).not.toContain("Times");
    expect(errors).toEqual([]);
  });

  test("switching them off takes everything out", async ({ page }) => {
    await openWith(page, "plain", script("workstation-1993"));
    await page.evaluate(script("off"));
    expect(await computed(page, "html", "backgroundColor")).toBe("rgba(0, 0, 0, 0)");
    expect(await page.evaluate(() => document.adoptedStyleSheets.length)).toBe(0);
    // Running them again doesn't stack a second sheet.
    await page.evaluate(script("workstation-1993"));
    await page.evaluate(script("workstation-1993"));
    expect(await page.evaluate(() => document.adoptedStyleSheets.length)).toBe(1);
  });
});
