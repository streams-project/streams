// The hexagon of the television family. The core sits in the middle, and everything it needs from
// a set goes through one of these ports. Each port has adapters in adapters/, and the surfaces in
// surfaces/ may use a port but never a browser feature behind one. ./check enforces that.
(function () {
  "use strict";
  var SHAPES = {
    // Runs one http effect from the core. send(effect, done) calls done(status, text) or
    // done(0, "", reason), where reason is offline, timeout or cancelled.
    network: ["send"],
    // Holds what the core keeps between runs. get(name), put(name, value), drop(name).
    secrets: ["get", "put", "drop"],
    // Plays films. command(given) takes a driver command, and listen(report) gets driver events.
    // mount(node) puts the picture inside a surface.
    player: ["command", "listen", "mount"],
    // Turns remote presses into words: left, right, up, down, ok, back, play, pause, stop, rewind,
    // forward, or a digit. listen(press) calls press(word) and uses the press if that says true.
    keys: ["listen"]
  };

  var tv = { ports: {}, surfaces: {} };

  // Called by each adapter. A missing call stops the app here and not in front of a viewer.
  tv.plug = function (port, adapter) {
    var shape = SHAPES[port];
    if (!shape) throw new Error("no port named " + port);
    for (var i = 0; i < shape.length; i += 1) {
      if (typeof adapter[shape[i]] !== "function") throw new Error(port + " needs " + shape[i]);
    }
    tv.ports[port] = adapter;
  };

  tv.ready = function () {
    for (var port in SHAPES) {
      if (!tv.ports[port]) throw new Error("nothing plugged into " + port);
    }
  };

  window.tv = tv;
})();
