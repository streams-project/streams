// Rows of films. The top bar stays. The first thing under it is a rounded hero card, and below that
// one row owns the screen at a time: its focused title opens wide, the others stand as posters,
// and the title's facts sit under the row. A set keeps few elements alive however long the page is.
(function () {
  "use strict";
  var tv = window.tv;
  var kit = tv.kit;
  var make = kit.make;
  var POSTER_STEP = 328;
  var POSTERS_AHEAD = 5;
  var PEEK_POSTERS = 6;
  var TABS = 5;
  var DETAIL_WAIT_MS = 240;
  var ROW_KINDS = { row: true, ranked: true, "continue": true, grid: true };
  var details = {};

  function image(images, role) {
    for (var i = 0; images && i < images.length; i += 1) {
      if (images[i].role === role) return images[i].url;
    }
    return null;
  }

  function wide(tile) {
    return image(tile.images, "backdrop") || image(tile.images, "still") || image(tile.images, "poster");
  }

  function facts(detail) {
    var parts = [];
    if (detail.genres && detail.genres.length) parts.push(detail.genres[0]);
    if (detail.year) parts.push(String(detail.year));
    if (detail.seasons && detail.seasons.length) {
      parts.push(detail.seasons.length + (detail.seasons.length > 1 ? " Seasons" : " Season"));
    } else if (detail.runtime_min) {
      parts.push(Math.floor(detail.runtime_min / 60) + "h " + detail.runtime_min % 60 + "m");
    }
    return parts.join(" · ");
  }

  function tabAddress(item) {
    return item.screen === "home" ? "/home" : "/" + item.screen;
  }

  function poster(tile) {
    var node = make("div", "poster");
    var url = image(tile.images, "poster") || wide(tile);
    kit.art(node, url);
    if (!url) node.appendChild(make("span", "poster-name", tile.name));
    return node;
  }

  function top(nav, active) {
    var bar = make("div", "top");
    var line = make("div", "top-tabs");
    var tabs = [];
    bar.appendChild(make("span", "top-search"));
    for (var i = 0; i < nav.length; i += 1) {
      var tab = make("span", "top-tab", nav[i].label);
      line.appendChild(tab);
      tabs.push(tab);
    }
    bar.appendChild(line);
    var mark = make("img", "top-mark");
    mark.src = "/mark.svg";
    mark.alt = "";
    bar.appendChild(mark);
    var at = 0;
    for (var n = 0; n < nav.length; n += 1) {
      if (nav[n].screen === active) at = n;
    }
    return { bar: bar, tabs: tabs, at: at };
  }

  tv.surfaces.browse = function (root, screenData, navItems, route) {
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
    var nav = [];
    for (var m = 0; m < (navItems || []).length && nav.length < TABS; m += 1) {
      if (navItems[m].screen !== "genre") nav.push(navItems[m]);
    }

    var page = make("div", "browse");
    var bar = top(nav, route.screen || "home");
    page.appendChild(bar.bar);

    // The hero card.
    var card = make("div", "hero-card");
    var cardArt = make("div", "hero-card-art");
    kit.art(cardArt, wide(lead));
    card.appendChild(cardArt);
    card.appendChild(make("div", "hero-card-shade"));
    var words = make("div", "hero-card-text");
    words.appendChild(make("h1", "", lead.name));
    var heroFacts = make("p", "facts", "");
    var heroText = make("p", "overview", "");
    var playButton = make("span", "button large", "Play");
    var infoButton = make("span", "button large quiet", "More Info");
    words.appendChild(heroFacts);
    words.appendChild(heroText);
    words.appendChild(playButton);
    words.appendChild(infoButton);
    card.appendChild(words);
    page.appendChild(card);

    // The row that owns the screen.
    var now = make("div", "row-now off");
    var label = make("h2", "", "");
    var openCard = make("div", "open-card");
    var openArt = make("div", "open-art");
    var openShade = make("div", "open-shade");
    var openName = make("p", "open-name", "");
    var ring = make("div", "open-ring");
    openCard.appendChild(openArt);
    openCard.appendChild(openShade);
    openCard.appendChild(openName);
    openCard.appendChild(ring);
    var before = make("div", "poster-before");
    var strip = make("div", "posters");
    var rowFacts = make("p", "facts row-facts", "");
    var rowText = make("p", "overview row-text", "");
    now.appendChild(label);
    now.appendChild(before);
    now.appendChild(strip);
    now.appendChild(openCard);
    now.appendChild(rowFacts);
    now.appendChild(rowText);
    page.appendChild(now);

    // The next row, dim, cut by the edge of the screen.
    var next = make("div", "row-next");
    var nextLabel = make("h2", "", "");
    var nextStrip = make("div", "next-posters");
    next.appendChild(nextLabel);
    next.appendChild(nextStrip);
    page.appendChild(next);
    root.appendChild(page);

    var where = "hero";
    var row = 0;
    var cols = [];
    var button = 0;
    var tab = bar.at;
    var waiting = null;
    var held = {};
    var shownRow = -1;

    function ask(tile, done) {
      var key = kit.pair(tile.title);
      window.clearTimeout(waiting);
      if (details[tile.title]) {
        done(details[tile.title]);
        return;
      }
      waiting = window.setTimeout(function () {
        kit.ask("GET", "/v1/titles/" + key.kind + "/" + key.id, function (status, text) {
          if (status < 200 || status >= 300) return;
          details[tile.title] = JSON.parse(text);
          done(details[tile.title]);
        });
      }, DETAIL_WAIT_MS);
    }

    function peek(index) {
      kit.empty(nextStrip);
      if (index >= rows.length) {
        next.className = "row-next off";
        return;
      }
      next.className = where === "row" ? "row-next" : "row-next high";
      nextLabel.textContent = rows[index].title || "";
      var tiles = rows[index].tiles;
      for (var i = 0; i < tiles.length && i < PEEK_POSTERS; i += 1) {
        var node = poster(tiles[i]);
        node.style.left = i * POSTER_STEP + "px";
        nextStrip.appendChild(node);
      }
    }

    function fillRow() {
      var tiles = rows[row].tiles;
      var c = cols[row] || 0;
      if (shownRow !== row) {
        kit.empty(strip);
        held = {};
        shownRow = row;
      }
      label.textContent = rows[row].title || "";
      for (var kept in held) {
        if (Number(kept) <= c || Number(kept) > c + POSTERS_AHEAD) {
          strip.removeChild(held[kept]);
          delete held[kept];
        }
      }
      for (var i = c + 1; i <= c + POSTERS_AHEAD && i < tiles.length; i += 1) {
        if (!held[i]) {
          held[i] = poster(tiles[i]);
          held[i].style.left = i * POSTER_STEP + "px";
          strip.appendChild(held[i]);
        }
      }
      strip.style.transform = "translateX(" + -(c + 1) * POSTER_STEP + "px)";
      kit.empty(before);
      if (c > 0) before.appendChild(poster(tiles[c - 1]));
      var tile = tiles[c];
      kit.art(openArt, wide(tile));
      openName.textContent = tile.name;
      rowFacts.textContent = tile.tags ? tile.tags.join(" · ") : "";
      rowText.textContent = "";
      ask(tile, function (detail) {
        if (rows[row].tiles[cols[row] || 0] !== tile) return;
        rowFacts.textContent = facts(detail);
        rowText.textContent = detail.overview || "";
      });
    }

    function draw() {
      for (var i = 0; i < bar.tabs.length; i += 1) {
        var name = "top-tab";
        if (i === bar.at) name += " active";
        if (where === "top" && i === tab) name += " on";
        bar.tabs[i].className = name;
      }
      var inRow = where === "row";
      card.className = inRow ? "hero-card off" : "hero-card";
      now.className = inRow ? "row-now" : "row-now off";
      playButton.className = where === "hero" && button === 0 ? "button large on" : "button large";
      infoButton.className = where === "hero" && button === 1 ? "button large quiet on" : "button large quiet";
      if (inRow) fillRow();
      peek(inRow ? row + 1 : 0);
    }

    function openTitle(tile) {
      var key = kit.pair(tile.title);
      kit.go("/title/" + key.kind + "/" + key.id);
    }

    ask(lead, function (detail) {
      heroFacts.textContent = facts(detail);
      heroText.textContent = detail.overview || "";
    });
    draw();

    return function (word) {
      if (word === "up") {
        if (where === "row" && row > 0) row -= 1;
        else if (where === "row") where = "hero";
        else if (where === "hero") where = "top";
        else return false;
      } else if (word === "down") {
        if (where === "top") where = "hero";
        else if (where === "hero" && rows.length) {
          where = "row";
          row = 0;
        } else if (where === "row" && row < rows.length - 1) row += 1;
        else return false;
      } else if (word === "left") {
        if (where === "top" && tab > 0) tab -= 1;
        else if (where === "hero") button = 0;
        else if (where === "row" && (cols[row] || 0) > 0) cols[row] = (cols[row] || 0) - 1;
        else return false;
      } else if (word === "right") {
        if (where === "top" && tab < bar.tabs.length - 1) tab += 1;
        else if (where === "hero") button = 1;
        else if (where === "row" && (cols[row] || 0) < rows[row].tiles.length - 1) cols[row] = (cols[row] || 0) + 1;
        else return false;
      } else if (word === "ok") {
        if (where === "top") {
          kit.go(tabAddress(nav[tab]));
        } else if (where === "row") {
          openTitle(rows[row].tiles[cols[row] || 0]);
        } else if (button === 1 || kit.pair(lead.title).kind === "tv") {
          openTitle(lead);
        } else {
          kit.go("/watch/" + kit.pair(lead.title).kind + "/" + kit.pair(lead.title).id);
        }
        return true;
      } else return false;
      draw();
      return true;
    };
  };
})();
