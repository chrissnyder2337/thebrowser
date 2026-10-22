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
    // Everything except SVG internals and icon-font glyphs, which would turn
    // into their ligature names if their font were swapped.
    // :where() keeps this at the specificity of `body *`, so the rules below win.
    var ALL = 'body, body :where(:not(svg *):not(i):not([class*="icon" i]):not([class*="fa-"]))';

    var pageCss = [
      "html { background-color: #000018 !important; background-image: " + STARS + " !important;",
      "  background-size: 200px 200px !important; background-repeat: repeat !important;",
      "  background-attachment: fixed !important; padding-bottom: 30px !important; }",
      ALL + " { color: #00ff00 !important; background-color: transparent !important;",
      "  font-family: " + FONT + " !important; text-shadow: none !important; border-color: #ff00ff !important; }",
      "html body { background: transparent !important; }",
      "html body a, html body a * { color: #00ffff !important; text-decoration: underline !important; }",
      "html body a:visited, html body a:visited * { color: #ff80ff !important; }",
      "html body a:hover, html body a:hover * { color: #ffff00 !important; animation: lgc-blink 0.5s steps(1) infinite; }",
      "html body :is(h1, h2, h3) { background-image: " + RAINBOW + " !important; background-size: 200% 100% !important;",
      "  -webkit-background-clip: text !important; background-clip: text !important;",
      "  color: transparent !important; -webkit-text-fill-color: transparent;",
      "  text-align: center !important; animation: lgc-slide 3s linear infinite; }",
      "html body h1 { font-size: 2.6em !important; }",
      "html body :is(h4, h5, h6, b, strong, th) { color: #ffff00 !important; }",
      "html body :is(em, i) { color: #ff80ff !important; }",
      "html body :is(img, video, canvas, iframe) { border: 3px ridge #ffff00 !important; }",
      "html body hr { height: 6px !important; border: 0 !important; background: " + RAINBOW + " !important; }",
      "html body :is(table, td, th) { border: 2px outset #c0c0c0 !important; }",
      "html body :is(button, input, select, textarea) { color: #000 !important; background-color: #c0c0c0 !important;",
      "  border: 2px outset #ffffff !important; font-family: \"Times New Roman\", Times, serif !important; }",
      "html body :is(input:not([type]), input[type=text], input[type=search], input[type=email], input[type=url], textarea) {",
      "  background-color: #ffffff !important; border-style: inset !important; }",
      "::selection { color: #000 !important; background: #ff00ff !important; }",
      "@keyframes lgc-blink { 50% { visibility: hidden; } }",
      "@keyframes lgc-slide { to { background-position: -200% 0; } }"
    ].join("\n");

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
        var sheets = document.adoptedStyleSheets.filter(function (s) { return s !== pageSheet; });
        document.adoptedStyleSheets = enable ? sheets.concat([pageSheet]) : sheets;
        // The banner and cursor trail only go on the top-level page, not in iframes.
        if (enable && window === window.top && document.documentElement) decorate();
        else undecorate();
      }
    };
  }
  self.set(on);
})
