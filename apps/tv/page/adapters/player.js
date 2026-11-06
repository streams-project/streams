// The player port, on one video element with hls.js where the set has no HLS of its own. It reports
// facts and decides nothing, and the core picks every next step.
(function () {
  "use strict";
  var movie = document.createElement("video");
  movie.id = "movie";
  movie.setAttribute("playsinline", "playsinline");

  var hls = null;
  var generation = 0;
  var firstFrame = false;
  var wanted = false;
  var lastSecond = -1;
  var report = function () {};

  function say(event) {
    report(generation, event);
  }

  function closeFilm() {
    wanted = false;
    if (hls) hls.destroy();
    hls = null;
    movie.pause();
    movie.removeAttribute("src");
    movie.load();
  }

  function startFilm() {
    wanted = true;
    var asked = movie.play();
    if (asked && asked.then) {
      asked.then(null, function (why) {
        if (why && why.name === "NotAllowedError" && !movie.muted) {
          movie.muted = true;
          movie.play();
        }
      });
    }
  }

  function errorClass(detail_) {
    var text = String(detail_ || "");
    if (text.indexOf("manifest") >= 0 || text.indexOf("level") >= 0) return "manifest";
    if (text.indexOf("frag") >= 0) return "segment";
    if (text.indexOf("key") >= 0) return "key";
    return "network";
  }

  function openFilm(command) {
    closeFilm();
    generation = command.generation;
    firstFrame = false;
    lastSecond = -1;
    var from = (command.start_at_ms || 0) / 1000;
    var url = command.url;
    if (url.indexOf("m3u8") >= 0 && window.Hls && window.Hls.isSupported()) {
      hls = new window.Hls({ startPosition: from > 0 ? from : -1, maxBufferLength: 30 });
      hls.on(window.Hls.Events.ERROR, function (name, data) {
        if (data.fatal) say({ kind: "error", "class": errorClass(data.details) });
      });
      hls.loadSource(url);
      hls.attachMedia(movie);
    } else {
      movie.src = url;
      if (from > 0) movie.currentTime = from;
    }
    startFilm();
  }

  function command(given) {
    switch (given.op) {
      case "open": openFilm(given); break;
      case "play": startFilm(); break;
      case "pause":
        wanted = false;
        movie.pause();
        break;
      case "seek": movie.currentTime = given.to_ms / 1000; break;
      case "rate": movie.playbackRate = given.percent / 100; break;
      case "volume": movie.volume = given.percent / 100; break;
      case "select_audio": if (hls) hls.audioTrack = Number(given.track); break;
      case "select_text": if (hls) hls.subtitleTrack = given.track === undefined ? -1 : Number(given.track); break;
      case "close": closeFilm(); break;
      default: break;
    }
  }

  movie.addEventListener("canplay", function () {
    if (wanted && movie.paused) startFilm();
  });
  movie.addEventListener("playing", function () {
    wanted = false;
    if (firstFrame) return;
    firstFrame = true;
    say({ kind: "first_frame" });
  });
  movie.addEventListener("waiting", function () { say({ kind: "stalled" }); });
  movie.addEventListener("ended", function () { say({ kind: "ended" }); });
  movie.addEventListener("error", function () {
    var code = movie.error ? movie.error.code : 0;
    say({ kind: "error", "class": code === 3 ? "decode" : code === 4 ? "format" : "network" });
  });
  movie.addEventListener("timeupdate", function () {
    var second = Math.floor(movie.currentTime);
    if (second === lastSecond) return;
    lastSecond = second;
    var event = { kind: "position", time_ms: Math.floor(movie.currentTime * 1000) };
    if (isFinite(movie.duration)) event.duration_ms = Math.floor(movie.duration * 1000);
    if (movie.videoWidth) event.video_width = movie.videoWidth;
    say(event);
  });

  window.tv.plug("player", {
    command: command,
    listen: function (given) { report = given; },
    mount: function (node) { node.appendChild(movie); }
  });
})();
