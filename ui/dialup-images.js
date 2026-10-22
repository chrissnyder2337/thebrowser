// Lantern dial-up simulator, "slow images" mode. Injected into content pages.
// Text arrives at full speed; each image stays hidden until it "arrives",
// then paints in from the top in bands, at the chosen modem speed.
// {{RATE}} is bytes per second, {{LATENCY}} milliseconds per request.
(function (rate, latency) {
  "use strict";
  if (window.__lanternDialup || !/^(https?|file):$/.test(location.protocol)) return;
  window.__lanternDialup = true;

  var LANES = 2;      // browsers of the era fetched about two images at a time
  var BAND = 4;       // reveal height step, in CSS pixels
  var MIN_AREA = 400; // spacers and tracking pixels just appear

  var queue = [];
  var active = [];
  var seen = new WeakSet();
  var ticking = false;
  var lastTick = 0;

  function setHidden(img, percent) {
    if (percent <= 0) img.style.removeProperty("clip-path");
    else img.style.setProperty("clip-path", "inset(0 0 " + percent + "% 0)", "important");
  }

  // Bytes for this image: the real transfer size when the browser exposes it
  // (same-origin or Timing-Allow-Origin), else a typical JPEG/GIF density.
  function imageBytes(img) {
    var src = img.currentSrc || img.src;
    var entries = src ? performance.getEntriesByName(src) : [];
    for (var i = entries.length - 1; i >= 0; i--) {
      var size = entries[i].encodedBodySize || entries[i].transferSize;
      if (size) return size;
    }
    return Math.max(800, img.naturalWidth * img.naturalHeight * 0.2);
  }

  function refill() {
    while (active.length < LANES && queue.length) {
      var img = queue.shift();
      active.push({ img: img, bytes: imageBytes(img), got: 0, wait: latency });
    }
  }

  function pump() {
    refill();
    if (active.length && !ticking) {
      ticking = true;
      lastTick = performance.now();
      requestAnimationFrame(tick);
    }
  }

  // The modem's bandwidth is shared by the images currently downloading;
  // each first waits out the request latency.
  function tick(now) {
    var dt = Math.max(0, now - lastTick);
    lastTick = now;
    var downloading = active.filter(function (job) { return job.wait <= 0; }).length;
    var share = downloading ? (rate * dt) / 1000 / downloading : 0;
    active = active.filter(function (job) {
      if (job.wait > 0) {
        job.wait -= dt;
        return true;
      }
      job.got += share;
      var progress = Math.min(1, job.got / job.bytes);
      var height = job.img.getBoundingClientRect().height || job.img.height || 1;
      var shown = Math.floor((progress * height) / BAND) * BAND;
      setHidden(job.img, progress >= 1 ? 0 : 100 - (shown / height) * 100);
      return progress < 1;
    });
    refill();
    if (active.length) requestAnimationFrame(tick);
    else ticking = false;
  }

  function arrived(img) {
    if (img.naturalWidth * img.naturalHeight < MIN_AREA) {
      setHidden(img, 0);
      return;
    }
    queue.push(img);
    pump();
  }

  function track(img) {
    if (seen.has(img)) return;
    seen.add(img);
    setHidden(img, 100);
    if (img.complete) {
      if (img.naturalWidth) arrived(img);
      else setHidden(img, 0); // broken image: show its placeholder
    } else {
      img.addEventListener("load", function () { arrived(img); }, { once: true });
      img.addEventListener("error", function () { setHidden(img, 0); }, { once: true });
    }
  }

  function scan(node) {
    if (node.nodeType !== 1) return;
    if (node.tagName === "IMG") track(node);
    else node.querySelectorAll("img").forEach(track);
  }

  new MutationObserver(function (records) {
    records.forEach(function (record) {
      if (record.type === "attributes") {
        if (record.target.tagName !== "IMG") return;
        // A lazy loader swapped in the real source: download it again.
        seen.delete(record.target);
        track(record.target);
      } else {
        record.addedNodes.forEach(scan);
      }
    });
  }).observe(document, {
    childList: true,
    subtree: true,
    attributes: true,
    attributeFilter: ["src", "srcset"]
  });
  if (document.documentElement) scan(document.documentElement);
})({{RATE}}, {{LATENCY}});
