// Rows of films: a hero that follows the focus, and windowed rows under one fixed ring, so a set
// keeps few elements alive however long the page is.
(function () {
  "use strict";
  var tv = window.tv;
  var kit = tv.kit;
  var make = kit.make;
  var pair = kit.pair;
  var picture = kit.picture;
  var art = kit.art;
  var go = kit.go;
  var TILE_STEP = 364;
  var ROW_H = 290;
  var ROWS_TOP = 640;
  var ROWS_BEHIND = 1;
  var ROWS_AHEAD = 2;
  var TILES_BEHIND = 2;
  var TILES_AHEAD = 6;
  var HERO_WAIT_MS = 220;
  var ROW_KINDS = { row: true, ranked: true, "continue": true, grid: true };

  function playTile(tile) {
    var key = pair(tile.title);
    if (key.kind === "tv") go("/title/tv/" + key.id);
    else go("/watch/" + key.kind + "/" + key.id);
  }

  tv.surfaces.browse = function (root, screenData) {
    if (!screenData) return null;
    var rows = [];
    var lead = null;
    for (var s = 0; s < screenData.sections.length; s += 1) {
      var section = screenData.sections[s];
      if (section.kind === "hero" && section.tiles && section.tiles.length) lead = section.tiles[0];
      else if (ROW_KINDS[section.kind] && section.tiles && section.tiles.length) rows.push(section);
    }
    if (!lead && rows.length) lead = rows[0].tiles[0];
    if (!lead) {
      root.appendChild(make("div", "message", "Nothing here yet."));
      return null;
    }
    var page = make("div", "browse");
    var back_art = make("div", "hero-art");
    var shade = make("div", "hero-shade");
    var text = make("div", "hero-text");
    var name = make("h1", "", lead.name);
    var tags = make("p", "", "");
    var buttons = make("div", "hero-buttons");
    var playButton = make("span", "button", "Play");
    var infoButton = make("span", "button quiet", "More info");
    buttons.appendChild(playButton);
    buttons.appendChild(infoButton);
    text.appendChild(name);
    text.appendChild(tags);
    text.appendChild(buttons);
    var list = make("div", "rows");
    var ring = make("div", "ring");
    page.appendChild(back_art);
    page.appendChild(shade);
    page.appendChild(text);
    page.appendChild(list);
    page.appendChild(ring);
    root.appendChild(page);

    var row = -1;
    var button = 0;
    var cols = [];
    var nodes = [];
    var waiting = null;
    var shownTile = null;

    function hero(tile) {
      if (tile === shownTile) return;
      shownTile = tile;
      name.textContent = tile.name;
      tags.textContent = tile.tags ? tile.tags.join(" · ") : "";
      window.clearTimeout(waiting);
      waiting = window.setTimeout(function () { art(back_art, picture(tile.images)); }, HERO_WAIT_MS);
    }

    function tileNode(tile, index) {
      var node = make("div", "tile");
      node.style.left = index * TILE_STEP + "px";
      var url = picture(tile.images);
      art(node, url);
      if (!url) node.appendChild(make("span", "tile-name", tile.name));
      if (tile.percent) {
        var meter = make("span", "tile-meter");
        var done = make("span");
        done.style.width = tile.percent + "%";
        meter.appendChild(done);
        node.appendChild(meter);
      }
      return node;
    }

    function fill(index) {
      var box = nodes[index];
      var tiles = rows[index].tiles;
      var from = Math.max(0, (cols[index] || 0) - TILES_BEHIND);
      var to = Math.min(tiles.length - 1, (cols[index] || 0) + TILES_AHEAD);
      for (var kept in box.tiles) {
        if (Number(kept) < from || Number(kept) > to) {
          box.strip.removeChild(box.tiles[kept]);
          delete box.tiles[kept];
        }
      }
      for (var t = from; t <= to; t += 1) {
        if (!box.tiles[t]) {
          box.tiles[t] = tileNode(tiles[t], t);
          box.strip.appendChild(box.tiles[t]);
        }
      }
      box.strip.style.transform = "translateX(" + -(cols[index] || 0) * TILE_STEP + "px)";
    }

    function place() {
      var from = Math.max(0, row - ROWS_BEHIND);
      var to = Math.min(rows.length - 1, row + ROWS_AHEAD);
      for (var r = 0; r < rows.length; r += 1) {
        var inside = r >= from && r <= to;
        if (inside && !nodes[r]) {
          var box = make("div", "row");
          box.style.top = r * ROW_H + "px";
          box.appendChild(make("h2", "", rows[r].title || ""));
          var strip = make("div", "strip");
          box.appendChild(strip);
          list.appendChild(box);
          nodes[r] = { box: box, strip: strip, tiles: {} };
        }
        if (!inside && nodes[r]) {
          list.removeChild(nodes[r].box);
          nodes[r] = null;
        }
        if (inside) fill(r);
      }
      var on = row >= 0;
      list.style.transform = "translateY(" + (on ? ROWS_TOP - row * ROW_H : ROWS_TOP + 130) + "px)";
      ring.className = on ? "ring" : "ring off";
      text.className = on ? "hero-text low" : "hero-text";
      buttons.className = on ? "hero-buttons off" : "hero-buttons";
      playButton.className = !on && button === 0 ? "button on" : "button";
      infoButton.className = !on && button === 1 ? "button quiet on" : "button quiet";
      hero(on ? rows[row].tiles[cols[row] || 0] : lead);
    }

    place();
    return function (word) {
      if (word === "down" && row < rows.length - 1) row += 1;
      else if (word === "up" && row >= 0) row -= 1;
      else if (word === "left") {
        if (row < 0) button = 0;
        else if ((cols[row] || 0) > 0) cols[row] = (cols[row] || 0) - 1;
      } else if (word === "right") {
        if (row < 0) button = 1;
        else if ((cols[row] || 0) < rows[row].tiles.length - 1) cols[row] = (cols[row] || 0) + 1;
      } else if (word === "ok") {
        var tile = row < 0 ? lead : rows[row].tiles[cols[row] || 0];
        var key = pair(tile.title);
        if (row < 0 && button === 0) playTile(tile);
        else go("/title/" + key.kind + "/" + key.id);
        return true;
      } else return false;
      place();
      return true;
    };
  };
})();
