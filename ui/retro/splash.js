// A Flash-style splash intro, circa 1999: the site's name zooms in over a
// spinning ring while a "LOADING..." bar fills, then an ENTER button pulses.
// SKIP INTRO is always there. Rust runs this as `(<this file>)("site name")`
// in the main frame when a load starts on a site the intro hasn't been shown
// for, so it covers the page while it loads. `window.__lanternSplash.remove()`
// takes it away.
(function (name) {
  "use strict";
  if (window.__lanternSplash) return;
  var FONT = "Verdana, Arial, Helvetica, sans-serif";
  var css = [
    ".intro { position: fixed; inset: 0; z-index: 2147483647; background: #000; color: #fff;",
    "  display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 22px;",
    "  font: 12px " + FONT + "; overflow: hidden; transition: opacity 0.4s; }",
    ".intro.out { opacity: 0; }",
    ".ring { position: absolute; width: 340px; height: 340px; border-radius: 50%; opacity: 0.55;",
    "  background: conic-gradient(#00ccff, transparent 30%, #ff00cc, transparent 70%, #00ccff);",
    "  -webkit-mask: radial-gradient(circle, transparent 62%, #000 63%, #000 66%, transparent 67%);",
    "  mask: radial-gradient(circle, transparent 62%, #000 63%, #000 66%, transparent 67%);",
    "  animation: spin 2.4s linear infinite; }",
    ".name { position: relative; max-width: 80vw; text-align: center; overflow-wrap: anywhere;",
    "  font: bold 44px " + FONT + "; letter-spacing: 6px; text-transform: uppercase;",
    "  background: linear-gradient(90deg, #00ccff, #ffffff, #ff00cc); -webkit-background-clip: text;",
    "  background-clip: text; color: transparent; animation: zoom 1.6s cubic-bezier(.2,.9,.3,1.2) both; }",
    ".tag { position: relative; letter-spacing: 8px; color: #8899aa; animation: fade 1s 1s both; }",
    ".load { position: relative; width: 240px; animation: fade 0.6s 0.4s both; }",
    ".load span { display: block; margin-bottom: 4px; letter-spacing: 3px; color: #00ccff; }",
    ".bar { height: 8px; border: 1px solid #00ccff; padding: 1px; }",
    ".bar i { display: block; height: 100%; width: 0; background: #00ccff; }",
    "button { font: bold 12px " + FONT + "; letter-spacing: 3px; color: #fff; background: transparent;",
    "  border: 1px solid #fff; padding: 8px 22px; cursor: pointer; }",
    "button:hover { background: #fff; color: #000; }",
    ".enter { position: relative; visibility: hidden; animation: pulse 1s ease-in-out infinite alternate; }",
    ".enter.ready { visibility: visible; }",
    ".skip { position: absolute; right: 16px; bottom: 16px; padding: 4px 10px; font-size: 10px;",
    "  border-color: #666; color: #aaa; }",
    "@keyframes spin { to { transform: rotate(360deg); } }",
    "@keyframes zoom { from { transform: scale(6) rotate(-8deg); opacity: 0; filter: blur(8px); } }",
    "@keyframes fade { from { opacity: 0; } }",
    "@keyframes pulse { to { box-shadow: 0 0 18px #00ccff; border-color: #00ccff; } }"
  ].join("\n");

  var host = document.createElement("lantern-splash");
  // Inline !important beats any page rule that matches the host element.
  host.style.setProperty("all", "initial", "important");
  host.style.setProperty("display", "block", "important");
  var root = host.attachShadow({ mode: "closed" });
  var sheet = new CSSStyleSheet();
  sheet.replaceSync(css);
  root.adoptedStyleSheets = [sheet];

  function el(tag, cls, text) {
    var e = document.createElement(tag);
    if (cls) e.className = cls;
    if (text) e.textContent = text;
    return e;
  }

  var intro = el("div", "intro");
  var load = el("div", "load");
  var label = el("span", "", "LOADING... 0%");
  var bar = el("div", "bar");
  var fill = el("i");
  bar.appendChild(fill);
  load.appendChild(label);
  load.appendChild(bar);
  var enter = el("button", "enter", "ENTER SITE");
  var skip = el("button", "skip", "SKIP INTRO >>");
  intro.appendChild(el("div", "ring"));
  intro.appendChild(el("div", "name", name));
  intro.appendChild(el("div", "tag", "WELCOME TO THE FUTURE"));
  intro.appendChild(load);
  intro.appendChild(enter);
  intro.appendChild(skip);
  root.appendChild(intro);

  var timer = 0;
  function remove() {
    clearInterval(timer);
    window.__lanternSplash = null;
    intro.classList.add("out");
    setTimeout(function () { host.remove(); }, 400);
  }
  enter.addEventListener("click", remove);
  skip.addEventListener("click", remove);

  // The bar takes about three seconds, whatever the page is really doing.
  var percent = 0;
  timer = setInterval(function () {
    percent = Math.min(100, percent + 1 + Math.floor(Math.random() * 6));
    label.textContent = (percent < 100 ? "LOADING... " : "LOADED ") + percent + "%";
    fill.style.width = percent + "%";
    if (percent >= 100) {
      clearInterval(timer);
      enter.classList.add("ready");
    }
  }, 90);

  window.__lanternSplash = { remove: remove };
  // The new document may not have its root element yet when a load starts.
  (function attach() {
    if (!window.__lanternSplash) return;
    if (document.documentElement) document.documentElement.appendChild(host);
    else setTimeout(attach, 10);
  })();
})
