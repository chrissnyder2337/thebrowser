// Each UI mode's scrollbars (ui/chrome/scrollbars/), as dump-pages writes the
// script Rust runs in content frames: every style on a long fixture page,
// with a golden of the vertical scrollbar, and the off script taking the
// sheet back out. Headless Chromium hides scrollbars, so this spec shows them.
const fs = require("fs");
const { fileURLToPath } = require("url");
const { test, expect } = require("@playwright/test");
const { pageUrl, serveFixtures, trackErrors } = require("./helpers");

test.use({ launchOptions: { ignoreDefaultArgs: ["--hide-scrollbars"] } });

const MODES = JSON.parse(fs.readFileSync(fileURLToPath(pageUrl("ui-modes.json")), "utf8"));
const STYLES = [...new Set(MODES.map((mode) => mode.scrollbars).filter(Boolean))];
const script = (name) => fs.readFileSync(fileURLToPath(pageUrl(`scrollbars-${name}.js`)), "utf8");

test.beforeEach(async ({ context }) => serveFixtures(context));

test("every mode but Lantern names a scrollbar style", () => {
  for (const mode of MODES) {
    expect(Boolean(mode.scrollbars), mode.key).toBe(mode.key !== "lantern");
  }
});

for (const style of STYLES) {
  test(`${style} scrollbars`, async ({ page }) => {
    const errors = trackErrors(page);
    await page.setViewportSize({ width: 400, height: 300 });
    await page.goto("http://fixtures.lantern.test/long.html");
    await page.evaluate(script(style));
    const width = await page.evaluate(() => innerWidth - document.documentElement.clientWidth);
    if (style === "none") {
      expect(width).toBe(0);
    } else {
      expect(width).toBeGreaterThan(0);
      await expect(page).toHaveScreenshot(`scrollbars-${style}.png`, {
        clip: { x: 400 - width - 40, y: 0, width: width + 40, height: 300 },
      });
    }
    // The off script takes the sheet back out.
    await page.evaluate(script("off"));
    expect(await page.evaluate(() => document.adoptedStyleSheets.length)).toBe(0);
    expect(errors).toEqual([]);
  });
}
