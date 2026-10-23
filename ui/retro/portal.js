// A 1997 web portal: the page squeezed into a 760px column (it has to fit an
// 800x600 screen), small Arial on white, headings on colored bars, blue
// underlined links, and portal trimmings: a channel bar, a search box, a
// blinking 468x60 banner ad and a copyright footer. A look for runtime.js.
(function (R) {
  "use strict";
  var FONT = "Arial, Helvetica, sans-serif";
  var BAR = "#336699";

  function css(p, all) {
    return [
      all + " { color: #000 !important; background-color: transparent !important; background-image: none !important;",
      "  font-family: " + FONT + " !important; font-size: 13px !important; line-height: 1.3 !important;",
      "  letter-spacing: normal !important; text-transform: none !important; text-shadow: none !important;",
      "  -webkit-text-fill-color: currentColor !important; -webkit-text-stroke: 0 !important;",
      "  box-shadow: none !important; border-radius: 0 !important; }",
      p + "a, " + p + "a * { color: #0000cc !important; text-decoration: underline !important; }",
      p + "a:visited, " + p + "a:visited * { color: #551a8b !important; }",
      p + ":is(h1, h2, h3) { background-color: " + BAR + " !important; padding: 2px 6px !important;",
      "  margin: 10px 0 4px !important; font-weight: bold !important; text-align: left !important; }",
      p + ":is(h1, h2, h3), " + p + ":is(h1, h2, h3) * { color: #fff !important; -webkit-text-fill-color: #fff !important; }",
      p + "h1, " + p + "h1 * { font-size: 18px !important; }",
      p + "h2, " + p + "h2 * { font-size: 15px !important; }",
      p + ":is(h4, h5, h6, b, strong) { font-weight: bold !important; }",
      p + "th { background-color: #dddddd !important; font-weight: bold !important; }",
      p + "hr { border: 0 !important; border-top: 1px solid #999 !important; height: 0 !important; }",
      p + ":is(img, video) { max-width: 100% !important; height: auto; }",
      p + ":is(button, select) { color: #000 !important; background-color: #d4d0c8 !important;",
      "  border: 2px outset #fff !important; font-size: 12px !important; padding: 1px 6px !important; }",
      p + ":is(input:not([type]), input[type=text], input[type=search], input[type=email], input[type=url], textarea) {",
      "  background-color: #fff !important; border: 2px inset #d4d0c8 !important; }",
      p + "[" + R.SOLID + "] { border: 1px solid #999 !important; }"
    ].join("\n");
  }

  var decorCss = [
    ":host > * { font: 13px " + FONT + "; color: #000; }",
    "a { color: #0000cc; }",
    ".top { display: flex; align-items: center; gap: 12px; padding: 6px 0; border-bottom: 2px solid " + BAR + "; }",
    ".logo { font: bold italic 26px Georgia, \"Times New Roman\", serif; color: #cc0000; white-space: nowrap; }",
    ".logo span { color: " + BAR + "; }",
    "form { display: flex; gap: 4px; align-items: center; flex: 1; min-width: 0; }",
    "input { flex: 1; min-width: 0; font: 13px " + FONT + "; border: 2px inset #d4d0c8; padding: 2px; }",
    "button { font: 12px " + FONT + "; background: #d4d0c8; border: 2px outset #fff; padding: 1px 8px; }",
    ".chan { padding: 4px 0; font-size: 12px; text-align: center; background: #ffffcc; border-bottom: 1px solid #cccc99; }",
    ".chan a { margin: 0 5px; }",
    ".ad { width: 468px; max-width: 100%; height: 60px; margin: 8px auto; box-sizing: border-box; cursor: pointer;",
    "  display: flex; flex-direction: column; align-items: center; justify-content: center;",
    "  border: 2px solid #000; background: linear-gradient(90deg, #ff0000, #ffcc00); text-align: center; }",
    ".ad b { font: bold 17px " + FONT + "; color: #fff; text-shadow: 1px 1px #000; animation: blink 0.8s steps(1) infinite; }",
    ".ad small { font-size: 11px; color: #000; }",
    ".adlabel { text-align: center; font-size: 10px; color: #666; }",
    ".foot { margin-top: 20px; padding: 8px 0; border-top: 2px solid " + BAR + "; text-align: center; font-size: 11px; color: #666; }",
    "@keyframes blink { 50% { visibility: hidden; } }"
  ].join("\n");

  var CHANNELS = ["Mail", "News", "Weather", "Stock Quotes", "Sports", "Horoscopes", "Chat", "Free Home Pages"];
  var head = null;
  var foot = null;
  var headSheet = null;

  function search(q) {
    return "https://duckduckgo.com/?q=" + encodeURIComponent(q);
  }

  function header() {
    var d = R.overlay("lantern-portal-header", headSheet);
    var top = R.el("div", "top");
    var logo = R.el("div", "logo", "Lantern");
    logo.appendChild(R.el("span", "", "Net"));
    var form = R.el("form");
    form.action = "https://duckduckgo.com/";
    form.method = "get";
    var input = R.el("input");
    input.name = "q";
    input.placeholder = "Search the Web";
    form.appendChild(input);
    form.appendChild(R.el("button", "", "Search"));
    top.appendChild(logo);
    top.appendChild(form);
    var chan = R.el("div", "chan");
    CHANNELS.forEach(function (name) {
      var a = R.el("a", "", name);
      a.href = search(name);
      chan.appendChild(a);
    });
    var ad = R.el("div", "ad");
    ad.appendChild(R.el("b", "", "CONGRATULATIONS! You are our 1,000,000th visitor!"));
    ad.appendChild(R.el("small", "", "Click here to claim your FREE prize!"));
    ad.addEventListener("click", function () {
      ad.firstChild.textContent = "Just kidding. It's 1997.";
    });
    d.root.appendChild(top);
    d.root.appendChild(chan);
    d.root.appendChild(ad);
    d.root.appendChild(R.el("div", "adlabel", "Advertisement"));
    return d;
  }

  function footer() {
    var d = R.overlay("lantern-portal-footer", headSheet);
    var f = R.el("div", "foot");
    ["Add URL", "Advertise with Us", "Help", "Privacy Policy"].forEach(function (name, i) {
      if (i) f.appendChild(document.createTextNode(" - "));
      var a = R.el("a", "", name);
      a.href = search(name);
      f.appendChild(a);
    });
    f.appendChild(R.el("br"));
    f.appendChild(document.createTextNode("Copyright © 1997 LanternNet, Inc. All rights reserved."));
    d.root.appendChild(f);
    return d;
  }

  return {
    solid: "#ffffff",
    unpinHeaders: true,
    pageCss: [
      "html { background: #fff !important; }",
      "html body { max-width: 760px !important; margin: 0 auto !important; padding: 0 8px !important;",
      "  background: #fff !important; }"
    ].join("\n"),
    css: css,
    decorate: function () {
      if (!headSheet) headSheet = R.sheet(decorCss);
      if (!head || !head.host.isConnected) {
        head = header();
        // Scroll anchoring would scroll the page down past the new header.
        var atTop = window.scrollY === 0;
        document.body.insertBefore(head.host, document.body.firstChild);
        if (atTop) window.scrollTo(0, 0);
      }
      if (!foot || !foot.host.isConnected) foot = footer();
      R.keepLast(foot.host);
    },
    undecorate: function () {
      [head, foot].forEach(function (d) { if (d) d.host.remove(); });
      head = foot = null;
    }
  };
})
