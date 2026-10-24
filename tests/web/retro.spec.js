// The retro looks (ui/retro/) on the fixture pages, injected the way
// src/shell/retro.rs runs them. Past bugs these pin down: footers that ended
// up mid-page, unreadable button labels, see-through floating boxes, and
// nodes added after the first scan staying unstyled.
const { test, expect } = require("@playwright/test");
const {
  FIXTURE_PAGES,
  LOOKS,
  openWithLook,
  serveFixtures,
  settle,
  trackErrors,
} = require("./helpers");

// Decoration hosts each look appends at the end of the page.
const FOOTERS = { "early-web": "lantern-early-web-footer", portal: "lantern-portal-footer" };
// Looks that keep floating boxes solid (`solid` in the look definition).
const SOLID_LOOKS = ["geocities", "portal"];

test.beforeEach(async ({ context }) => {
  await serveFixtures(context);
});

/** In-page: contrast ratio of each button label against what's behind it. */
function buttonContrast() {
  function rgba(text) {
    const n = (text.match(/[\d.]+/g) || []).map(Number);
    return { r: n[0] || 0, g: n[1] || 0, b: n[2] || 0, a: n.length > 3 ? n[3] : 1 };
  }
  function luminance(c) {
    const f = (v) => {
      v /= 255;
      return v <= 0.03928 ? v / 12.92 : Math.pow((v + 0.055) / 1.055, 2.4);
    };
    return 0.2126 * f(c.r) + 0.7152 * f(c.g) + 0.0722 * f(c.b);
  }
  // First opaque background up the tree (through shadow hosts); else the
  // page's own background, else white.
  function background(el) {
    for (let e = el; e; e = e.parentElement || (e.getRootNode().host || null)) {
      const c = rgba(getComputedStyle(e).backgroundColor);
      if (c.a > 0.5) return c;
    }
    return { r: 255, g: 255, b: 255, a: 1 };
  }
  const buttons = [];
  (function collect(scope) {
    scope.querySelectorAll("button, input[type=submit], input[type=button]").forEach((b) => buttons.push(b));
    scope.querySelectorAll("*").forEach((e) => e.shadowRoot && collect(e.shadowRoot));
  })(document);
  return buttons.map((b) => {
    // The label is the button's own text, or its first text-bearing child.
    let label = b;
    for (const child of b.querySelectorAll("span")) {
      if (child.textContent.trim()) {
        label = child;
        break;
      }
    }
    const fg = luminance(rgba(getComputedStyle(label).color));
    const bg = luminance(background(b));
    const ratio = (Math.max(fg, bg) + 0.05) / (Math.min(fg, bg) + 0.05);
    return { label: (b.textContent || b.value).trim(), ratio: Math.round(ratio * 100) / 100 };
  });
}

/** In-page: every [data-test-float] box, in the document and open shadow roots. */
function floatingBoxes() {
  const out = [];
  (function collect(scope) {
    scope.querySelectorAll("[data-test-float]").forEach((e) => {
      const s = getComputedStyle(e);
      const n = (s.backgroundColor.match(/[\d.]+/g) || []).map(Number);
      out.push({
        id: e.id || e.textContent.trim().slice(0, 30),
        position: s.position,
        alpha: n.length === 0 ? 0 : n.length > 3 ? n[3] : 1,
        solid: e.hasAttribute("data-lantern-retro-solid"),
      });
    });
    scope.querySelectorAll("*").forEach((e) => e.shadowRoot && collect(e.shadowRoot));
  })(document);
  return out;
}

for (const look of LOOKS) {
  test.describe(`look ${look}`, () => {
    for (const fixture of FIXTURE_PAGES) {
      test(`${fixture}: no errors, readable buttons, golden`, async ({ page }) => {
        const errors = trackErrors(page);
        await openWithLook(page, fixture, look);
        expect(await page.evaluate(() => !!window.__lanternRetro)).toBe(true);
        for (const { label, ratio } of await page.evaluate(buttonContrast)) {
          expect(ratio, `contrast of button "${label}"`).toBeGreaterThanOrEqual(3);
        }
        for (const frame of page.frames()) {
          for (const { label, ratio } of await frame.evaluate(buttonContrast)) {
            expect(ratio, `contrast of framed button "${label}"`).toBeGreaterThanOrEqual(3);
          }
        }
        await expect(page).toHaveScreenshot(`${look}-${fixture}.png`);
        expect(errors).toEqual([]);
      });
    }

    test("footer stays at the end of the page", async ({ page }) => {
      const errors = trackErrors(page);
      await openWithLook(page, "long", look);
      // Content arriving after the look started (a page still loading) must
      // still end up above the look's footer.
      await page.evaluate(() => {
        const p = document.createElement("p");
        p.id = "late";
        p.textContent = "Added after the look started.";
        document.body.appendChild(p);
      });
      await settle(page);
      const footer = FOOTERS[look];
      if (footer) {
        const last = await page.evaluate(() => document.body.lastElementChild.localName);
        expect(last).toBe(footer);
        const order = await page.evaluate((tag) => {
          const host = document.querySelector(tag).getBoundingClientRect();
          const own = document.getElementById("page-footer").getBoundingClientRect();
          const late = document.getElementById("late").getBoundingClientRect();
          return { host: host.top, own: own.bottom, late: late.bottom };
        }, footer);
        expect(order.host).toBeGreaterThanOrEqual(order.own);
        expect(order.host).toBeGreaterThanOrEqual(order.late);
      } else {
        // GeoCities pins its bar to the bottom of the window; the page is
        // padded so its own footer can still scroll clear of it.
        // Without the page's own bottom margins, only the look's padding
        // keeps the last line clear of the bar.
        await page.evaluate(() => {
          document.body.style.setProperty("padding-bottom", "0", "important");
          document.body.style.setProperty("margin-bottom", "0", "important");
          document.getElementById("late").style.setProperty("margin", "0", "important");
          window.scrollTo(0, document.documentElement.scrollHeight);
        });
        const { bottom, height } = await page.evaluate(() => ({
          bottom: document.getElementById("late").getBoundingClientRect().bottom,
          height: window.innerHeight,
        }));
        expect(bottom).toBeLessThanOrEqual(height - 26);
      }
      expect(errors).toEqual([]);
    });

    test("floating boxes stay opaque", async ({ page }) => {
      await openWithLook(page, "floats", look);
      await openWithLook(page, "floats", look);
      const boxes = await page.evaluate(floatingBoxes);
      expect(boxes.length).toBe(4);
      for (const box of boxes) {
        if (SOLID_LOOKS.includes(look)) {
          expect(box.solid, `${box.id} marked solid`).toBe(true);
          expect(box.alpha, `${box.id} background alpha`).toBe(1);
        } else {
          // With the page's styles off nothing floats over the text.
          expect(box.position, `${box.id} position`).toBe("static");
        }
      }
      // A box that was see-through to begin with isn't made solid.
      expect(await page.evaluate(() => document.getElementById("ghost").hasAttribute("data-lantern-retro-solid"))).toBe(false);
      const shadowBoxes = await (async () => {
        await page.goto("about:blank");
        await openWithLook(page, "shadow-dom", look);
        return page.evaluate(floatingBoxes);
      })();
      expect(shadowBoxes.length).toBe(1);
      if (SOLID_LOOKS.includes(look)) expect(shadowBoxes[0].solid).toBe(true);
    });

    test("a rescan restyles new nodes", async ({ page }) => {
      const errors = trackErrors(page);
      await openWithLook(page, "tables", look);
      const before = await page.evaluate(() => getComputedStyle(document.querySelector("h1")).fontFamily);
      await page.evaluate(() => {
        const p = document.createElement("p");
        p.id = "new-text";
        p.textContent = "A paragraph added later.";
        document.body.insertBefore(p, document.getElementById("page-footer"));
        const box = document.createElement("div");
        box.id = "new-float";
        box.textContent = "A banner added later";
        box.style.cssText = "position: fixed; top: 0; left: 0; z-index: 3; background: #333; color: #fff";
        document.body.appendChild(box);
        customElements.define("x-late", class extends HTMLElement {
          constructor() {
            super();
            this.attachShadow({ mode: "open" }).innerHTML =
              "<style>p { font-family: Georgia; color: rgb(1, 2, 3); }</style><p>Late shadow text</p>";
          }
        });
        document.body.insertBefore(document.createElement("x-late"), document.getElementById("page-footer"));
      });
      await settle(page);
      const styled = await page.evaluate(() => {
        const text = getComputedStyle(document.getElementById("new-text"));
        const late = document.querySelector("x-late").shadowRoot;
        const inner = getComputedStyle(late.querySelector("p"));
        return {
          text: text.fontFamily,
          shadowSheets: late.adoptedStyleSheets.length,
          shadow: inner.fontFamily,
          shadowColor: inner.color,
          floatSolid: document.getElementById("new-float").hasAttribute("data-lantern-retro-solid"),
        };
      });
      // New nodes get the same font the look gave the page's first heading.
      expect(styled.text).toBe(before);
      expect(styled.shadowSheets).toBe(1);
      expect(styled.shadow).not.toContain("Georgia");
      if (look !== "early-web") expect(styled.shadowColor).not.toBe("rgb(1, 2, 3)");
      if (SOLID_LOOKS.includes(look)) expect(styled.floatSolid).toBe(true);
      expect(errors).toEqual([]);
    });

    test("switching off removes everything", async ({ page }) => {
      await openWithLook(page, "shadow-dom", look);
      await page.evaluate(() => window.__lanternRetro.apply("off"));
      const left = await page.evaluate(() => ({
        sheets: document.adoptedStyleSheets.length,
        shadowSheets: [...document.querySelectorAll("x-card")].map((c) => c.shadowRoot.adoptedStyleSheets.length),
        decor: document.querySelectorAll("[data-lantern-retro-decor]").length,
        marks: document.querySelectorAll("[data-lantern-retro-solid], [data-lantern-retro-unpinned]").length,
        disabled: [...document.styleSheets].filter((s) => s.disabled).length,
      }));
      expect(left).toEqual({ sheets: 0, shadowSheets: [0, 0], decor: 0, marks: 0, disabled: 0 });
    });
  });
}
