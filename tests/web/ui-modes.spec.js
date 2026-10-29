// Every UI mode's toolbar and status bar (ui/chrome/<mode>/), as dump-pages
// writes them, driven through `lantern.update` the way Rust pushes state.
// The contract in docs/ui-modes/spec.md ("What every mode must offer") holds
// for every mode. Each mode except Lantern also gets golden screenshots
// (Lantern's are in chrome.spec.js).
const fs = require("fs");
const { fileURLToPath } = require("url");
const { test, expect } = require("@playwright/test");
const { pageUrl, trackErrors } = require("./helpers");

const MODES = JSON.parse(fs.readFileSync(fileURLToPath(pageUrl("ui-modes.json")), "utf8"));

const TOOLBAR_STATE = {
  url: "https://example.com/a?b=c",
  title: "Example <b>page</b>",
  canBack: true,
  canFwd: true,
  loading: true,
  secure: true,
  retro: "portal",
  retroName: "Web portal (1997)",
  bookmarks: [{ title: "Example", url: "https://example.com/" }],
};
const STATUS_STATE = {
  text: "Loading <i>example.com</i>...",
  progress: 0.5,
  loading: true,
  modem: "28.8K",
  modemTip: "Dial-up: 28.8K",
  secure: true,
  canBack: true,
  canFwd: true,
};

/** Open a mode's toolbar or status page at its real view size, with state. */
async function open(browser, mode, view, state) {
  const height = view === "toolbar" ? mode.toolbarHeight : mode.statusHeight;
  const page = await browser.newPage({ viewport: { width: 800, height } });
  const errors = trackErrors(page);
  await page.goto(pageUrl(`${view}-${mode.key}.html`));
  await expect(page).toHaveTitle("lantern|1|ready|");
  await page.evaluate((s) => window.lantern.update(s), state);
  return { page, errors };
}

/** The commands a page's elements send, as [selector, command, arg]. */
async function commandsOf(page) {
  return page.evaluate(() =>
    Array.from(document.querySelectorAll("[data-cmd]"), (el, i) => {
      el.setAttribute("data-test-index", String(i));
      let arg = el.dataset.arg || "";
      if ("anchor" in el.dataset) {
        const r = el.getBoundingClientRect();
        arg = (arg ? arg + "," : "") + r.left + "," + r.bottom;
      }
      return [`[data-test-index="${i}"]`, el.dataset.cmd, arg];
    }));
}

for (const mode of MODES) {
  test.describe(`UI mode ${mode.key}`, () => {
    test("offers what every mode must", async ({ browser }) => {
      const toolbar = await open(browser, mode, "toolbar", TOOLBAR_STATE);
      const status = await open(browser, mode, "status", STATUS_STATE);
      const sent = [...(await commandsOf(toolbar.page)), ...(await commandsOf(status.page))];
      const commands = new Set(sent.map(([, command]) => command));
      for (const command of ["back", "forward", "reload", "stop", "home"]) {
        expect(commands, `${mode.key} has ${command}`).toContain(command);
      }
      await expect(toolbar.page.locator("#url")).toHaveCount(1);
      // The way to the Browser UI menu: Lantern's Retro menu or an Options menu.
      const toUiMenu = sent.some(([, command, arg]) =>
        command === "retro-menu" || (command === "menu" && arg.startsWith("options,")));
      expect(toUiMenu, `${mode.key} reaches the Browser UI menu`).toBe(true);
      // Status text and a progress meter, somewhere.
      const both = [toolbar.page, status.page];
      const count = async (sel) => (await Promise.all(both.map((p) => p.locator(sel).count()))).reduce((a, b) => a + b);
      expect(await count("#msg")).toBe(1);
      expect(await count("#meter")).toBe(1);
      expect([...toolbar.errors, ...status.errors]).toEqual([]);
    });

    test("each control sends its command", async ({ browser }) => {
      for (const [view, state] of [["toolbar", TOOLBAR_STATE], ["status", STATUS_STATE]]) {
        const { page, errors } = await open(browser, mode, view, state);
        let seq = 1;
        for (const [selector, command, arg] of await commandsOf(page)) {
          await page.click(selector);
          seq += 1;
          await expect(page).toHaveTitle(`lantern|${seq}|${command}|${encodeURIComponent(arg)}`);
        }
        if ((await page.locator("#url").count()) > 0) {
          await page.fill("#url", "rust lang|x");
          await page.press("#url", "Enter");
          seq += 1;
          await expect(page).toHaveTitle(`lantern|${seq}|go|rust%20lang%7Cx`);
        }
        // Disabled controls send nothing.
        await page.evaluate(() => window.lantern.update({ canBack: false, canFwd: false, loading: false }));
        const before = await page.title();
        for (const id of ["back", "forward", "stop"]) {
          if ((await page.locator(`#${id}`).count()) > 0) await page.click(`#${id}`);
        }
        expect(await page.title()).toBe(before);
        expect(errors).toEqual([]);
      }
    });

    test("state from lantern.update shows", async ({ browser }) => {
      const toolbar = await open(browser, mode, "toolbar", TOOLBAR_STATE);
      await expect(toolbar.page.locator("#url")).toHaveValue(TOOLBAR_STATE.url);
      if ((await toolbar.page.locator("#title").count()) > 0) {
        // Titles are text, never markup.
        await expect(toolbar.page.locator("#title")).toHaveText(TOOLBAR_STATE.title);
        await expect(toolbar.page.locator("#title b")).toHaveCount(0);
      }
      const status = await open(browser, mode, "status", STATUS_STATE);
      for (const page of [toolbar.page, status.page]) {
        if ((await page.locator("#msg").count()) > 0) {
          await expect(page.locator("#msg")).toHaveText(STATUS_STATE.text);
        }
        for (const id of ["back", "forward", "stop"]) {
          if ((await page.locator(`#${id}`).count()) > 0) {
            await expect(page.locator(`#${id}`)).not.toHaveClass(/disabled/);
          }
        }
      }
      if (mode.key === "lantern") return;
      for (const page of [toolbar.page, status.page]) await page.mouse.move(0, 0);
      await expect(toolbar.page).toHaveScreenshot(`toolbar-${mode.key}.png`);
      await expect(status.page).toHaveScreenshot(`status-${mode.key}.png`);
      await toolbar.page.evaluate(() =>
        window.lantern.update({ url: "", title: "", canBack: false, canFwd: false, loading: false, secure: false }));
      await status.page.evaluate(() =>
        window.lantern.update({ text: "Done.", progress: 1, loading: false, modem: "", secure: false, canBack: false, canFwd: false }));
      await expect(toolbar.page).toHaveScreenshot(`toolbar-${mode.key}-idle.png`);
      await expect(status.page).toHaveScreenshot(`status-${mode.key}-idle.png`);
    });
  });
}
