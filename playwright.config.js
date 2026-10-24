// Playwright suite for Lantern's web layer: the retro looks on fixture pages,
// the chrome pages and the internal pages. Run it with tests/web/run.sh, which
// uses the official Playwright image pinned to the version in package.json,
// so golden screenshots render the same here and in CI.
const { defineConfig } = require("@playwright/test");

module.exports = defineConfig({
  testDir: "tests/web",
  // Goldens live in tests/golden/<spec>/<name>.png. Existing ones are part of
  // the oracle (see the steward skill); only new ones may be added.
  snapshotPathTemplate: "tests/golden/{testFileName}/{arg}{ext}",
  forbidOnly: !!process.env.CI,
  retries: 0,
  reporter: process.env.CI ? [["list"], ["html", { open: "never" }]] : "list",
  expect: {
    toHaveScreenshot: {
      animations: "disabled",
      caret: "hide",
      // A small tolerance for antialiasing; layout or color changes exceed it.
      maxDiffPixelRatio: 0.002,
      threshold: 0.2,
    },
  },
  use: {
    browserName: "chromium",
    viewport: { width: 800, height: 600 },
    deviceScaleFactor: 1,
    locale: "en-US",
    timezoneId: "UTC",
  },
});
