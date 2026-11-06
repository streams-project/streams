// What every surface shares: small drawing helpers, and the only ways to reach the core.
(function () {
  "use strict";
  var tv = window.tv;
  var root = null;
  var trail = [];
  var here = "/home";

  function make(tag, name, text) {
    var node = document.createElement(tag);
    if (name) node.className = name;
    if (text !== undefined) node.textContent = text;
    return node;
  }

  function empty(node) {
    while (node.firstChild) node.removeChild(node.firstChild);
  }

  tv.kit = {
    make: make,
    empty: empty,
    root: function () {
      if (!root) {
        root = make("div");
        root.id = "screens";
        root.className = "off";
        document.getElementById("stage").appendChild(root);
      }
      return root;
    },
    act: function (action) { window.tvHost.act(action); },
    ask: function (method, path, reply) { window.tvHost.ask(method, path, null, reply); },
    go: function (address) {
      trail.push(here);
      here = address;
      window.tvHost.go(address);
    },
    back: function () {
      here = trail.length ? trail.pop() : "/home";
      window.tvHost.go(here);
    },
    atHome: function () {
      trail = [];
      here = "/home";
    },
    pair: function (key) {
      var parts = String(key).split(":");
      return { kind: parts[0], id: parts[1] };
    },
    picture: function (images) {
      var roles = ["backdrop", "still", "poster"];
      if (!images) return null;
      for (var r = 0; r < roles.length; r += 1) {
        for (var i = 0; i < images.length; i += 1) {
          if (images[i].role === roles[r]) return images[i].url;
        }
      }
      return null;
    },
    art: function (node, url) {
      node.style.backgroundImage = url ? "url('" + url + "')" : "";
    }
  };
})();
