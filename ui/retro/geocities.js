// GeoCities homepage, 1998: starfield, lime Comic Sans, rainbow headings,
// blinking links, and a bottom bar with an UNDER CONSTRUCTION sign, a welcome
// marquee and a hit counter, plus a sparkle trail behind the mouse.
// A look for runtime.js; it uses only gradients and text, so no image a CSP
// could refuse is needed.
(function (R) {
  "use strict";
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

  function css(p, all) {
    return [
      all + " { color: #00ff00 !important; background-color: transparent !important;",
      "  font-family: " + FONT + " !important; text-shadow: none !important; border-color: #ff00ff !important; }",
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

  var decorSheet = R.sheet([
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
  ].join("\n"));

  var decor = null;
  var lastSpark = 0;
  var SPARKS = ["✦", "✧", "★", "✶", "*"];
  var COLORS = ["#ffff00", "#ff00ff", "#00ffff", "#ffffff", "#00ff00"];

  function sparkle(e) {
    var now = Date.now();
    if (!decor || now - lastSpark < 40 || decor.root.querySelectorAll(".spark").length > 30) return;
    lastSpark = now;
    var s = R.el("span", "spark", SPARKS[Math.floor(Math.random() * SPARKS.length)]);
    s.style.left = e.clientX + 6 + "px";
    s.style.top = e.clientY + 4 + "px";
    s.style.color = COLORS[Math.floor(Math.random() * COLORS.length)];
    s.style.setProperty("--dx", Math.round(Math.random() * 30 - 15) + "px");
    s.addEventListener("animationend", function () { s.remove(); });
    decor.root.appendChild(s);
  }

  return {
    solid: "#000080",
    pageCss: [
      "html { background-color: #000018 !important; background-image: " + STARS + " !important;",
      "  background-size: 200px 200px !important; background-repeat: repeat !important;",
      "  background-attachment: fixed !important; padding-bottom: 30px !important; }",
      "html body { background: transparent !important; }"
    ].join("\n"),
    css: css,
    decorate: function () {
      if (decor && decor.host.isConnected) return;
      decor = R.overlay("lantern-geocities", decorSheet);
      var bar = R.el("div", "bar");
      var uc = R.el("div", "uc");
      uc.appendChild(R.el("span", "", "UNDER CONSTRUCTION"));
      var name = document.title.trim() || location.hostname || "my homepage";
      var marquee = R.el("marquee", "", "★ Welcome to " + name + "!!! ★ Sign my guestbook! ★ " +
        "This page is best viewed in Lantern at 800x600 ★ Don't forget to bookmark this site! ★");
      marquee.setAttribute("scrollamount", "4");
      var count = R.el("div", "count", "Visitors:");
      count.appendChild(R.el("b", "", R.visitors()));
      bar.appendChild(uc);
      bar.appendChild(marquee);
      bar.appendChild(count);
      decor.root.appendChild(bar);
      document.documentElement.appendChild(decor.host);
      document.addEventListener("mousemove", sparkle, true);
    },
    undecorate: function () {
      document.removeEventListener("mousemove", sparkle, true);
      if (decor) decor.host.remove();
      decor = null;
    }
  };
})
