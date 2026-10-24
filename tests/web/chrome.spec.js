// The toolbar and status bar (ui/toolbar.html, ui/status.html) as Lantern
// builds them, driven through a stubbed `lantern.update` the way Rust pushes
// state. Each command must reach Rust as `lantern|<seq>|<command>|<arg>`
// (parsed by `commands::parse_title`).
const { test, expect } = require("@playwright/test");
const { pageUrl, trackErrors } = require("./helpers");

// The views' real sizes (window.rs): toolbar 112 DIP, status bar 24 DIP.
const TOOLBAR = { width: 800, height: 112 };
const STATUS = { width: 800, height: 24 };

const TOOLBAR_STATE = {
  url: "https://example.com/a?b=c",
  canBack: true,
  canFwd: true,
  loading: true,
  secure: true,
  retro: "portal",
  retroName: "Web portal (1997)",
  bookmarks: [
    { title: "Example", url: "https://example.com/a b?c=d&e" },
    { title: "<b>Not bold</b>", url: "https://example.org/" },
  ],
};

/** Commands sent so far: the title channel's last value. */
async function lastTitle(page) {
  return page.evaluate(() => document.title);
}

test.describe("toolbar", () => {
  test.use({ viewport: TOOLBAR });

  test("each button sends its command", async ({ page }) => {
    const errors = trackErrors(page);
    await page.goto(pageUrl("toolbar.html"));
    await expect(page).toHaveTitle("lantern|1|ready|");
    await page.evaluate((s) => window.lantern.update(s), TOOLBAR_STATE);

    let seq = 1;
    const sends = async (action, command, arg = "") => {
      await action();
      seq += 1;
      await expect(page).toHaveTitle(`lantern|${seq}|${command}|${arg}`);
    };
    for (const [id, command] of [
      ["back", "back"],
      ["forward", "forward"],
      ["reload", "reload"],
      ["home", "home"],
      ["search", "search"],
      ["bookmarks", "bookmarks"],
      ["mark", "bookmark-add"],
      ["stop", "stop"],
    ]) {
      await sends(() => page.click(`#${id}`), command);
    }
    // Retro carries where its menu should open: the button's left,bottom.
    const box = await page.evaluate(() => {
      const r = document.getElementById("retro").getBoundingClientRect();
      return encodeURIComponent(r.left + "," + r.bottom);
    });
    await sends(() => page.click("#retro"), "retro-menu", box);
    await sends(() => page.click("#throbber"), "about");
    // Typed text is sent as typed, percent-encoded.
    await sends(async () => {
      await page.fill("#url", "rust lang|x");
      await page.press("#url", "Enter");
    }, "go", "rust%20lang%7Cx");
    await sends(async () => {
      await page.fill("#url", "  example.com  ");
      await page.click("#go");
    }, "go", "example.com");
    await sends(() => page.click(".ql >> nth=0"), "open", encodeURIComponent("https://example.com/a b?c=d&e"));

    // Disabled buttons send nothing; neither does an empty URL field.
    await page.evaluate(() => window.lantern.update({ canBack: false, canFwd: false, loading: false }));
    await page.click("#back");
    await page.click("#forward");
    await page.click("#stop");
    await page.fill("#url", "   ");
    await page.press("#url", "Enter");
    expect(await lastTitle(page)).toBe(`lantern|${seq}|open|${encodeURIComponent("https://example.com/a b?c=d&e")}`);
    expect(errors).toEqual([]);
  });

  test("state from lantern.update shows", async ({ page }) => {
    await page.goto(pageUrl("toolbar.html"));
    await page.evaluate((s) => window.lantern.update(s), TOOLBAR_STATE);
    await expect(page.locator("#back")).not.toHaveClass(/disabled/);
    await expect(page.locator("#stop")).not.toHaveClass(/disabled/);
    await expect(page.locator("#retro")).toHaveAttribute("aria-pressed", "true");
    await expect(page.locator("#retro")).toHaveAttribute("title", "Retro looks: Web portal (1997)");
    await expect(page.locator("#field")).toHaveClass(/secure/);
    await expect(page.locator("#url")).toHaveValue(TOOLBAR_STATE.url);
    // Bookmark titles are text, never markup.
    await expect(page.locator(".ql span").nth(1)).toHaveText("<b>Not bold</b>");
    await expect(page.locator(".ql b")).toHaveCount(0);
    await page.mouse.move(0, 0);
    await expect(page).toHaveScreenshot("toolbar.png");
    await page.evaluate(() =>
      window.lantern.update({ url: "", canBack: false, canFwd: false, loading: false, secure: false, retro: "off", retroName: "Off", bookmarks: [] }));
    await expect(page.locator("#links .none")).toHaveText("(no bookmarks)");
    await expect(page).toHaveScreenshot("toolbar-idle.png");
  });
});

test.describe("status bar", () => {
  test.use({ viewport: STATUS });

  test("the modem pane opens settings", async ({ page }) => {
    const errors = trackErrors(page);
    await page.goto(pageUrl("status.html"));
    await expect(page).toHaveTitle("lantern|1|ready|");
    await page.click("#modem-pane");
    await expect(page).toHaveTitle("lantern|2|settings|");
    expect(errors).toEqual([]);
  });

  test("state from lantern.update shows", async ({ page }) => {
    await page.goto(pageUrl("status.html"));
    await page.evaluate(() =>
      window.lantern.update({ text: "Loading <i>example.com</i>...", progress: 0.5, loading: true, modem: "28.8K", modemTip: "Dial-up: 28.8K", secure: true }));
    await expect(page.locator("#msg")).toHaveText("Loading <i>example.com</i>...");
    await expect(page.locator("#meter i")).toHaveCount(6);
    await expect(page.locator("#modem-pane")).toHaveClass(/dialup/);
    await expect(page.locator("#lock-pane")).toHaveClass(/secure/);
    await expect(page).toHaveScreenshot("status.png");
    await page.evaluate(() => window.lantern.update({ text: "Done.", progress: 7, loading: false, modem: "", secure: false }));
    await expect(page.locator("#meter i")).toHaveCount(11);
    await expect(page).toHaveScreenshot("status-idle.png");
  });
});
