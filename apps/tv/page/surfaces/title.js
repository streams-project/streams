// One title: its facts and Play. A series needs an episode, so Play starts the first one.
(function () {
  "use strict";
  var tv = window.tv;
  var kit = tv.kit;
  var make = kit.make;

  function play(data) {
    var key = kit.pair(data.title);
    if (key.kind !== "tv") {
      kit.go("/watch/" + key.kind + "/" + key.id);
      return;
    }
    var number = data.seasons && data.seasons.length ? data.seasons[0].number : 1;
    kit.ask("GET", "/v1/titles/" + key.kind + "/" + key.id + "/seasons/" + number, function (status, text) {
      var first = 1;
      if (status >= 200 && status < 300) {
        var season = JSON.parse(text);
        if (season.episodes && season.episodes.length) first = season.episodes[0].number;
      }
      kit.go("/watch/" + key.kind + "/" + key.id + "/" + number + "/" + first);
    });
  }

  tv.surfaces.title = function (root, data) {
    if (!data) return null;
    var page = make("div", "title");
    var art = make("div", "hero-art");
    kit.art(art, kit.picture(data.images));
    page.appendChild(art);
    page.appendChild(make("div", "hero-shade"));
    var text = make("div", "title-text");
    text.appendChild(make("h1", "", data.name));
    var facts = [];
    if (data.year) facts.push(String(data.year));
    if (data.runtime_min) facts.push(Math.floor(data.runtime_min / 60) + "h " + data.runtime_min % 60 + "m");
    if (data.seasons && data.seasons.length) {
      facts.push(data.seasons.length + (data.seasons.length > 1 ? " seasons" : " season"));
    }
    text.appendChild(make("p", "title-facts", facts.join(" · ")));
    if (data.overview) text.appendChild(make("p", "title-overview", data.overview));
    var buttons = make("div", "hero-buttons");
    buttons.appendChild(make("span", "button on", "Play"));
    text.appendChild(buttons);
    page.appendChild(text);
    root.appendChild(page);
    return function (word) {
      if (word === "ok") {
        play(data);
        return true;
      }
      return false;
    };
  };
})();
