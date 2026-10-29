// Puts a UI mode's browser defaults (ui/chrome/<mode>/pages.css) under a
// plain page in a content frame: one with no style sheets of its own, which
// is when an old browser's defaults showed. Pages with style sheets keep
// their look. The page color (`background`) only goes on the top page, and
// only when its html and body set no background. Every rule is wrapped in
// :where(), so a style attribute still wins. retro.rs runs this once a frame
// has loaded and when the settings change; empty rules take the sheet out.
(function (css, background) {
  "use strict";
  var KEY = "__lanternModePages";
  var doc = document;
  // Kept per document: a window reused for a new document can't adopt a
  // sheet made for the old one.
  var sheet = doc[KEY] || (doc[KEY] = new CSSStyleSheet());
  var others = doc.adoptedStyleSheets.filter(function (s) { return s !== sheet; });
  doc.adoptedStyleSheets = others;
  if (!css || doc.styleSheets.length > 0) return;

  function bare(el) {
    if (!el) return true;
    var style = getComputedStyle(el);
    return style.backgroundImage === "none" &&
      (style.backgroundColor === "transparent" || style.backgroundColor === "rgba(0, 0, 0, 0)");
  }
  var colored = window === window.top && bare(doc.documentElement) && bare(doc.body);
  sheet.replaceSync(colored ? css + "\n" + background : css);
  // First, so a retro look's sheets (adopted after it) win any tie.
  doc.adoptedStyleSheets = [sheet].concat(others);
})
