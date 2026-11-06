// The keys port, for a remote. Every code a brand sends lives here, so a new brand touches one file.
(function () {
  "use strict";
  // The two television back codes are from memory, not a real set.
  var WORDS = {
    37: "left", 38: "up", 39: "right", 40: "down", 13: "ok",
    10009: "back", 461: "back", 27: "back", 8: "back",
    415: "play", 19: "pause", 413: "stop", 412: "rewind", 417: "forward"
  };
  window.tv.plug("keys", {
    listen: function (press) {
      document.addEventListener("keydown", function (event) {
        var code = event.keyCode;
        var word = WORDS[code] || (code >= 48 && code <= 57 ? String(code - 48) : null);
        if (word && press(word)) event.preventDefault();
      });
    }
  });
})();
