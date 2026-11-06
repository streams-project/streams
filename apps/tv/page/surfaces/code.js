// The code a phone approves. The core keeps the code alive, so this only draws it.
(function () {
  "use strict";
  var tv = window.tv;
  var kit = tv.kit;
  var make = kit.make;

  function drawQr(canvas, text) {
    var grid = window.tvQr(text);
    var context = canvas.getContext("2d");
    context.fillStyle = "#fff";
    context.fillRect(0, 0, canvas.width, canvas.height);
    if (!grid) return;
    var cell = Math.floor(canvas.width / (grid.length + 8));
    var edge = Math.floor((canvas.width - cell * grid.length) / 2);
    context.fillStyle = "#000";
    for (var y = 0; y < grid.length; y += 1) {
      for (var x = 0; x < grid.length; x += 1) {
        if (grid[y][x]) context.fillRect(edge + x * cell, edge + y * cell, cell, cell);
      }
    }
  }

  tv.surfaces.code = function (root, device) {
    var panel = make("div", "code");
    var state = device ? device.state : "waiting";
    if (state === "off") {
      panel.appendChild(make("h1", "", "This server cannot sign in a television yet."));
      root.appendChild(panel);
      return null;
    }
    var side = make("div", "code-side");
    side.appendChild(make("h1", "", "Sign in on your phone"));
    if (device && device.user_code) {
      side.appendChild(make("p", "", "Scan the code, or go to"));
      side.appendChild(make("p", "code-address", device.verification_uri || ""));
      side.appendChild(make("p", "", "and enter"));
      side.appendChild(make("p", "code-value", device.user_code));
    } else if (state === "denied") {
      side.appendChild(make("p", "", "That was refused. Press OK to try again."));
    } else {
      side.appendChild(make("p", "", "Getting a code..."));
    }
    panel.appendChild(side);
    if (device && device.verification_uri_complete) {
      var canvas = make("canvas", "code-qr");
      canvas.width = 460;
      canvas.height = 460;
      panel.appendChild(canvas);
      drawQr(canvas, device.verification_uri_complete);
    }
    root.appendChild(panel);
    return function (word) {
      if (word === "ok" && state === "denied") {
        kit.act({ action: "device_sign_in", device_name: "Television" });
        return true;
      }
      return false;
    };
  };
})();
