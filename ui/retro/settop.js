// A set-top TV web box, 1996: the page is reflowed into one column that fits a
// 544-pixel television screen, in large sans-serif type with wide line
// spacing. Where the page sets no colors it is black on a grey-blue screen
// with dark blue links that aren't underlined; the selected link (the first
// one until the reader moves) gets a yellow rounded box instead of changing
// color. Fields are big and rounded, buttons are grey-blue lozenges. The page's
// own stylesheets stay on. A look for runtime.js.
(function (R) {
  "use strict";

  // Marks the selected link.
  var CURRENT = "data-lantern-retro-current";
  var SANS = "Helvetica, Arial, \"Liberation Sans\", sans-serif";
  var MONO = "\"Courier New\", \"Liberation Mono\", monospace";
  var SCREEN = "#c8c8d8";
  var LINK = "#10107a";
  var LOZENGE = "#8f95bd";
  var EDGE = "#2a2a44";
  var BODY = 19;

  // The yellow selection box: a 3px outline that follows the rounded corners,
  // with a 1px dark edge inside it.
  var SELECTED = "outline: 3px solid #ffff00 !important; outline-offset: 1px !important;" +
    " border-radius: 6px !important; box-shadow: 0 0 0 1px #202020 !important;";

  function css(p, all) {
    var buttons = ":is(button, input:is([type=submit], [type=button], [type=reset], [type=file]))";
    var fields = ":is(input:not([type]), input:is([type=text], [type=search], [type=email], [type=url]," +
      " [type=tel], [type=password], [type=number], [type=date], [type=time]), textarea, select)";
    var heads = ":is(h1, h2, h3, h4, h5, h6)";
    return [
      // One column that fits the screen: nothing wider than it, no floats,
      // no minimum widths, flex rows wrap and grids collapse to one track.
      all + " { font-family: " + SANS + " !important; font-size: inherit !important; line-height: 1.5 !important;" +
        " max-width: 100% !important; min-width: 0 !important; float: none !important;" +
        " letter-spacing: normal !important; text-shadow: none !important; overflow-wrap: anywhere;" +
        " flex-wrap: wrap !important; grid-template-columns: minmax(0, 1fr) !important; }",
      p + "{ font-size: " + BODY + "px !important; }",
      // Colors only where the page set none (:where() has no specificity, so
      // any page rule beats these).
      ":where(a:link, a:visited) { color: " + LINK + "; }",
      p + ":is(a:link, a:visited, a:hover, a:active, a:focus) { text-decoration: none !important; }",
      // Pictures scale down to fit; the box doesn't draw tiny type.
      p + ":is(img, video, canvas, iframe, object, embed) { max-width: 100% !important; }",
      p + ":is(img, video) { height: auto !important; }",
      p + "svg:not([width]) { width: 1.25em; height: 1.25em; }",
      p + ":is(small, sub, sup, figcaption, caption, label) { font-size: 16px !important; }",
      p + ":is(code, kbd, samp, tt, pre) { font-family: " + MONO + " !important; font-size: 16px !important; line-height: 1.4 !important; }",
      p + "pre { white-space: pre-wrap !important; overflow-wrap: anywhere !important; }",
      p + "p, " + p + "ul, " + p + "ol, " + p + "dl, " + p + "blockquote, " + p + "pre { margin-top: 0 !important; margin-bottom: 14px !important; }",
      p + ":is(ul, ol) { padding-left: 1.4em !important; }",
      p + "blockquote { margin-left: 12px !important; margin-right: 0 !important; }",
      // Headings: big and bold, one flat step down per level.
      p + heads + " { font-weight: bold !important; line-height: 1.25 !important; margin: 18px 0 10px !important; }",
      p + heads + " *:not(svg *) { font-size: inherit !important; font-weight: inherit !important; }",
      p + "h1 { font-size: 32px !important; }",
      p + "h2 { font-size: 27px !important; }",
      p + "h3 { font-size: 23px !important; }",
      p + ":is(h4, h5, h6) { font-size: 20px !important; }",
      // Tables stay tables, squeezed to the screen.
      p + "table { max-width: 100% !important; table-layout: auto; }",
      p + ":is(td, th) { padding: 3px 5px; }",
      p + "hr { border: 0 !important; border-top: 2px solid " + EDGE + " !important; height: 0 !important; margin: 16px 0 !important; }",
      // The selected link. The first link stays selected until the reader
      // hovers or tabs to another one.
      p + "a:is(:hover, :focus, :focus-visible, :active) { " + SELECTED + " }",
      "html body:not(:has(a:is(:hover, :focus))) a[" + CURRENT + "] { " + SELECTED + " }",
      p + ":is(button, input, textarea, select, summary, [tabindex]):is(:focus, :focus-visible) { " +
        "outline: 3px solid #ffff00 !important; outline-offset: 1px !important; box-shadow: 0 0 0 1px #202020 !important; }",
      // Fields: large, rounded, white.
      p + fields + " { font-family: " + SANS + " !important; font-size: " + BODY + "px !important; line-height: 1.3 !important;" +
        " color: #000 !important; background: #ffffff !important; border: 2px solid " + EDGE + " !important;" +
        " border-radius: 10px !important; padding: 8px 12px !important; box-sizing: border-box !important;" +
        " box-shadow: none; }",
      p + ":is(input:not([type=checkbox], [type=radio]), textarea, select, button)" +
        " { min-height: 44px; }",
      p + ":is(input[type=checkbox], input[type=radio]) { width: 26px !important; height: 26px !important;" +
        " accent-color: " + EDGE + "; vertical-align: middle; }",
      // Buttons: bold grey-blue lozenges.
      p + buttons + " { font-family: " + SANS + " !important; font-size: " + BODY + "px !important; font-weight: bold !important;" +
        " color: #000 !important; background: " + LOZENGE + " !important; border: 2px solid " + EDGE + " !important;" +
        " border-radius: 22px !important; padding: 8px 22px !important; box-shadow: none; cursor: pointer;" +
        " text-shadow: none !important; }",
      p + "button *" + " { color: inherit !important; font-weight: inherit !important; }",
      p + "::placeholder { color: #5a5a6e !important; }"
    ].join("\n");
  }

  return {
    noPageStyles: false,
    solid: SCREEN,
    unpinHeaders: true,
    pageCss: [
      // The screen: a 544px column in the middle of the window. :where()
      // gives the colors no weight, so a page's own always win.
      ":where(html, body) { background-color: " + SCREEN + "; color: #000; }",
      "html body { width: auto !important; min-width: 0 !important; max-width: 544px !important;" +
        " margin: 0 auto !important; padding: 12px 16px 24px !important; box-sizing: border-box !important;" +
        " overflow-x: hidden; }"
    ].join("\n"),
    css: css,
    decorate: function () {
      if (!document.querySelector("a[" + CURRENT + "]")) {
        var first = document.querySelector("body a[href]");
        if (first) first.setAttribute(CURRENT, "");
      }
    },
    undecorate: function () {
      Array.prototype.forEach.call(document.querySelectorAll("[" + CURRENT + "]"), function (el) {
        el.removeAttribute(CURRENT);
      });
    }
  };
})
