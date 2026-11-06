// The host for a television: it steps the core and performs what the core asks for. It holds no
// rules and no screens. Written plainly so every set that has WebAssembly can run it.
(function () {
  "use strict";

  var CONTRACT = "2026-11-05";
  var ABI = 1;
  var CORE_FILE = "core.wasm";
  // The first eight bytes of every module. A missing file comes back as a page with status 200,
  // because the server answers unknown paths with the website, so the status proves nothing.
  var CORE_HEADER = [0, 97, 115, 109, 1, 0, 0, 0];
  var LOAD_LIMIT_MS = 60000;
  var WAKE_CAP_MS = 60000;
  var ROUNDS_PER_TURN = 64;
  // The core gives no answer to an ask it drops or holds and then forgets. The longest wait for a
  // real answer is its request deadline of 60 s plus a token refresh.
  var ASK_LIMIT_MS = 90000;
  var RESTART_LIMIT = 3;

  var ui = null;
  var base = "";
  var kept = null;
  var x = null;
  var handle = 0;
  var encoder = null;
  var decoder = null;
  var run = 0;
  var restarts = 0;
  var queue = [];
  var running = false;
  var wake = null;
  var asks = 0;
  var waiting = {};
  var lastAddress = "/";

  function mono() {
    return Math.floor(performance.now());
  }

  function seed() {
    var bytes = window.crypto.getRandomValues(new Uint8Array(16));
    var text = "";
    for (var i = 0; i < bytes.length; i += 1) {
      text += (bytes[i] < 16 ? "0" : "") + bytes[i].toString(16);
    }
    return text;
  }

  function isCore(buffer) {
    if (!buffer || buffer.byteLength < CORE_HEADER.length) return false;
    var bytes = new Uint8Array(buffer, 0, CORE_HEADER.length);
    for (var i = 0; i < bytes.length; i += 1) {
      if (bytes[i] !== CORE_HEADER[i]) return false;
    }
    return true;
  }

  // The one place that decides how a core reaches this page. It gives `ready` the exports of a
  // fresh module, or `failed` one word: no_engine, no_file or refused. Compiling is the only true
  // test on a set, because the page policy gates a compile and WebAssembly.validate passes anyway.
  function loadCore(ready, failed) {
    function instantiate(bytes) {
      WebAssembly.instantiate(bytes, {}).then(function (made) {
        if (made.instance.exports.core_abi() === ABI) ready(made.instance.exports);
        else failed("refused");
      }).then(null, function (why) {
        console.log("the core did not start", String(why));
        x = null;
        failed("refused");
      });
    }
    if (typeof WebAssembly !== "object") {
      failed("no_engine");
      return;
    }
    if (kept) {
      instantiate(kept);
      return;
    }
    var xhr = new XMLHttpRequest();
    xhr.open("GET", CORE_FILE, true);
    xhr.responseType = "arraybuffer";
    xhr.timeout = LOAD_LIMIT_MS;
    xhr.onload = function () {
      if (!isCore(xhr.response)) {
        failed("no_file");
        return;
      }
      kept = xhr.response;
      instantiate(kept);
    };
    xhr.onerror = function () { failed("no_file"); };
    xhr.ontimeout = function () { failed("no_file"); };
    xhr.send(null);
  }

  function place(text) {
    var bytes = encoder.encode(text);
    var at = x.core_alloc(bytes.length);
    if (at === 0) throw new Error("the core has no memory left");
    new Uint8Array(x.memory.buffer, at, bytes.length).set(bytes);
    return { at: at, len: bytes.length };
  }

  function stepOnce(events) {
    var input = place(JSON.stringify({
      contract: CONTRACT,
      type: "core_in",
      now_ms: Math.floor(Date.now()),
      mono_ms: mono(),
      events: events
    }));
    var slots = x.core_alloc(8);
    var status = x.core_step(handle, input.at, input.len, slots, slots + 4);
    // Read memory only now: a step can grow it, which replaces the buffer.
    var memory = new DataView(x.memory.buffer);
    var at = memory.getUint32(slots, true);
    var len = memory.getUint32(slots + 4, true);
    x.core_free(slots, 8);
    x.core_free(input.at, input.len);
    if (status !== 0) throw new Error("the core refused a step, status " + status);
    var text = decoder.decode(new Uint8Array(x.memory.buffer, at, len));
    x.core_free(at, len);
    return JSON.parse(text);
  }

  // A fault in a screen must not stop the effects after it, or the core waits on them for ever.
  function tell(name, first, second) {
    try {
      if (ui[name]) ui[name](first, second);
    } catch (thrown) {
      console.log("the screen failed on " + name, String(thrown));
    }
  }

  function settle(id, status, text) {
    var ask = waiting[id];
    if (!ask) return;
    delete waiting[id];
    window.clearTimeout(ask.limit);
    try {
      ask.reply(status, text);
    } catch (thrown) {
      console.log("an ask reply failed", String(thrown));
    }
  }

  function secret(effect) {
    var done = { id: effect.id, type: "secret_done" };
    var store = window.tv.ports.secrets;
    try {
      if (effect.op === "get") {
        var value = store.get(effect.name);
        if (value !== null) done.value = value;
      } else if (effect.op === "put") {
        store.put(effect.name, effect.value || "");
      } else {
        store.drop(effect.name);
      }
    } catch (refused) {
      return { id: effect.id, type: "unsupported" };
    }
    return done;
  }

  function http(effect) {
    var thisRun = run;
    var call = {
      method: effect.method,
      url: base + effect.path,
      headers: effect.headers,
      body: effect.body,
      deadline_ms: effect.deadline_ms
    };
    window.tv.ports.network.send(call, function (status, text, reason) {
      if (thisRun !== run) return;
      if (reason) {
        settle(effect.id, 0, "");
        send([{ id: effect.id, type: "http_failed", reason: reason }]);
        return;
      }
      var done = { id: effect.id, type: "http_done", status: status };
      if (effect.to === "screen" && status >= 200 && status < 300) tell("page", effect.path, text);
      else if (text) done.body = text;
      settle(effect.id, status, text);
      send([done]);
    });
  }

  function perform(effect) {
    switch (effect.kind) {
      case "http":
        http(effect);
        break;
      case "secret":
        queue.push(secret(effect));
        break;
      case "player":
        tell("player", effect.command);
        break;
      case "view":
        if (effect.notice) tell("notice", effect.notice);
        break;
      case "open":
        // The same step carries this target in view.route, and a television has no address bar.
        break;
      case "log":
        console.log(effect.code, effect.detail || "");
        break;
      default:
        // Store and timer come only from the offline layer, which a television does not load.
        queue.push({ id: effect.id, type: "unsupported" });
    }
  }

  function arm(at) {
    window.clearTimeout(wake);
    wake = null;
    if (at === undefined) return;
    var after = Math.max(0, Math.min(WAKE_CAP_MS, at - mono()));
    wake = window.setTimeout(function () { send([]); }, after);
  }

  function apply(out) {
    if (out.view) tell("view", out.view);
    arm(out.next_wake_ms);
    for (var i = 0; i < out.effects.length; i += 1) perform(out.effects[i]);
  }

  // One caller at a time: a screen that steps the core from inside a step only adds to the queue,
  // and the loop below picks that up in order.
  function send(events) {
    if (!x) return;
    for (var i = 0; i < events.length; i += 1) queue.push(events[i]);
    if (running) return;
    running = true;
    var rounds = 0;
    try {
      do {
        var batch = queue;
        queue = [];
        apply(stepOnce(batch));
        rounds += 1;
      } while (queue.length > 0 && rounds < ROUNDS_PER_TURN);
    } catch (thrown) {
      // A trap leaves nothing in the module to trust, and a refused step lost its whole batch, so
      // the core may be waiting on answers it will never get. Both start again.
      running = false;
      console.log("the core stopped", String(thrown));
      restart();
      return;
    }
    running = false;
    if (queue.length > 0) window.setTimeout(function () { send([]); }, 0);
  }

  function restart() {
    x = null;
    run += 1;
    queue = [];
    window.clearTimeout(wake);
    for (var id in waiting) settle(id, 0, "");
    restarts += 1;
    if (restarts > RESTART_LIMIT) {
      tell("failed", "stopped");
      return;
    }
    loadCore(open, function (word) { tell("failed", word); });
  }

  function open(exports) {
    var config = {
      contract: CONTRACT,
      type: "core_config",
      "class": "tv",
      form: "tv",
      mode: "local",
      app: "tv",
      version: ui.version,
      os: ui.os,
      seed: seed()
    };
    x = exports;
    encoder = new TextEncoder();
    decoder = new TextDecoder();
    var placed = place(JSON.stringify(config));
    handle = x.core_open(placed.at, placed.len);
    x.core_free(placed.at, placed.len);
    if (handle === 0) {
      x = null;
      tell("failed", "refused");
      return;
    }
    // The first step carries the address, so a core started again never shows the door on its way
    // back. The core keeps an address that comes while it is still starting.
    go(lastAddress);
  }

  // `address` is one of the core's own addresses, such as "/home", and never this page's path.
  function go(address) {
    lastAddress = address;
    send([{ type: "address", address: address }]);
  }

  function act(action) {
    send([{ type: "action", action: action }]);
  }

  // `reply` gets the status and the text, or status 0 when no answer came.
  function ask(method, path, body, reply) {
    asks += 1;
    var id = "ask-" + asks;
    var sendable = x !== null && path.indexOf("/v1/") === 0;
    var action = { action: "request", method: method, path: path };
    if (body !== null && body !== undefined) action.body = body;
    waiting[id] = {
      reply: reply,
      limit: window.setTimeout(function () { settle(id, 0, ""); }, sendable ? ASK_LIMIT_MS : 0)
    };
    if (sendable) send([{ id: id, type: "action", action: action }]);
  }

  function driver(generation, event) {
    send([{ type: "driver", generation: generation, event: event }]);
  }

  // `given` holds version, os, base and the calls view, notice, page, player and failed.
  function start(given) {
    ui = given;
    base = given.base || "";
    document.addEventListener("keydown", function () { send([{ type: "activity" }]); });
    document.addEventListener("visibilitychange", function () {
      send([{ type: "visibility", visible: !document.hidden }]);
    });
    window.addEventListener("online", function () { send([{ type: "network", online: true }]); });
    window.addEventListener("offline", function () { send([{ type: "network", online: false }]); });
    loadCore(open, function (word) { tell("failed", word); });
  }

  window.tvHost = { start: start, go: go, act: act, ask: ask, driver: driver };
})();
