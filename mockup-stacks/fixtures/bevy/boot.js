// Boot script for `pack-wasm.py generic`: builds the page chrome, starts the wasm-bindgen module from the embedded
// bytes, and flips window.__mockupReady once Bevy has drawn a few frames.
(function () {
  window.__mockupReady = false;
  var css = document.createElement('style');
  css.textContent =
    'body{background:#14161c;font:13px system-ui,sans-serif;color:#aab}' +
    '#mockup-control{height:40px;display:flex;align-items:center;justify-content:center;gap:10px}' +
    '#mockup-control a{padding:4px 12px;border-radius:12px;background:#262a36;color:#dde;text-decoration:none}' +
    '#mockup-control a.on{background:#0b6cf0;color:#fff}' +
    '#frame{width:1024px;height:576px;margin:0 auto;position:relative;box-shadow:0 0 0 1px #333}' +
    '#bevy-canvas{width:100%;height:100%;display:block;outline:none}';
  document.head.appendChild(css);
  document.body.innerHTML =
    '<div id="mockup-control"><span>Mockup control, not part of the game:</span>' +
    '<a href="#playing" data-s="playing">Playing</a><a href="#gameover" data-s="gameover">Game over</a></div>' +
    '<div id="frame"><canvas id="bevy-canvas"></canvas></div>';
  function mark() {
    var s = location.hash === '#gameover' ? 'gameover' : 'playing';
    document.querySelectorAll('#mockup-control a').forEach(function (a) { a.className = a.dataset.s === s ? 'on' : ''; });
  }
  mark(); window.addEventListener('hashchange', mark);
  var poll = setInterval(function () {
    if ((window.__bevyFrames || 0) >= 8) { clearInterval(poll); window.__mockupReady = true; }
  }, 50);
  window.__mockupBytes('star-catcher_bg.wasm').then(function (bytes) {
    return wasm_bindgen({ module_or_path: bytes });
  }).catch(function (e) {
    // winit used to end its start-up by throwing this on purpose.
    if (!/control flow/.test(String(e))) { document.getElementById('frame').textContent = 'Bevy failed: ' + e; throw e; }
  });
})();
