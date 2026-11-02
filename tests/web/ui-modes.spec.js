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

/** The chrome views a mode shows, with their real sizes (window.rs). */
function viewsOf(mode) {
  return [
    ["toolbar", 800, mode.toolbarHeight],
    ["status", 800, mode.statusHeight],
    ["side", mode.sideWidth, 600],
    ["frame", 800, mode.frameHeight],
  ].filter(([, w, h]) => w > 0 && h > 0);
}

/** Commands the status view may send (commands.rs STATUS_COMMANDS). */
const STATUS_COMMANDS = ["back", "forward", "reload", "stop", "home", "search", "bookmarks",
  "bookmark-add", "focus-url", "settings", "panel", "go", "menu", "about"];

/**
 * Open a mode's chrome view at its real size. Rust pushes every view both
 * the toolbar and the status state, so the test does too.
 */
async function open(browser, mode, [view, width, height], states = [TOOLBAR_STATE, STATUS_STATE]) {
  const page = await browser.newPage({ viewport: { width, height } });
  const errors = trackErrors(page);
  await page.goto(pageUrl(`${view}-${mode.key}.html`));
  await expect(page).toHaveTitle("lantern|1|ready|");
  for (const state of states) await page.evaluate((s) => window.lantern.update(s), state);
  return { view, page, errors };
}

async function openAll(browser, mode) {
  const opened = [];
  for (const view of viewsOf(mode)) opened.push(await open(browser, mode, view));
  return opened;
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

async function count(opened, selector) {
  let total = 0;
  for (const { page } of opened) total += await page.locator(selector).count();
  return total;
}

for (const mode of MODES) {
  test.describe(`UI mode ${mode.key}`, () => {
    test("offers what every mode must", async ({ browser }) => {
      const opened = await openAll(browser, mode);
      const sent = [];
      for (const { view, page } of opened) {
        const commands = await commandsOf(page);
        if (view === "status") {
          for (const [, command] of commands) {
            expect(STATUS_COMMANDS, `${mode.key} status sends ${command}`).toContain(command);
          }
        }
        sent.push(...commands);
      }
      const commands = new Set(sent.map(([, command]) => command));
      for (const command of ["back", "forward", "reload", "stop", "home"]) {
        expect(commands, `${mode.key} has ${command}`).toContain(command);
      }
      expect(await count(opened, "#url"), `${mode.key} has one address field`).toBe(1);
      // Menu buttons name menus the mode has (ui/chrome/<mode>/menus.txt).
      for (const [, command, arg] of sent) {
        if (command === "menu") expect(mode.menus, `${mode.key}`).toContain(arg.split(",")[0]);
      }
      // The way to the Browser UI menu: Lantern's Retro menu or a menu holding it.
      const toUiMenu = sent.some(([, command, arg]) =>
        command === "retro-menu" || (command === "menu" && mode.uiMenus.includes(arg.split(",")[0])));
      expect(toUiMenu, `${mode.key} reaches the Browser UI menu`).toBe(true);
      if (mode.statusOpen > 0) expect(commands, `${mode.key} opens its panel`).toContain("panel");
      // Status text and a progress meter, somewhere.
      expect(await count(opened, "#msg")).toBe(1);
      expect(await count(opened, "#meter")).toBe(1);
      expect(opened.flatMap(({ errors }) => errors)).toEqual([]);
    });

    test("each control sends its command", async ({ browser }) => {
      for (const view of viewsOf(mode)) {
        const { page, errors } = await open(browser, mode, view);
        let seq = 1;
        for (const [selector, command, arg] of await commandsOf(page)) {
          if (!(await page.locator(selector).isVisible())) continue;
          await page.click(selector);
          seq += 1;
          await expect(page).toHaveTitle(`lantern|${seq}|${command}|${encodeURIComponent(arg)}`);
        }
        if ((await page.locator("#url").count()) > 0 && (await page.locator("#url").isVisible())) {
          await page.fill("#url", "rust lang|x");
          await page.press("#url", "Enter");
          seq += 1;
          await expect(page).toHaveTitle(`lantern|${seq}|go|rust%20lang%7Cx`);
        }
        // Disabled controls send nothing.
        await page.evaluate(() => window.lantern.update({ canBack: false, canFwd: false, loading: false }));
        const before = await page.title();
        for (const id of ["back", "forward", "stop"]) {
          const el = page.locator(`#${id}`);
          if ((await el.count()) > 0 && (await el.isVisible())) await el.click();
        }
        expect(await page.title()).toBe(before);
        expect(errors).toEqual([]);
      }
    });

    test("state from lantern.update shows", async ({ browser }) => {
      const opened = await openAll(browser, mode);
      for (const { page } of opened) {
        if ((await page.locator("#url").count()) > 0) {
          await expect(page.locator("#url")).toHaveValue(TOOLBAR_STATE.url);
        }
        if ((await page.locator("#title").count()) > 0) {
          // Titles are text, never markup.
          await expect(page.locator("#title")).toHaveText(TOOLBAR_STATE.title);
          await expect(page.locator("#title b")).toHaveCount(0);
        }
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
      for (const { view, page } of opened) {
        await page.mouse.move(0, 0);
        await expect(page).toHaveScreenshot(`${view}-${mode.key}.png`);
      }
      for (const { view, page } of opened) {
        await page.evaluate(() => {
          window.lantern.update({ url: "", title: "", canBack: false, canFwd: false, loading: false, secure: false });
          window.lantern.update({ text: "Done.", progress: 1, loading: false, modem: "", secure: false, canBack: false, canFwd: false });
        });
        await expect(page).toHaveScreenshot(`${view}-${mode.key}-idle.png`);
      }
    });

    // The start page in this mode's look (ui/chrome/<mode>/home.css), at
    // the size the mode leaves for the page.
    if (mode.key !== "lantern") {
      test("its start page fits and matches the golden screenshot", async ({ browser }) => {
        const width = 800 - mode.sideWidth;
        const height = 600 - mode.toolbarHeight - mode.statusHeight - mode.frameHeight;
        const page = await browser.newPage({ viewport: { width, height } });
        const errors = trackErrors(page);
        await page.goto(pageUrl(`home-${mode.key}.html`));
        await expect(page.locator("#q")).toBeVisible();
        await expect(page.locator("ul.marks a")).toHaveCount(3);
        const overflow = await page.evaluate(() => document.documentElement.scrollWidth - innerWidth);
        expect(overflow, "no horizontal scroll").toBeLessThanOrEqual(0);
        await expect(page).toHaveScreenshot(`home-${mode.key}.png`);
        expect(errors).toEqual([]);
      });
    }

    // The panel the status view grows into: a strip under the page, or the
    // whole window (panelFull, the page hidden).
    if (mode.statusOpen > 0) {
      test("its open panel matches the golden screenshot", async ({ browser }) => {
        const height = mode.panelFull ? 600 - mode.toolbarHeight : mode.statusOpen;
        const { page, errors } = await open(browser, mode, ["status", 800, height]);
        await page.evaluate(() => window.lantern.update({ panel: true }));
        await expect(page.locator("html")).toHaveClass(/panel-open/);
        await page.mouse.move(0, 0);
        await expect(page).toHaveScreenshot(`status-${mode.key}-panel.png`);
        expect(errors).toEqual([]);
      });
    }
  });
}
