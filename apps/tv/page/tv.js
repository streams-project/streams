// The signed out landing for a television. Written plainly so the oldest sets can run it.
(function () {
  "use strict";
  var FRAME = 1000 / 24;
  // Share of the picture width the card still has to travel, one value per frame, as measured.
  var SLIDE = [0.97, 0.647, 0.323, 0.106, 0.077, 0.039, 0.017, 0.005, 0.001, 0];
  var EMPTY_CARD_MS = 333;
  var PEEK_AFTER_MS = 10000;

  var stage = document.getElementById("stage");
  var card = document.getElementById("card");
  var film = document.getElementById("film");
  var opening = document.getElementById("opening");
  var peek = document.getElementById("peek");
  var order = [
    document.querySelector(".round"),
    document.querySelector(".tab"),
    document.getElementById("help"),
    document.getElementById("sign-in"),
    document.getElementById("join")
  ];
  var at = 3;

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

  function start(video) {
    var asked = video.play();
    if (asked && asked.catch) {
      asked.catch(function () {
        video.muted = true;
        video.play().catch(function () {});
      });
    }
  }

  function slideIn(done) {
    var began = null;
    function step(now) {
      if (began === null) began = now;
      var frame = Math.floor((now - began) / FRAME);
      var left = SLIDE[Math.min(frame, SLIDE.length - 1)];
      var move = "translateX(" + left * 1920 + "px)";
      card.style.transform = move;
      if (frame < SLIDE.length - 1) window.requestAnimationFrame(step);
      else done();
    }
    window.requestAnimationFrame(step);
  }

  function landing() {
    opening.className = "done";
    opening.pause();
    focus(at);
    slideIn(function () {
      window.setTimeout(function () { start(film); }, EMPTY_CARD_MS);
      window.setTimeout(function () { peek.className = "seen"; }, PEEK_AFTER_MS);
    });
  }

  function press(event) {
    var key = event.keyCode;
    if (key === 37 && at > 0) focus(at - 1);
    else if (key === 39 && at < order.length - 1) focus(at + 1);
    else if (key === 38 && at >= 2) focus(1);
    else if (key === 40 && at < 2) focus(3);
    else if (key === 13) {
      var go = order[at].getAttribute("data-go");
      if (go) window.location.href = go;
    } else return;
    event.preventDefault();
  }

  function begin() {
    card.style.transform = "translateX(1862px)";
    opening.addEventListener("ended", landing);
    opening.addEventListener("error", landing);
    start(opening);
  }

  window.addEventListener("resize", fit);
  document.addEventListener("keydown", press);
  fetch("/v1/hello", { headers: { "api-version": "2026-11-05" } })
    .then(function (answer) { return answer.json(); })
    .then(function (hello) { document.title = hello.brand.name; })
    .catch(function () {});
  fit();
  begin();
})();
