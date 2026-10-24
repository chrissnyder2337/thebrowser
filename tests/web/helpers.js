// Shared helpers for the web-layer tests: fixture serving, the pages written
// by `cargo run --example dump-pages`, and checks used by several specs.
const fs = require("fs");
const path = require("path");
const { pathToFileURL } = require("url");

const ROOT = path.resolve(__dirname, "../..");
const FIXTURES = path.join(ROOT, "tests/fixtures");
const PAGES = path.resolve(ROOT, process.env.LANTERN_PAGES || "target/pages");
const FIXTURE_ORIGIN = "http://fixtures.lantern.test";

const LOOKS = ["geocities", "early-web", "portal"];
const FIXTURE_PAGES = ["tables", "forms", "floats", "shadow-dom", "iframe", "long"];

function pageFile(name) {
  const file = path.join(PAGES, name);
  if (!fs.existsSync(file)) {
    throw new Error(`${file} is missing: run \`cargo run --example dump-pages -- ${PAGES}\` first`);
  }
  return file;
}

function pageUrl(name) {
  return pathToFileURL(pageFile(name)).href;
}

/** The script Lantern runs in content frames for `look` (from retro.rs). */
function retroScript(look) {
  return fs.readFileSync(pageFile(`retro-${look}.js`), "utf8");
}

/**
 * Serve tests/fixtures at FIXTURE_ORIGIN. A fixed Last-Modified keeps the
 * early-web look's "Last updated" line stable.
 */
async function serveFixtures(context) {
  await context.route(`${FIXTURE_ORIGIN}/**`, async (route) => {
    const name = new URL(route.request().url()).pathname.replace(/^\/+/, "");
    const file = path.join(FIXTURES, name);
    if (!file.startsWith(FIXTURES + path.sep) || !fs.existsSync(file)) {
      return route.fulfill({ status: 404, body: "not found" });
    }
    return route.fulfill({
      status: 200,
      contentType: "text/html; charset=utf-8",
      headers: { "last-modified": "Sat, 01 Aug 1998 12:00:00 GMT" },
      body: fs.readFileSync(file),
    });
  });
}

/** Collect uncaught exceptions and console errors from every frame. */
function trackErrors(page) {
  const errors = [];
  page.on("pageerror", (err) => errors.push(`pageerror: ${err.message}`));
  page.on("console", (msg) => {
    if (msg.type() === "error") errors.push(`console: ${msg.text()}`);
  });
  return errors;
}

/**
 * Open a fixture with `look` applied the way Lantern does: when each frame
 * starts loading (init script) and again when it has loaded.
 */
async function openWithLook(page, fixture, look) {
  await freezeAnimations(page);
  const script = retroScript(look);
  await page.addInitScript({ content: script });
  await page.goto(`${FIXTURE_ORIGIN}/${fixture}.html`, { waitUntil: "load" });
  for (const frame of page.frames()) await frame.evaluate(script);
  await settle(page);
}

/**
 * Hold every animation at its start, including those in the looks' closed
 * shadow roots, which Playwright's `animations: "disabled"` can't reach.
 */
async function freezeAnimations(page) {
  if (page.__lanternFrozen) return;
  const cdp = await page.context().newCDPSession(page);
  await cdp.send("Animation.enable");
  await cdp.send("Animation.setPlaybackRate", { playbackRate: 0 });
  page.__lanternFrozen = true;
}

/** Wait past the runtime's rescan delay (100 ms while a page is new). */
async function settle(page) {
  await page.evaluate(() => new Promise((done) => setTimeout(done, 350)));
}

module.exports = {
  FIXTURE_ORIGIN,
  FIXTURE_PAGES,
  LOOKS,
  openWithLook,
  pageUrl,
  retroScript,
  serveFixtures,
  settle,
  trackErrors,
};
