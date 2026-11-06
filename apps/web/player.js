// The one place that touches the video element and hls.js. It reports facts and decides nothing:
// the core picks every next step. The Rust host calls these functions by name.
(function () {
  "use strict";
  let hls = null;
  // One element for the life of the page. Browsers let a video start by itself only once a press
  // has touched it, and a play starts after a network answer, so a new element would stay paused.
  const video = document.createElement("video");
  video.playsInline = true;
  let generation = 0;
  let report = function () {};
  let firstFrame = false;
  // True from an open or a play until the film moves, so a start the browser dropped is tried
  // again as soon as there is something to show.
  let wanted = false;
  let lastPosition = -1;

  function say(event) {
    report(generation, JSON.stringify(event));
  }

  function classOf(detail) {
    const text = String(detail || "");
    if (text.indexOf("manifest") >= 0 || text.indexOf("level") >= 0) return "manifest";
    if (text.indexOf("frag") >= 0) return "segment";
    if (text.indexOf("key") >= 0) return "key";
    return "network";
  }

  function tracks() {
    const audio = hls ? hls.audioTracks.map(function (track, index) {
      return { id: String(index), lang: track.lang || undefined, label: track.name || undefined };
    }) : [];
    const text = hls ? hls.subtitleTracks.map(function (track, index) {
      return { id: String(index), lang: track.lang || undefined, label: track.name || undefined };
    }) : [];
    say({ kind: "tracks", audio: audio, text: text });
  }

  function listen(element) {
    element.addEventListener("canplay", function () {
      if (wanted && element.paused) start();
    });
    element.addEventListener("playing", function () {
      wanted = false;
      if (firstFrame) return;
      firstFrame = true;
      say({ kind: "first_frame" });
    });
    element.addEventListener("waiting", function () { say({ kind: "stalled" }); });
    element.addEventListener("ended", function () { say({ kind: "ended" }); });
    element.addEventListener("timeupdate", function () {
      const seconds = Math.floor(element.currentTime);
      if (seconds === lastPosition) return;
      lastPosition = seconds;
      const position = { kind: "position", time_ms: Math.floor(element.currentTime * 1000) };
      if (isFinite(element.duration)) position.duration_ms = Math.floor(element.duration * 1000);
      if (element.videoWidth) position.video_width = element.videoWidth;
      say(position);
    });
    element.addEventListener("error", function () {
      const code = element.error ? element.error.code : 0;
      say({ kind: "error", class: code === 3 ? "decode" : code === 4 ? "format" : "network" });
    });
  }

  function close() {
    wanted = false;
    if (hls) hls.destroy();
    hls = null;
    video.pause();
    video.removeAttribute("src");
    video.load();
  }

  function unlock() {
    video.load();
  }
  document.addEventListener("pointerdown", unlock, { once: true, capture: true });
  document.addEventListener("keydown", unlock, { once: true, capture: true });

  // Sound needs a press. Without one the film still starts, muted, and the page offers sound.
  function start() {
    wanted = true;
    const asked = video.play();
    if (!asked) return;
    asked.catch(function (why) {
      if (why && why.name === "NotAllowedError" && !video.muted) {
        video.muted = true;
        video.play().catch(function () {});
      }
    });
  }

  listen(video);

  window.webPlayer = {
    attach: function (stage, callback) {
      stage.appendChild(video);
      report = callback;
    },
    muted: function () { return video.muted; },
    unmute: function () { video.muted = false; },
    open: function (url, startMs, stamp) {
      close();
      generation = stamp;
      firstFrame = false;
      lastPosition = -1;
      const from = startMs / 1000;
      const streamed = url.split("?")[0].endsWith(".m3u8") || url.indexOf("m3u8") >= 0;
      if (streamed && window.Hls && window.Hls.isSupported()) {
        hls = new window.Hls({ startPosition: from > 0 ? from : -1, maxBufferLength: 30 });
        hls.on(window.Hls.Events.MANIFEST_PARSED, tracks);
        hls.on(window.Hls.Events.ERROR, function (_, data) {
          if (data.fatal) say({ kind: "error", class: classOf(data.details) });
        });
        hls.loadSource(url);
        hls.attachMedia(video);
      } else {
        video.src = url;
        if (from > 0) video.currentTime = from;
      }
      start();
    },
    play: start,
    pause: function () {
      wanted = false;
      video.pause();
    },
    seek: function (ms) { video.currentTime = ms / 1000; },
    rate: function (percent) { video.playbackRate = percent / 100; },
    volume: function (percent) { video.volume = percent / 100; },
    audio: function (track) { if (hls) hls.audioTrack = Number(track); },
    text: function (track) { if (hls) hls.subtitleTrack = track === null ? -1 : Number(track); },
    close: close,
    fullscreen: function () {
      const page = document.documentElement;
      if (document.fullscreenElement) document.exitFullscreen();
      else if (page.requestFullscreen) page.requestFullscreen();
    },
  };
})();
