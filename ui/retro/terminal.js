// A text-mode browser in an 80-column terminal, 1999: the page's own
// stylesheets are switched off and everything is drawn as light grey
// monospace text on black. Images give way to their alt text (or [INLINE],
// [LINK]), links
// are green with the selected one in reverse video. A look for runtime.js.
(function (R) {
  "use strict";

  // Marks images that have their alt text beside them, and those labels.
  var ALT = "data-lantern-retro-alt";
  // Marks the current link.
  var CURRENT = "data-lantern-retro-current";
  var MONO = "\"DejaVu Sans Mono\", \"Liberation Mono\", Menlo, Consolas, \"Courier New\", monospace";

  function css(p, all) {
    // Doubling the :not() lifts a rule above the [style] revert below, which
    // would otherwise bring back an inline display:none.
    var plain = ":not([" + R.DECOR + "]):not([" + R.DECOR + "])";
    return [
      // Inline style attributes survive switching the stylesheets off, so
      // send those elements back to the browser's own defaults too.
      p + "[style]:not([" + R.DECOR + "]) { all: revert !important; }",
      all + " { font-family: " + MONO + " !important; font-size: 15px !important; line-height: 20px !important;" +
        " font-weight: normal !important; font-style: normal !important; color: #c0c0c0 !important;" +
        " background: transparent !important; text-shadow: none !important; box-shadow: none !important; }",
      // A terminal has one size and one weight; only h1, bold and th stand out.
      p + ":is(h1, h2, h3, h4, h5, h6) { margin: 20px 0 !important; }",
      p + "h1 { font-weight: bold !important; color: #ffffff !important; text-align: center !important;" +
        " text-transform: none !important; }",
      p + "h1 *" + plain + " { font-weight: inherit !important; color: inherit !important; text-transform: none !important; }",
      p + ":is(b, strong, th) { font-weight: bold !important; color: #ffffff !important; }",
      p + ":is(em, i, cite, var, dfn) { font-style: normal !important; text-decoration: underline !important; }",
      p + "p, " + p + "ul, " + p + "ol, " + p + "dl, " + p + "pre, " + p + "blockquote { margin: 20px 0 !important; }",
      // Text starts at column 4; the text-mode browser indented these by 3.
      p + ":is(p, table, dl, blockquote) { margin-left: 3ch !important; }",
      p + "pre { white-space: pre !important; overflow-x: auto; }",
      p + ":is(a:link, a:visited) { color: #00aa00 !important; text-decoration: none !important; }",
      // The current link: reverse video, bold.
      p + "a:is(:hover, :focus, :active) { background: #d9d9d9 !important; color: #000000 !important;" +
        " font-weight: bold !important; outline: 0 !important; }",
      // The browser always had a current link: the first on the page, until
      // the reader moved.
      p + "a[" + CURRENT + "]:not(:hover) { background: #d9d9d9 !important; color: #000000 !important; font-weight: bold !important; }",
      // Text wrapped in spans inside a link takes the link's color, not the
      // catch-all grey above.
      p + "a * { color: inherit !important; font-weight: inherit !important; }",
      // No pictures: decorate() puts each image's alt text (or a label)
      // beside it, since content: attr(alt) on <img> draws nothing in Chromium.
      p + ":is(img, video, input[type=image], picture, canvas, iframe, svg, object, embed, audio):not([" + R.DECOR + "])" + plain +
        " { display: none !important; }",
      // Tables keep their layout, without borders or backgrounds.
      p + ":is(table, thead, tbody, tfoot, tr, td, th, caption) { border: 0 !important; background: transparent !important;" +
        " border-radius: 0 !important; }",
      p + "hr { border: 0 !important; border-top: 1px solid #a2a2a2 !important; height: 0 !important; margin: 10px 3ch !important; }",
      // Bullets at column 5, text at column 7.
      p + "ul { list-style: none !important; padding-left: 7ch !important; }",
      p + "ul > li { list-style: \"* \" outside !important; display: list-item !important; }",
      p + "ol { list-style: decimal !important; padding-left: 40px !important; }",
      p + "ol > li { list-style: decimal !important; }",
      // Form controls: text fields get an underline, buttons are [label].
      p + ":is(input, textarea, select, button)" + plain + " { font-family: " + MONO + " !important; font-size: 15px !important;" +
        " background: transparent !important; color: #c0c0c0 !important; border: 0 !important;" +
        " border-bottom: 1px solid #c0c0c0 !important; border-radius: 0 !important; padding: 0 !important;" +
        " appearance: none !important; outline: 0 !important; }",
      p + ":is(input, textarea, select, button):focus" + plain + " { color: #ffffff !important; border-bottom-color: #ffffff !important; }",
      p + ":is(input:is([type=submit], [type=button], [type=reset]), button)" + plain + " { border: 0 !important;" +
        " cursor: pointer; }",
      p + "button *" + plain + " { color: inherit !important; }",
      p + "button::before { content: \"[\"; }",
      p + "button::after { content: \"]\"; }",
      p + "input[type=checkbox]" + plain + " { width: 3ch !important; height: 20px !important; border: 0 !important;" +
        " display: inline-block; vertical-align: top; margin: 0 !important; }",
      p + "input[type=checkbox]::before { content: \"[ ]\"; }",
      p + "input[type=checkbox]:checked::before { content: \"[X]\"; }",
      p + "input[type=radio]" + plain + " { width: 3ch !important; height: 20px !important; border: 0 !important;" +
        " display: inline-block; vertical-align: top; margin: 0 !important; }",
      p + "input[type=radio]::before { content: \"( )\"; }",
      p + "input[type=radio]:checked::before { content: \"(*)\"; }",
      p + "::placeholder { color: #808080 !important; }"
    ].join("\n");
  }

  return {
    noPageStyles: true,
    solid: null,
    pageCss: [
      "html, html body { background: #000000 !important; color: #c0c0c0 !important; }",
      "html body { margin: 0 !important; max-width: none !important; }"
    ].join("\n"),
    css: css,
    decorate: function () {
      Array.prototype.forEach.call(document.querySelectorAll("img:not([" + ALT + "])"), function (img) {
        img.setAttribute(ALT, "");
        var label = document.createElement("span");
        label.setAttribute(R.DECOR, "");
        label.setAttribute(ALT, "");
        var alt = (img.getAttribute("alt") || "").trim();
        // Text only: alt text is the page's, never markup.
        var link = img.closest("a");
        // A link with text of its own needs no label for its picture.
        if (!alt && link && link.textContent.trim()) return;
        label.textContent = alt ? alt.slice(0, 80) : (link ? "[LINK]" : "[INLINE]");
        img.after(label);
      });
      if (!document.querySelector("a[" + CURRENT + "]")) {
        var first = document.querySelector("body a[href]");
        if (first) first.setAttribute(CURRENT, "");
      }
    },
    undecorate: function () {
      Array.prototype.forEach.call(document.querySelectorAll("span[" + ALT + "]"), function (el) { el.remove(); });
      Array.prototype.forEach.call(document.querySelectorAll("[" + ALT + "], [" + CURRENT + "]"), function (el) {
        el.removeAttribute(ALT);
        el.removeAttribute(CURRENT);
      });
    }
  };
})
