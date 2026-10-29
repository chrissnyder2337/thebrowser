// The Text terminal retro look (ui/retro/terminal.js), the Text terminal UI
// mode's era look, on the fixture pages. It has its own spec because the
// shared list of looks in helpers.js belongs to retro.spec.js's existing
// cases. Scripts come from dump-pages' retro-terminal.js, the exact code
// Rust runs in content frames.
const { test, expect } = require("@playwright/test");
const { FIXTURE_PAGES, openWithLook, retroScript, serveFixtures, settle, trackErrors } = require("./helpers");

const computed = (page, selector, prop) =>
  page.evaluate(([s, p]) => getComputedStyle(document.querySelector(s))[p], [selector, prop]);

test.beforeEach(async ({ context }) => serveFixtures(context));

for (const fixture of FIXTURE_PAGES) {
  test(`terminal look on ${fixture}: no errors, monospace on black`, async ({ page }) => {
    const errors = trackErrors(page);
    await openWithLook(page, fixture, "terminal");
    expect(await computed(page, "body", "fontFamily")).toMatch(/mono/i);
    const background = await page.evaluate(() =>
      [document.documentElement, document.body].map((el) => getComputedStyle(el).backgroundColor));
    expect(background).toContain("rgb(0, 0, 0)");
    // No pictures in a terminal.
    for (const img of await page.locator("img").all()) await expect(img).toBeHidden();
    if (["tables", "long"].includes(fixture)) {
      await expect(page).toHaveScreenshot(`terminal-${fixture}.png`);
    }
    expect(errors).toEqual([]);
  });
}

test("text in spans inside a link keeps the link color", async ({ page }) => {
  await page.route("http://fixtures.lantern.test/span-link.html", (route) =>
    route.fulfill({ contentType: "text/html", body: '<p><a href="#x"><span id="s">Main page</span></a></p>' }));
  const script = retroScript("terminal");
  await page.goto("http://fixtures.lantern.test/span-link.html");
  await page.evaluate(script);
  await settle(page);
  expect(await computed(page, "#s", "color")).toBe(await computed(page, "a", "color"));
  expect(await computed(page, "a", "color")).not.toBe("rgb(192, 192, 192)");
});

test("switching the terminal look off restores the page", async ({ page }) => {
  await openWithLook(page, "tables", "terminal");
  await page.evaluate(retroScript("off"));
  await settle(page);
  expect(await computed(page, "body", "fontFamily")).not.toMatch(/mono/i);
  expect(await computed(page, "body", "backgroundColor")).toBe("rgb(250, 250, 250)");
});
