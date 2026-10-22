// GeoCities mode: restyles the page like a 1998 personal homepage. Rust runs
// this as `(<this file>)(true|false)` in every frame of the content view, after
// each load and whenever the option is toggled, so it must be idempotent.
//
// Styles go in constructed stylesheets (adoptedStyleSheets), which a page's
// Content-Security-Policy doesn't block the way it blocks <style> elements, and
// they use only gradients and text, so no image a CSP could refuse is needed.
(function (on) {
  "use strict";
  var KEY = "__lanternGeocities";
  var self = window[KEY];
  if (!self) {
    var FONT = '"Comic Sans MS", "Comic Neue", "Chalkboard SE", "Comic Relief", cursive';
    var RAINBOW = "linear-gradient(90deg, #ff0000, #ff8000, #ffff00, #00ff00, #00ffff, #0080ff, #ff00ff, #ff0000)";
    var STARS = [
      "radial-gradient(1px 1px at 23px 31px, #fff, transparent)",
      "radial-gradient(1px 1px at 61px 143px, #ffff80, transparent)",
      "radial-gradient(2px 2px at 97px 62px, #fff, transparent)",
      "radial-gradient(1px 1px at 131px 171px, #80ffff, transparent)",
      "radial-gradient(1px 1px at 157px 17px, #fff, transparent)",
      "radial-gradient(2px 2px at 181px 113px, #ffc0ff, transparent)",
      "radial-gradient(1px 1px at 11px 89px, #fff, transparent)"
    ].join(", ");
    // Marks elements that float over the page (menus, cookie banners, sticky
    // headers) and had a solid background, so they stay solid instead of
    // turning see-through like everything else.
    var SOLID = "data-lantern-geocities-solid";
    // Everything except SVG internals and icon-font glyphs, which would turn
    // into their ligature names if their font were swapped.
    var ANY = ':where(:not(svg *):not(i):not([class*="icon" i]):not([class*="fa-"]))';

    // The same rules serve the document and open shadow trees (custom
    // elements), which document styles don't reach. In the document `p` is
    // "html body " and the catch-all is `body *`; in a shadow tree `p` is
    // empty and the catch-all is `*`. :where() keeps the catch-all at the
    // lowest specificity so every later rule beats it.
    function css(p, all) {
      return [
        all + " { color: #00ff00 !important; background-color: transparent !important;",
        "  font-family: " + FONT + " !important; text-shadow: none !important; border-color: #ff00ff !important; }",
        p + "[" + SOLID + "] { background-color: #000080 !important; }",
        p + "a, " + p + "a * { color: #00ffff !important; text-decoration: underline !important; }",
        p + "a:visited, " + p + "a:visited * { color: #ff80ff !important; }",
        p + "a:hover, " + p + "a:hover * { color: #ffff00 !important; animation: lgc-blink 0.5s steps(1) infinite; }",
        p + ":is(h1, h2, h3) { background-image: " + RAINBOW + " !important; background-size: 200% 100% !important;",
        "  -webkit-background-clip: text !important; background-clip: text !important;",
        "  color: transparent !important; -webkit-text-fill-color: transparent;",
        "  text-align: center !important; animation: lgc-slide 3s linear infinite; }",
        // Text in nested tags must be transparent too, or it hides the rainbow.
        p + ":is(h1, h2, h3) :where(:not(svg, svg *)) { color: transparent !important;",
        "  -webkit-text-fill-color: transparent !important; }",
        p + "h1 { font-size: 2.6em !important; }",
        p + ":is(h4, h5, h6, b, strong, th) { color: #ffff00 !important; }",
        p + ":is(em, i) { color: #ff80ff !important; }",
        p + ":is(img, video, canvas, iframe) { border: 3px ridge #ffff00 !important; }",
        p + "hr { height: 6px !important; border: 0 !important; background: " + RAINBOW + " !important; }",
        p + ":is(table, td, th) { border: 2px outset #c0c0c0 !important; }",
        p + "button :where(:not(svg *)) { color: #000 !important; }",
        p + ":is(button, input, select, textarea) { color: #000 !important; background-color: #c0c0c0 !important;",
        "  border: 2px outset #ffffff !important; font-family: \"Times New Roman\", Times, serif !important; }",
        p + ":is(input:not([type]), input[type=text], input[type=search], input[type=email], input[type=url], textarea) {",
        "  background-color: #ffffff !important; border-style: inset !important; }",
        "::selection { color: #000 !important; background: #ff00ff !important; }",
        "@keyframes lgc-blink { 50% { visibility: hidden; } }",
        "@keyframes lgc-slide { to { background-position: -200% 0; } }"
      ].join("\n");
    }

    var pageCss = [
      "html { background-color: #000018 !important; background-image: " + STARS + " !important;",
      "  background-size: 200px 200px !important; background-repeat: repeat !important;",
      "  background-attachment: fixed !important; padding-bottom: 30px !important; }",
      "html body { background: transparent !important; }",
      css("html body ", "body, body " + ANY)
    ].join("\n");
    var shadowCss = css("", ANY);

    var decorCss = [
      ":host { all: initial; }",
      ".bar { position: fixed; left: 0; right: 0; bottom: 0; height: 26px; z-index: 2147483647;",
      "  display: flex; align-items: center; gap: 8px; padding: 0 6px; box-sizing: border-box;",
      "  background: #000080; border-top: 3px ridge #ffff00; pointer-events: none;",
      "  font: bold 13px " + FONT + "; color: #ffff00; }",
      ".uc { flex: none; color: #000; background: repeating-linear-gradient(45deg, #ffcc00 0 8px, #000 8px 16px);",
      "  padding: 1px 6px; }",
      ".uc span { background: #ffcc00; padding: 0 3px; animation: blink 1s steps(1) infinite; }",
      "marquee { flex: 1; min-width: 0; color: #00ffff; }",
      ".count { flex: none; display: flex; align-items: center; gap: 4px; color: #fff; font-size: 11px; }",
      ".count b { font: bold 13px \"Courier New\", monospace; color: #00ff00; background: #000;",
      "  border: 2px inset #808080; padding: 0 3px; letter-spacing: 2px; }",
      ".spark { position: fixed; z-index: 2147483647; pointer-events: none; font: 14px sans-serif;",
      "  animation: fall 0.9s ease-in forwards; }",
      "@keyframes blink { 50% { visibility: hidden; } }",
      "@keyframes fall { to { transform: translate(var(--dx), 40px) rotate(180deg); opacity: 0; } }"
    ].join("\n");

    var pageSheet = new CSSStyleSheet();
    pageSheet.replaceSync(pageCss);
    var shadowSheet = new CSSStyleSheet();
    shadowSheet.replaceSync(shadowCss);
    var decorSheet = new CSSStyleSheet();
    decorSheet.replaceSync(decorCss);

    var host = null;
    var root = null;
    var lastSpark = 0;
    var SPARKS = ["✦", "✧", "★", "✶", "*"];
    var COLORS = ["#ffff00", "#ff00ff", "#00ffff", "#ffffff", "#00ff00"];

    function el(tag, cls, text) {
      var e = document.createElement(tag);
      if (cls) e.className = cls;
      if (text) e.textContent = text;
      return e;
    }

    // A stable, plausible-looking hit count for this page.
    function visitors() {
      var text = location.hostname + location.pathname;
      var h = 1998;
      for (var i = 0; i < text.length; i++) h = (h * 31 + text.charCodeAt(i)) >>> 0;
      return ("0000000" + (h % 9000000 + 1000)).slice(-7);
    }

    function sparkle(e) {
      var now = Date.now();
      if (!root || now - lastSpark < 40 || root.querySelectorAll(".spark").length > 30) return;
      lastSpark = now;
      var s = el("span", "spark", SPARKS[Math.floor(Math.random() * SPARKS.length)]);
      s.style.left = e.clientX + 6 + "px";
      s.style.top = e.clientY + 4 + "px";
      s.style.color = COLORS[Math.floor(Math.random() * COLORS.length)];
      s.style.setProperty("--dx", Math.round(Math.random() * 30 - 15) + "px");
      s.addEventListener("animationend", function () { s.remove(); });
      root.appendChild(s);
    }

    function adopt(target, sheet, enable) {
      var sheets = target.adoptedStyleSheets.filter(function (s) { return s !== sheet; });
      target.adoptedStyleSheets = enable ? sheets.concat([sheet]) : sheets;
    }

    // Pages keep adding content (and custom elements with shadow trees), so
    // rescan after DOM changes, at most about once a second.
    var OBSERVE = { childList: true, subtree: true };
    var observer = new MutationObserver(schedule);
    var roots = [];
    var scanTimer = 0;
    var lastScan = 0;

    function schedule() {
      if (!scanTimer) scanTimer = setTimeout(scan, Math.max(250, 1000 - (Date.now() - lastScan)));
    }

    // Every element of `scope` and of the open shadow trees inside it,
    // styling and watching shadow trees not seen before.
    function walk(scope, out) {
      var all = scope.querySelectorAll("*");
      for (var i = 0; i < all.length; i++) {
        out.push(all[i]);
        var shadow = all[i].shadowRoot;
        if (shadow) {
          if (roots.indexOf(shadow) < 0) {
            roots.push(shadow);
            adopt(shadow, shadowSheet, true);
            observer.observe(shadow, OBSERVE);
          }
          walk(shadow, out);
        }
      }
      return out;
    }

    function floatsWithSolidBackground(style) {
      var floating = style.position === "fixed" || style.position === "sticky" ||
        (style.position === "absolute" && parseInt(style.zIndex, 10) > 0);
      if (!floating) return false;
      var rgba = style.backgroundColor.match(/[\d.]+/g);
      return !!rgba && (rgba.length < 4 || Number(rgba[3]) > 0.5);
    }

    function scan() {
      scanTimer = 0;
      lastScan = Date.now();
      var elements = walk(document, []);
      // Read the page's own backgrounds with our sheets switched off. Nothing
      // paints in between, so this doesn't flicker.
      pageSheet.disabled = shadowSheet.disabled = true;
      var solid = elements.map(function (e) { return floatsWithSolidBackground(getComputedStyle(e)); });
      pageSheet.disabled = shadowSheet.disabled = false;
      elements.forEach(function (e, i) {
        if (solid[i] !== e.hasAttribute(SOLID)) e.toggleAttribute(SOLID, solid[i]);
      });
    }

    function restyle(enable) {
      adopt(document, pageSheet, enable);
      if (enable) {
        observer.observe(document, OBSERVE);
        scan();
        return;
      }
      observer.disconnect();
      clearTimeout(scanTimer);
      scanTimer = 0;
      roots.forEach(function (shadow) { adopt(shadow, shadowSheet, false); });
      [document].concat(roots).forEach(function (scope) {
        scope.querySelectorAll("[" + SOLID + "]").forEach(function (e) { e.removeAttribute(SOLID); });
      });
      roots = [];
    }

    function decorate() {
      if (host && host.isConnected) return;
      host = document.createElement("lantern-geocities");
      root = host.attachShadow({ mode: "closed" });
      root.adoptedStyleSheets = [decorSheet];
      var bar = el("div", "bar");
      var uc = el("div", "uc");
      uc.appendChild(el("span", "", "UNDER CONSTRUCTION"));
      var name = document.title.trim() || location.hostname || "my homepage";
      var marquee = el("marquee", "", "★ Welcome to " + name + "!!! ★ Sign my guestbook! ★ " +
        "This page is best viewed in Lantern at 800x600 ★ Don't forget to bookmark this site! ★");
      marquee.setAttribute("scrollamount", "4");
      var count = el("div", "count", "Visitors:");
      count.appendChild(el("b", "", visitors()));
      bar.appendChild(uc);
      bar.appendChild(marquee);
      bar.appendChild(count);
      root.appendChild(bar);
      document.documentElement.appendChild(host);
      document.addEventListener("mousemove", sparkle, true);
    }

    function undecorate() {
      document.removeEventListener("mousemove", sparkle, true);
      if (host) host.remove();
      host = root = null;
    }

    self = window[KEY] = {
      set: function (enable) {
        restyle(enable);
        // The banner and cursor trail only go on the top-level page, not in iframes.
        if (enable && window === window.top && document.documentElement) decorate();
        else undecorate();
      }
    };
  }
  self.set(on);
})
