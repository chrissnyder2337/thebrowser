// A text-mode browser in an 80-column terminal, 1999: the page's own
// stylesheets are switched off and everything is drawn as light grey
// monospace text on black. Images are dropped (an image-only link keeps an
// "[IMG]" label so it stays clickable), links are bright with the selected
// one in reverse video. A look for runtime.js.
(function (R) {
  "use strict";

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
      // A terminal has one size and one weight; headings alone are bold.
      p + ":is(h1, h2, h3, h4, h5, h6) { font-weight: bold !important; color: #ffffff !important;" +
        " margin: 20px 0 !important; }",
      p + "h1 { text-align: center !important; text-transform: uppercase !important; }",
      p + ":is(b, strong, th) { font-weight: bold !important; }",
      p + ":is(em, i, cite, var, dfn) { font-style: normal !important; text-decoration: underline !important; }",
      p + "p, " + p + "ul, " + p + "ol, " + p + "dl, " + p + "pre, " + p + "blockquote { margin: 20px 0 !important; }",
      p + "pre { white-space: pre !important; overflow-x: auto; }",
      p + "a:link { color: #55ffff !important; text-decoration: none !important; }",
      p + "a:visited { color: #00aaaa !important; text-decoration: none !important; }",
      p + "a:is(:hover, :focus, :active) { background: #c0c0c0 !important; color: #000000 !important; outline: 0 !important; }",
      // Text wrapped in spans inside a link takes the link's color, not the
      // catch-all grey above.
      p + "a * { color: inherit !important; }",
      // No images: an image that was the only content of a link leaves a label.
      p + ":is(img, video, input[type=image], picture, canvas, iframe, svg, object, embed, audio):not([" + R.DECOR + "])" + plain +
        " { display: none !important; }",
      p + "a:has(> img:only-child)::before { content: \"[IMG]\"; }",
      // Tables keep their layout, without borders or backgrounds.
      p + ":is(table, thead, tbody, tfoot, tr, td, th, caption) { border: 0 !important; background: transparent !important;" +
        " border-radius: 0 !important; }",
      p + "hr { border: 0 !important; border-top: 1px dashed #c0c0c0 !important; height: 0 !important; margin: 10px 0 !important; }",
      p + "ul { list-style: none !important; padding-left: 40px !important; }",
      p + "ul > li { list-style: \"* \" outside !important; display: list-item !important; }",
      p + "ol { list-style: decimal !important; padding-left: 40px !important; }",
      p + "ol > li { list-style: decimal !important; }",
      // Form controls: a grey box on black, buttons in reverse video.
      p + ":is(input, textarea, select, button)" + plain + " { font-family: " + MONO + " !important; font-size: 15px !important;" +
        " background: #000000 !important; color: #c0c0c0 !important; border: 1px solid #c0c0c0 !important;" +
        " border-radius: 0 !important; padding: 0 4px !important; appearance: none !important; outline: 0 !important; }",
      p + ":is(input, textarea, select, button):focus" + plain + " { border-color: #ffffff !important; color: #ffffff !important; }",
      p + ":is(input:is([type=submit], [type=button], [type=reset]), button)" + plain + " { background: #c0c0c0 !important;" +
        " color: #000000 !important; cursor: pointer; }",
      p + "button *" + plain + " { color: inherit !important; }",
      p + "button::before { content: \"[ \"; }",
      p + "button::after { content: \" ]\"; }",
      p + ":is(input[type=checkbox], input[type=radio]) { appearance: auto !important; width: 15px; height: 15px; }",
      p + "::placeholder { color: #808080 !important; }"
    ].join("\n");
  }

  return {
    noPageStyles: true,
    solid: null,
    pageCss: [
      "html, html body { background: #000000 !important; color: #c0c0c0 !important; }",
      "html body { margin: 8px !important; max-width: none !important; }"
    ].join("\n"),
    css: css,
    undecorate: function () {}
  };
})
