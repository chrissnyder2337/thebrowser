// The text-only web of 1995: the page's own stylesheets are switched off, so
// it renders the way an early browser showed plain HTML (gray page, Times,
// blue and purple underlined links), with the homepage furniture of the day
// at the bottom: a "best viewed with" line, a hit counter and a last-updated
// date. A look for runtime.js.
(function (R) {
  "use strict";

  function css(p, all) {
    return [
      // Inline style attributes survive switching the stylesheets off, so
      // send those elements back to the browser's own defaults too.
      p + "[style]:not([" + R.DECOR + "]) { all: revert !important; }",
      all + " { font-family: \"Times New Roman\", Times, serif !important; }",
      p + ":is(code, kbd, pre, samp, tt) { font-family: \"Courier New\", Courier, monospace !important; }",
      p + "a:link { color: #0000ee !important; }",
      p + "a:visited { color: #551a8b !important; }",
      p + "a:active { color: #ff0000 !important; }",
      p + "img, " + p + "video { max-width: 100%; height: auto; }",
      // Unstyled icon SVGs would otherwise fill the whole width.
      p + "svg:not([width]) { width: 1.25em; height: 1.25em; }"
    ].join("\n");
  }

  var footSheet = R.sheet([
    "hr { border: 0; border-top: 1px solid #808080; border-bottom: 1px solid #fff; margin: 24px 0 12px; }",
    "p { text-align: center; margin: 8px 0; font: 16px \"Times New Roman\", Times, serif; color: #000; }",
    ".badge { display: inline-block; margin: 4px 0 8px; padding: 6px 10px; border: 2px outset #fff;",
    "  background: #000080; color: #fff; font: bold 12px Arial, Helvetica, sans-serif; letter-spacing: 1px; }",
    ".badge b { color: #ffff00; }",
    ".count { font: bold 15px \"Courier New\", monospace; background: #000; color: #00ff00; padding: 0 4px;",
    "  border: 2px inset #808080; letter-spacing: 2px; }",
    "a { color: #0000ee; }",
    "small { font-size: 13px; }"
  ].join("\n"));

  var foot = null;

  function lastUpdated() {
    var d = new Date(document.lastModified);
    if (isNaN(d)) d = new Date();
    var months = ["January", "February", "March", "April", "May", "June", "July",
      "August", "September", "October", "November", "December"];
    return months[d.getMonth()] + " " + d.getDate() + ", " + d.getFullYear();
  }

  function para(parts) {
    var p = R.el("p");
    parts.forEach(function (part) { p.appendChild(typeof part === "string" ? document.createTextNode(part) : part); });
    return p;
  }

  return {
    noPageStyles: true,
    solid: null,
    pageCss: [
      "html, html body { background: #c0c0c0 !important; color: #000 !important; }",
      "html body { margin: 8px !important; }"
    ].join("\n"),
    css: css,
    decorate: function () {
      if (foot && foot.host.isConnected) return;
      foot = R.overlay("lantern-netscape-footer", footSheet);
      var badge = R.el("div", "badge");
      badge.appendChild(document.createTextNode("Lantern "));
      badge.appendChild(R.el("b", "", "NOW!"));
      var top = R.el("a", "", "Back to top");
      top.href = "#";
      top.addEventListener("click", function (e) { e.preventDefault(); window.scrollTo(0, 0); });
      var mail = R.el("a", "", "Email the webmaster");
      mail.href = "mailto:webmaster@" + (location.hostname || "localhost");
      foot.root.appendChild(R.el("hr"));
      foot.root.appendChild(para([badge]));
      foot.root.appendChild(para(["This page is best viewed with Lantern 3.0 at 800x600 in 256 colors."]));
      foot.root.appendChild(para(["You are visitor number ", R.el("span", "count", R.visitors()), " since 1996."]));
      foot.root.appendChild(para(["[ ", top, " | ", mail, " ]"]));
      foot.root.appendChild(para([R.el("small", "", "Last updated: " + lastUpdated())]));
      document.body.appendChild(foot.host);
    },
    undecorate: function () {
      if (foot) foot.host.remove();
      foot = null;
    }
  };
})
