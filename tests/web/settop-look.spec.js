// The Set-top TV retro look (ui/retro/settop.js), the Set-top TV UI modes'
// era look, on the fixture pages. Scripts come from dump-pages'
// retro-settop.js, the exact code Rust runs in content frames.
const { test, expect } = require("@playwright/test");
const { FIXTURE_PAGES, openWithLook, retroScript, serveFixtures, settle, trackErrors } = require("./helpers");

const computed = (page, selector, prop) =>
  page.evaluate(([s, p]) => getComputedStyle(document.querySelector(s))[p], [selector, prop]);

test.beforeEach(async ({ context }) => serveFixtures(context));

for (const fixture of FIXTURE_PAGES) {
  test(`set-top look on ${fixture}: no errors, large type, no sideways scroll`, async ({ page }) => {
    const errors = trackErrors(page);
    await openWithLook(page, fixture, "settop");
    expect(parseFloat(await computed(page, "body", "fontSize"))).toBeGreaterThanOrEqual(18);
    const sideways = await page.evaluate(() => document.documentElement.scrollWidth > innerWidth);
    expect(sideways).toBe(false);
    if (["tables", "long"].includes(fixture)) {
      await expect(page).toHaveScreenshot(`settop-${fixture}.png`);
    }
    expect(errors).toEqual([]);
  });
}

test("the first link starts selected, and off takes the selection away", async ({ page }) => {
  await openWithLook(page, "plain", "settop");
  const first = page.locator("body a[href]").first();
  await expect(first).toHaveAttribute("data-lantern-retro-current", "");
  await page.evaluate(retroScript("off"));
  await settle(page);
  await expect(page.locator("[data-lantern-retro-current]")).toHaveCount(0);
});
