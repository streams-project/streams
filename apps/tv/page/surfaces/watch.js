// The film: the player's picture, and controls that wake on a press and sleep after three seconds.
(function () {
  "use strict";
  var tv = window.tv;
  var kit = tv.kit;
  var make = kit.make;
  var CONTROLS_MS = 3000;
  var SEEK_MS = 10000;
  var position = 0;
  var length = 0;
  var draw = null;

  function clock(ms) {
    var seconds = Math.floor(ms / 1000);
    var hours = Math.floor(seconds / 3600);
    var minutes = Math.floor(seconds / 60) % 60;
    var rest = seconds % 60;
    var tail = (rest < 10 ? "0" : "") + rest;
    if (hours > 0) return hours + ":" + (minutes < 10 ? "0" : "") + minutes + ":" + tail;
    return minutes + ":" + tail;
  }

  tv.surfaces.watch = function (root, state) {
    var page = make("div", "watch");
    tv.ports.player.mount(page);
    var controls = make("div", "controls");
    var line = make("div", "controls-line");
    var done = make("span");
    line.appendChild(done);
    var times = make("p", "controls-time", "");
    controls.appendChild(line);
    controls.appendChild(times);
    page.appendChild(controls);
    root.appendChild(page);
    var hide = null;

    function wake() {
      controls.className = "controls";
      window.clearTimeout(hide);
      hide = window.setTimeout(function () { controls.className = "controls off"; }, CONTROLS_MS);
    }
    draw = function () {
      done.style.width = length ? (position / length) * 100 + "%" : "0";
      times.textContent = clock(position) + (length ? " / " + clock(length) : "");
    };
    wake();
    return function (word) {
      wake();
      if (word === "ok" || word === "play" || word === "pause") {
        kit.act(state() === "playing" ? { action: "pause" } : { action: "unpause" });
      } else if (word === "left" || word === "rewind") {
        kit.act({ action: "seek", to_ms: Math.max(0, position - SEEK_MS) });
      } else if (word === "right" || word === "forward") {
        kit.act({ action: "seek", to_ms: position + SEEK_MS });
      } else if (word === "back" || word === "stop") {
        kit.act({ action: "stop" });
        draw = null;
        kit.back();
      } else return false;
      return true;
    };
  };

  tv.surfaces.watch.tick = function (event) {
    if (event.kind !== "position") return;
    position = event.time_ms;
    if (event.duration_ms) length = event.duration_ms;
    if (draw) draw();
  };
})();
