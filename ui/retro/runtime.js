// Shared engine for Lantern's retro looks (see src/shell/retro.rs). Rust runs
//
//   (function (style) {
//     var R = window.__lanternRetro;
//     if (!R) { R = window.__lanternRetro = (<this file>)(); R.define("geocities", <geocities.js>); ... }
//     R.apply(style);
//   })("geocities")
//
// in every frame of the content view when a load starts and when it ends, and
// whenever the look changes, so applying the active look again only rescans.
// R.apply("off") takes everything back out.
//
// A look is a factory taking the helpers below and returning
//   css(p, all)    rules for the document (p = "html body ", all = `body *`)
//                  and for open shadow trees (p = "", all = `*`)
//   pageCss        extra rules for the document only (html, body)
//   solid          background for floating boxes (menus, cookie banners) that
//                  had a solid one, or null to leave them alone
//   unpinHeaders   put bars pinned across the top of the window (sticky
//                  headers) back into the page flow
//   noPageStyles   switch the page's own stylesheets off ("no styles" looks)
//   decorate()     add banners and the like to the top-level page; called on
//                  every scan, so it must check whether it is already there
//   undecorate()
//
// Styles go in constructed stylesheets (adoptedStyleSheets), which a page's
// Content-Security-Policy doesn't block the way it blocks <style> elements.
(function () {
  "use strict";
  // Marks floating elements that had a solid background, so a look can keep
  // them solid instead of letting them turn see-through.
  var SOLID = "data-lantern-retro-solid";
  var UNPINNED = "data-lantern-retro-unpinned";
  // Marks decoration hosts, which page styles must leave alone.
  var DECOR = "data-lantern-retro-decor";
  // Everything except SVG internals and icon-font glyphs, which would turn
  // into their ligature names if their font were swapped. :where() keeps the
  // catch-all at the lowest specificity so a look's other rules beat it.
  var ANY = ':where(:not(svg *):not(i):not([class*="icon" i]):not([class*="fa-"]))';
  var OBSERVE = { childList: true, subtree: true };

  var styles = {};
  var active = null;
  var roots = [];
  var hidden = [];
  var observer = new MutationObserver(schedule);
  var scanTimer = 0;
  var lastScan = 0;
  var startedAt = 0;

  function sheet(text) {
    var s = new CSSStyleSheet();
    s.replaceSync(text);
    return s;
  }

  function adopt(target, s, enable) {
    var sheets = target.adoptedStyleSheets.filter(function (x) { return x !== s; });
    target.adoptedStyleSheets = enable ? sheets.concat([s]) : sheets;
  }

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

  // A custom element with a closed shadow root, for decorations the page's
  // scripts and styles can't reach. The host is reset to `all: initial`, so
  // nothing is inherited from the page: set fonts and colors inside.
  function overlay(tag, css) {
    var host = document.createElement(tag);
    host.setAttribute(DECOR, "");
    // Page rules can still match the host element itself (`body > * { display:
    // none }`); inline !important declarations beat them.
    host.style.setProperty("all", "initial", "important");
    host.style.setProperty("display", "block", "important");
    var root = host.attachShadow({ mode: "closed" });
    root.adoptedStyleSheets = [css];
    return { host: host, root: root };
  }

  var helpers = {
    SOLID: SOLID,
    DECOR: DECOR,
    sheet: sheet,
    el: el,
    visitors: visitors,
    overlay: overlay,
    top: window === window.top
  };

  function schedule() {
    if (scanTimer) return;
    // Keep up closely while the page is still loading, then rescan at most
    // about once a second.
    var now = Date.now();
    var wait = now - startedAt < 5000 ? 100 : Math.max(250, 1000 - (now - lastScan));
    scanTimer = setTimeout(scan, wait);
  }

  // Every element of `scope` and of the open shadow trees inside it,
  // styling and watching shadow trees not seen before.
  function walk(scope, out) {
    var all = scope.querySelectorAll("*");
    for (var i = 0; i < all.length; i++) {
      out.push(all[i]);
      var shadow = all[i].shadowRoot;
      if (shadow) {
        if (roots.indexOf(shadow) < 0) {
          roots.push(shadow);
          adopt(shadow, active.shadowSheet, true);
          observer.observe(shadow, OBSERVE);
        }
        walk(shadow, out);
      }
    }
    return out;
  }

  // Switch off the page's own stylesheets, like View > Page Style > No Style.
  function hidePageStyles() {
    [document].concat(roots).forEach(function (scope) {
      var own = Array.prototype.slice.call(scope.styleSheets).concat(scope.adoptedStyleSheets);
      own.forEach(function (s) {
        if (!s.disabled && s !== active.pageSheet && s !== active.shadowSheet) {
          s.disabled = true;
          hidden.push(s);
        }
      });
    });
  }

  function floatsWithSolidBackground(style) {
    var floating = style.position === "fixed" || style.position === "sticky" ||
      (style.position === "absolute" && parseInt(style.zIndex, 10) > 0);
    if (!floating) return false;
    var rgba = style.backgroundColor.match(/[\d.]+/g);
    return !!rgba && (rgba.length < 4 || Number(rgba[3]) > 0.5);
  }

  function pinnedAcrossTop(e, style) {
    if (style.position !== "fixed" && style.position !== "sticky") return false;
    var rect = e.getBoundingClientRect();
    return rect.top < 150 && rect.width > window.innerWidth / 2;
  }

  function mark(elements, attr, flags) {
    elements.forEach(function (e, i) {
      if (flags[i] !== e.hasAttribute(attr)) e.toggleAttribute(attr, flags[i]);
    });
  }

  function scan() {
    scanTimer = 0;
    lastScan = Date.now();
    if (!active) return;
    var elements = walk(document, []);
    var def = active.def;
    if (def.noPageStyles) hidePageStyles();
    if (def.solid || def.unpinHeaders) {
      // Read the page's own layout with our sheets switched off. Nothing
      // paints in between, so this doesn't flicker.
      active.pageSheet.disabled = active.shadowSheet.disabled = true;
      var solid = [];
      var pinned = [];
      elements.forEach(function (e) {
        var style = getComputedStyle(e);
        solid.push(!!def.solid && floatsWithSolidBackground(style));
        pinned.push(!!def.unpinHeaders && pinnedAcrossTop(e, style));
      });
      active.pageSheet.disabled = active.shadowSheet.disabled = false;
      if (def.solid) mark(elements, SOLID, solid);
      if (def.unpinHeaders) mark(elements, UNPINNED, pinned);
    }
    // Banners only go on the top-level page, not in iframes.
    if (helpers.top && document.body && active.def.decorate) active.def.decorate();
  }

  function sheetsFor(def) {
    if (!def.pageSheet) {
      // Attribute rules: the document's need the "html body " prefix to beat
      // the look's own rules there.
      var marks = function (p) {
        return (def.solid ? p + "[" + SOLID + "] { background-color: " + def.solid + " !important; }\n" : "") +
          (def.unpinHeaders ? p + "[" + UNPINNED + "] { position: relative !important; top: auto !important; " +
            "bottom: auto !important; }\n" : "");
      };
      def.pageSheet = sheet((def.pageCss || "") + "\n" + def.css("html body ", "body, body " + ANY) +
        "\n" + marks("html body "));
      def.shadowSheet = sheet(def.css("", ANY) + "\n" + marks(""));
    }
    return def;
  }

  function start(name) {
    var def = sheetsFor(styles[name]);
    active = { name: name, def: def, pageSheet: def.pageSheet, shadowSheet: def.shadowSheet };
    startedAt = Date.now();
    adopt(document, active.pageSheet, true);
    observer.observe(document, OBSERVE);
    scan();
  }

  function stop() {
    if (!active) return;
    observer.disconnect();
    clearTimeout(scanTimer);
    scanTimer = 0;
    adopt(document, active.pageSheet, false);
    roots.forEach(function (shadow) { adopt(shadow, active.shadowSheet, false); });
    [document].concat(roots).forEach(function (scope) {
      scope.querySelectorAll("[" + SOLID + "], [" + UNPINNED + "]").forEach(function (e) {
        e.removeAttribute(SOLID);
        e.removeAttribute(UNPINNED);
      });
    });
    hidden.forEach(function (s) { s.disabled = false; });
    hidden = [];
    roots = [];
    if (active.def.undecorate) active.def.undecorate();
    active = null;
  }

  return {
    define: function (name, factory) {
      if (!styles[name]) styles[name] = factory(helpers);
    },
    apply: function (name) {
      if (active && active.name === name) {
        scan();
        return;
      }
      stop();
      if (styles[name]) start(name);
    }
  };
})
