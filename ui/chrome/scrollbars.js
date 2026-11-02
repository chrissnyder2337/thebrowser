// Puts a UI mode's scrollbars (ui/chrome/scrollbars/<style>.css) on a page in
// a content frame, whatever the page's own styles: scrollbars are the
// browser's, not the site's. The sheet goes last, so its !important rules
// beat the page's. retro.rs runs this when a frame starts and finishes
// loading and when the mode changes; empty rules take the sheet out.
(function (css) {
  "use strict";
  var KEY = "__lanternScrollbars";
  var doc = document;
  // Kept per document: a window reused for a new document can't adopt a
  // sheet made for the old one.
  var sheet = doc[KEY] || (doc[KEY] = new CSSStyleSheet());
  var others = doc.adoptedStyleSheets.filter(function (s) { return s !== sheet; });
  if (!css) {
    doc.adoptedStyleSheets = others;
    return;
  }
  sheet.replaceSync(css);
  doc.adoptedStyleSheets = others.concat([sheet]);
})
