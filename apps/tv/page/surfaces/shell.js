// The one place that decides which surface shows. The core says where the viewer is, and this
// picks the surface, hands it its data, and sends it the keys.
(function () {
  "use strict";
  var tv = window.tv;
  var kit = tv.kit;
  var VERSION = "0.1.0";
  var FAILURES = {
    no_engine: "This television cannot run the app.",
    no_file: "The app's core did not load from this server.",
    refused: "This television refused to start the app's core.",
    stopped: "The app stopped and could not start again."
  };
  var BROWSE = { home: true, kids_home: true, screen: true };
  var MESSAGES = {
    not_found: "That page is not here.",
    upgrade: "This app needs an update.",
    activate: "Open this address on your phone."
  };
  var notice = document.getElementById("notice");
  var route = null;
  var keys = null;
  var device = null;
  var people = [];
  var play = null;
  var screen = null;
  var detail = null;

  function setKeys(given) {
    keys = given;
  }

  function draw() {
    var root = kit.root();
    kit.empty(root);
    keys = null;
    if (!route) return;
    var target = route.target;
    if (target === "sign_in") keys = tv.surfaces.code(root, device);
    else if (target === "profiles") keys = tv.surfaces.people(root, people, setKeys);
    else if (BROWSE[target]) keys = tv.surfaces.browse(root, screen);
    else if (target === "title") keys = tv.surfaces.title(root, detail && detail.title === route.title ? detail : null);
    else if (target === "watch") keys = tv.surfaces.watch(root, function () { return play && play.state; });
    else if (MESSAGES[target]) root.appendChild(kit.make("div", "message", MESSAGES[target]));
  }

  function show(given) {
    var left = route && route.target;
    notice.className = "off";
    document.getElementById("stage").setAttribute("data-route", given.target);
    if (given.target === "start") {
      if (left === "sign_in") kit.act({ action: "device_sign_in_stop" });
      route = null;
      kit.empty(kit.root());
      kit.root().className = "off";
      tv.surfaces.door.enter();
      return;
    }
    tv.surfaces.door.leave();
    kit.root().className = "";
    route = given;
    if (left === "sign_in" && given.target !== "sign_in") kit.act({ action: "device_sign_in_stop" });
    if (given.target === "sign_in" && left !== "sign_in") {
      kit.act({ action: "device_sign_in", device_name: "Television" });
    }
    if (given.target === "home" || given.target === "kids_home") kit.atHome();
    if (!BROWSE[given.target]) screen = null;
    draw();
  }

  function press(word) {
    if (tv.surfaces.door.isOpen()) return tv.surfaces.door.press(word);
    if (keys && keys(word)) return true;
    if (word === "back" && route && route.target !== "home" && route.target !== "kids_home") {
      kit.back();
      return true;
    }
    return false;
  }

  function say(text) {
    notice.textContent = text;
    notice.className = "";
  }

  tv.ready();
  tv.ports.keys.listen(press);
  tv.ports.player.listen(function (generation, event) {
    window.tvHost.driver(generation, event);
    tv.surfaces.watch.tick(event);
  });
  window.tvHost.start({
    version: VERSION,
    os: "browser",
    view: function (view) {
      if (view.hello && view.hello.brand.name) document.title = view.hello.brand.name;
      if (view.play) play = view.play;
      if (view.profiles) {
        people = view.profiles.profiles;
        if (route && route.target === "profiles") draw();
      }
      if (view.device_sign_in) {
        device = view.device_sign_in;
        if (route && route.target === "sign_in") draw();
      }
      if (view.route) show(view.route);
    },
    notice: function (given) { say(given.text); },
    page: function (path, text) {
      if (path.indexOf("/v1/screens/") === 0) {
        screen = JSON.parse(text);
        if (route && BROWSE[route.target]) draw();
      } else if (path.indexOf("/titles/") >= 0) {
        detail = JSON.parse(text);
        if (route && route.target === "title") draw();
      }
    },
    player: function (command) { tv.ports.player.command(command); },
    failed: function (word) {
      tv.surfaces.door.leave();
      say(FAILURES[word]);
    }
  });
})();
