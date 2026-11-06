// The row that owns the screen: its focused title opens wide, the next ones stand as posters beside
// it, the one before peeks in at the edge, and the title's facts sit under the row.
(function () {
  "use strict";
  var tv = window.tv;
  var kit = tv.kit;
  var make = kit.make;
  var STEP = 328;
  var AHEAD = 5;

  tv.parts.openRow = function () {
    var root = make("div", "row-now off");
    var label = make("h2");
    var card = make("div", "open-card");
    var art = make("div", "open-art");
    var clip = make("div", "open-clip");
    var shade = make("div", "open-shade");
    var name = make("p", "open-name");
    var ring = make("div", "open-ring");
    card.appendChild(art);
    card.appendChild(clip);
    card.appendChild(shade);
    card.appendChild(name);
    card.appendChild(ring);
    var before = make("div", "poster-before");
    var strip = make("div", "posters");
    var facts = make("p", "facts row-facts");
    var text = make("p", "overview row-text");
    root.appendChild(label);
    root.appendChild(before);
    root.appendChild(strip);
    root.appendChild(card);
    root.appendChild(facts);
    root.appendChild(text);
    var held = {};
    var shown = null;

    function fill(section, at) {
      var tiles = section.tiles;
      if (shown !== section) {
        kit.empty(strip);
        held = {};
        shown = section;
      }
      label.textContent = section.title || "";
      for (var kept in held) {
        if (Number(kept) <= at || Number(kept) > at + AHEAD) {
          strip.removeChild(held[kept]);
          delete held[kept];
        }
      }
      for (var i = at + 1; i <= at + AHEAD && i < tiles.length; i += 1) {
        if (!held[i]) {
          held[i] = tv.parts.poster(tiles[i]);
          held[i].style.left = i * STEP + "px";
          strip.appendChild(held[i]);
        }
      }
      strip.style.transform = "translateX(" + -(at + 1) * STEP + "px)";
      kit.empty(before);
      if (at > 0) before.appendChild(tv.parts.poster(tiles[at - 1]));
      var tile = tiles[at];
      kit.art(art, tv.parts.wide(tile));
      name.textContent = tile.name;
      kit.empty(facts);
      text.textContent = "";
      tv.ports.clips.find(tile.title, function () {});
      tv.parts.details(tile, function (detail) {
        tv.parts.facts(facts, detail);
        text.textContent = detail.overview || "";
      });
    }

    return {
      node: root,
      draw: function (on, section, at) {
        root.className = on ? "row-now" : "row-now off";
        if (on) fill(section, at);
      }
    };
  };

  // The next row, dim, cut by the screen's edge.
  tv.parts.nextRow = function () {
    var POSTERS = 6;
    var root = make("div", "row-next");
    var label = make("h2");
    var strip = make("div", "next-posters");
    root.appendChild(label);
    root.appendChild(strip);
    return {
      node: root,
      draw: function (section, high) {
        kit.empty(strip);
        if (!section) {
          root.className = "row-next off";
          return;
        }
        root.className = high ? "row-next high" : "row-next";
        label.textContent = section.title || "";
        for (var i = 0; i < section.tiles.length && i < POSTERS; i += 1) {
          var node = tv.parts.poster(section.tiles[i]);
          node.style.left = i * STEP + "px";
          strip.appendChild(node);
        }
      }
    };
  };
})();
