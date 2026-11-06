// The television page. The core says where the viewer is and host.js carries its messages, so
// this file only draws the route it is given. Written plainly so the oldest sets can run it.
(function () {
  "use strict";
  var VERSION = "0.1.0";
  var FRAME = 1000 / 24;
  // Share of the picture width the card still has to travel, one value per frame, as measured.
  var SLIDE = [0.97, 0.647, 0.323, 0.106, 0.077, 0.039, 0.017, 0.005, 0.001, 0];
  var EMPTY_CARD_MS = 333;
  var PEEK_AFTER_MS = 10000;
  // Back on Samsung, LG and a keyboard. The two television codes are from memory, not a real set.
  var BACK_KEYS = [10009, 461, 27, 8];
  var FAILURES = {
    no_engine: "This television cannot run the app.",
    no_file: "The app's core did not load from this server.",
    refused: "This television refused to start the app's core.",
    stopped: "The app stopped and could not start again."
  };

  var stage = document.getElementById("stage");
  var door = document.getElementById("door");
  var card = document.getElementById("card");
  var film = document.getElementById("film");
  var opening = document.getElementById("opening");
  var peek = document.getElementById("peek");
  var notice = document.getElementById("notice");
  var order = [
    document.querySelector(".round"),
    document.querySelector(".tab"),
    document.getElementById("help"),
    document.getElementById("sign-in"),
    document.getElementById("join")
  ];
  var at = 3;
  var inDoor = false;
  var openingPlayed = false;
  var timers = [];
  var frame = 0;

  function fit() {
    var scale = Math.min(window.innerWidth / 1920, window.innerHeight / 1080);
    var left = (window.innerWidth - 1920 * scale) / 2;
    var top = (window.innerHeight - 1080 * scale) / 2;
    stage.style.transform = "translate(" + left + "px," + top + "px) scale(" + scale + ")";
  }

  function focus(index) {
    order[at].className = order[at].className.replace(/ ?\bon\b/, "");
    at = index;
    order[at].className += " on";
  }

  function play(video) {
    var asked = video.play();
    if (asked && asked.catch) {
      asked.catch(function (why) {
        if (inDoor && why && why.name === "NotAllowedError" && !video.muted) {
          video.muted = true;
          video.play().catch(function () {});
        }
      });
    }
  }

  // Dropping the source gives the decoder back. A television has few, and the player needs one.
  function clear(video) {
    video.pause();
    video.removeAttribute("src");
    video.load();
  }

  function later(work, after) {
    timers.push(window.setTimeout(work, after));
  }

  function slideIn(done) {
    var began = null;
    function step(now) {
      if (began === null) began = now;
      var index = Math.floor((now - began) / FRAME);
      var left = SLIDE[Math.min(index, SLIDE.length - 1)];
      card.style.transform = "translateX(" + left * 1920 + "px)";
      if (index < SLIDE.length - 1) frame = window.requestAnimationFrame(step);
      else done();
    }
    frame = window.requestAnimationFrame(step);
  }

  function landing() {
    opening.className = "done";
    clear(opening);
    focus(at);
    slideIn(function () {
      later(function () { play(film); }, EMPTY_CARD_MS);
      later(function () { peek.className = "seen"; }, PEEK_AFTER_MS);
    });
  }

  function openingOver() {
    if (inDoor && opening.className !== "done") landing();
  }

  // The opening plays once for each load of the page, as it did when the door was the whole page.
  function enterDoor() {
    if (inDoor) return;
    inDoor = true;
    door.className = "";
    film.src = "showreel.mp4";
    if (openingPlayed) {
      landing();
      return;
    }
    openingPlayed = true;
    card.style.transform = "translateX(1862px)";
    opening.className = "";
    opening.src = "opening.mp4";
    play(opening);
  }

  function leaveDoor() {
    if (!inDoor) return;
    inDoor = false;
    window.cancelAnimationFrame(frame);
    for (var i = 0; i < timers.length; i += 1) window.clearTimeout(timers[i]);
    timers = [];
    clear(opening);
    clear(film);
    opening.className = "done";
    card.style.transform = "";
    peek.className = "";
    door.className = "off";
  }

  function say(text) {
    notice.textContent = text;
    notice.className = "";
  }

  function show(route) {
    stage.setAttribute("data-route", route.target);
    notice.className = "off";
    if (route.target === "start") enterDoor();
    else leaveDoor();
  }

  function pressInDoor(key) {
    if (key === 37 && at > 0) focus(at - 1);
    else if (key === 39 && at < order.length - 1) focus(at + 1);
    else if (key === 38 && at >= 2) focus(1);
    else if (key === 40 && at < 2) focus(3);
    else if (key === 13) {
      var go = order[at].getAttribute("data-go");
      if (go) window.tvHost.go(go);
    } else return false;
    return true;
  }

  function press(event) {
    var key = event.keyCode;
    var used = false;
    if (inDoor) {
      used = pressInDoor(key);
    } else if (BACK_KEYS.indexOf(key) >= 0) {
      window.tvHost.go("/");
      used = true;
    }
    if (used) event.preventDefault();
  }

  window.addEventListener("resize", fit);
  document.addEventListener("keydown", press);
  opening.addEventListener("ended", openingOver);
  opening.addEventListener("error", openingOver);
  fit();
  window.tvHost.start({
    version: VERSION,
    os: "browser",
    view: function (view) {
      if (view.hello && view.hello.brand.name) document.title = view.hello.brand.name;
      if (view.route) show(view.route);
    },
    notice: function (given) { say(given.text); },
    player: function (command) {
      // No player until the first slice. Saying so lets the core stop waiting on a start.
      if (command.op === "open") {
        window.tvHost.driver(command.generation, { kind: "error", "class": "format" });
      }
    },
    failed: function (word) {
      leaveDoor();
      say(FAILURES[word]);
    }
  });
})();
