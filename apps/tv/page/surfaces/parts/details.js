// A title's details for whatever has focus. It waits for the viewer to settle, keeps what it read,
// and drops an answer that arrives after the focus moved on.
(function () {
  "use strict";
  var tv = window.tv;
  var kit = tv.kit;
  var WAIT_MS = 240;
  var kept = {};
  var waiting = null;
  var asked = 0;

  tv.parts.details = function (tile, done) {
    window.clearTimeout(waiting);
    asked += 1;
    var mine = asked;
    if (kept[tile.title]) {
      done(kept[tile.title]);
      return;
    }
    var key = kit.pair(tile.title);
    waiting = window.setTimeout(function () {
      kit.ask("GET", "/v1/titles/" + key.kind + "/" + key.id, function (status, text) {
        if (status < 200 || status >= 300) return;
        kept[tile.title] = JSON.parse(text);
        if (mine === asked) done(kept[tile.title]);
      });
    }, WAIT_MS);
  };
})();
