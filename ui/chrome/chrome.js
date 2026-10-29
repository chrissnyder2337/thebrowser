// Controller for the chrome pages (toolbar and status bar) of every UI mode.
// pages.rs puts it into each mode's ui/chrome/<mode>/{toolbar,status}.html, so a
// mode only supplies markup and CSS. It binds by convention; every element is
// optional:
//   [data-cmd]      sends its command on click, unless it has the "disabled"
//                   class. data-arg is the argument; data-anchor appends the
//                   element's bottom-left corner ("x,y") for a menu to open at.
//   #url, #go       the address field and its Go button.
//   #field          gets "secure" on https pages.
//   #title          shows the page title.
//   #throbber       activity indicator ("loading" class); opens About.
//   #back, #forward, #stop   disabled to match the navigation state.
//   #retro          latched while a retro look is on.
//   #links          quick links, drawn with <template id="page-icon">.
//   #msg, #meter, #lock-pane, #modem-pane, #modem-label   the status line.
(function () {
  "use strict";
  var seq = 0;
  // Title channel to Rust: "lantern|<seq>|<command>|<encoded arg>".
  function send(cmd, arg) {
    seq += 1;
    document.title = "lantern|" + seq + "|" + cmd + "|" + encodeURIComponent(arg || "");
  }
  function $(id) { return document.getElementById(id); }

  var url = $("url");
  var field = $("field");
  var title = $("title");
  var throbber = $("throbber");
  var links = $("links");
  var iconTemplate = $("page-icon");
  var pageIcon = iconTemplate ? iconTemplate.innerHTML : "";
  var msg = $("msg");
  var meter = $("meter");
  var lock = $("lock-pane");
  var modem = $("modem-pane");
  var modemLabel = $("modem-label");
  var buttons = { back: $("back"), forward: $("forward"), stop: $("stop"), retro: $("retro") };
  var state = { url: "" };
  var editing = false;

  Array.prototype.forEach.call(document.querySelectorAll("[data-cmd]"), function (b) {
    b.addEventListener("click", function () {
      if (b.classList.contains("disabled")) return;
      var arg = b.dataset.arg || "";
      if ("anchor" in b.dataset) {
        // Menus open under the element, so tell Rust where that is.
        var r = b.getBoundingClientRect();
        arg = (arg ? arg + "," : "") + r.left + "," + r.bottom;
      }
      send(b.dataset.cmd, arg);
    });
  });
  if (throbber) throbber.addEventListener("click", function () { send("about"); });

  function go() {
    var text = url.value.trim();
    editing = false;
    if (text) send("go", text);
  }
  if (url) {
    if ($("go")) $("go").addEventListener("click", go);
    url.addEventListener("keydown", function (e) {
      if (e.key === "Enter") { e.preventDefault(); go(); }
      else if (e.key === "Escape") { editing = false; url.value = state.url; url.select(); }
    });
    url.addEventListener("input", function () { editing = true; });
    url.addEventListener("focus", function () { setTimeout(function () { url.select(); }, 0); });
    url.addEventListener("blur", function () {
      if (editing && url.value.trim() === "") { editing = false; url.value = state.url; }
    });
  }

  // Chrome pages have no context menu (except the URL field) and nothing to
  // drag. A link dropped on a page with an address field opens it.
  document.addEventListener("contextmenu", function (e) { if (!url || e.target !== url) e.preventDefault(); });
  document.addEventListener("dragstart", function (e) { if (!url || e.target !== url) e.preventDefault(); });
  document.addEventListener("dragover", function (e) { e.preventDefault(); });
  document.addEventListener("drop", function (e) {
    e.preventDefault();
    if (!url) return;
    var text = e.dataTransfer && (e.dataTransfer.getData("text/uri-list") || e.dataTransfer.getData("text/plain"));
    if (text) send("go", text.split("\n")[0]);
  });

  function setDisabled(el, disabled) { if (el) el.classList.toggle("disabled", !!disabled); }

  function renderLinks(list) {
    links.textContent = "";
    if (!list.length) {
      var none = document.createElement("span");
      none.className = "none";
      none.textContent = "(no bookmarks)";
      links.appendChild(none);
      return;
    }
    list.forEach(function (b) {
      var btn = document.createElement("button");
      btn.className = "ql";
      btn.title = b.url;
      btn.innerHTML = pageIcon;
      var label = document.createElement("span");
      label.textContent = b.title;
      btn.appendChild(label);
      btn.addEventListener("click", function () { send("open", b.url); });
      links.appendChild(btn);
    });
  }

  // The meter is a row of blocks; data-blocks says how many fill it.
  var blocks = meter ? Number(meter.dataset.blocks || 11) : 0;
  var shown = -1;
  function setBlocks(n) {
    if (n === shown) return;
    shown = n;
    meter.textContent = "";
    for (var i = 0; i < n; i++) meter.appendChild(document.createElement("i"));
  }

  var lastLinks = null;
  window.lantern = {
    update: function (s) {
      if ("url" in s && url) {
        state.url = s.url;
        if (!editing || document.activeElement !== url) { editing = false; url.value = s.url; }
      }
      if ("title" in s && title) { title.textContent = s.title; title.title = s.title; }
      if ("canBack" in s) setDisabled(buttons.back, !s.canBack);
      if ("canFwd" in s) setDisabled(buttons.forward, !s.canFwd);
      if ("loading" in s) {
        setDisabled(buttons.stop, !s.loading);
        if (throbber) throbber.classList.toggle("loading", !!s.loading);
        if (meter) meter.classList.toggle("on", !!s.loading);
      }
      if ("secure" in s) {
        if (field) field.classList.toggle("secure", !!s.secure);
        if (lock) {
          lock.classList.toggle("secure", !!s.secure);
          lock.title = s.secure ? "This page was delivered over a secure connection." : "";
        }
      }
      if ("retro" in s && buttons.retro) {
        var on = s.retro !== "off";
        buttons.retro.classList.toggle("latched", on);
        buttons.retro.setAttribute("aria-pressed", on ? "true" : "false");
        buttons.retro.title = "Retro looks: " + s.retroName;
      }
      if (s.bookmarks && links) {
        var key = JSON.stringify(s.bookmarks);
        if (key !== lastLinks) { lastLinks = key; renderLinks(s.bookmarks); }
      }
      if ("text" in s && msg) { msg.textContent = s.text; msg.parentNode.title = s.text; }
      if ("progress" in s && meter) setBlocks(Math.round(Math.max(0, Math.min(1, s.progress)) * blocks));
      if ("modem" in s && modem) {
        if (modemLabel) modemLabel.textContent = s.modem;
        modem.classList.toggle("dialup", !!s.modem);
        modem.title = s.modemTip || "Dial-up simulator";
      }
    },
    focusUrl: function () {
      if (!url) return;
      url.focus();
      url.select();
    }
  };

  send("ready");
})();
