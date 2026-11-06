// A page of rows. The parts draw, and this file only knows where the focus is.
(function () {
  "use strict";
  var tv = window.tv;
  var kit = tv.kit;
  var ROW_KINDS = { row: true, ranked: true, "continue": true, grid: true };

  function sectionsOf(data) {
    var found = { lead: null, rows: [] };
    for (var i = 0; i < data.sections.length; i += 1) {
      var section = data.sections[i];
      if (!section.tiles || !section.tiles.length) continue;
      if (section.kind === "hero") found.lead = section.tiles[0];
      else if (ROW_KINDS[section.kind]) found.rows.push(section);
    }
    if (!found.lead && found.rows.length) found.lead = found.rows[0].tiles[0];
    return found;
  }

  tv.surfaces.browse = function (root, data, nav, route, person) {
    if (!data) return null;
    var found = sectionsOf(data);
    var lead = found.lead;
    var rows = found.rows;
    if (!lead) {
      root.appendChild(kit.make("div", "message", "Nothing here yet."));
      return null;
    }
    var page = kit.make("div", "browse");
    var top = tv.parts.topbar(nav, route.screen || "home", person);
    var hero = tv.parts.hero(lead);
    var open = tv.parts.openRow();
    var next = tv.parts.nextRow();
    page.appendChild(top.node);
    page.appendChild(hero.node);
    page.appendChild(open.node);
    page.appendChild(next.node);
    root.appendChild(page);

    var where = "hero";
    var row = 0;
    var cols = [];
    var button = 0;
    var tab = top.home;

    function draw(direction) {
      var inRow = where === "row";
      top.draw(where === "top" ? tab : -1);
      hero.draw(!inRow, where === "hero" ? button : -1);
      open.draw(inRow, rows[row], cols[row] || 0, direction);
      next.draw(rows[inRow ? row + 1 : 0], !inRow);
    }

    function openTitle(tile) {
      var key = kit.pair(tile.title);
      kit.go("/title/" + key.kind + "/" + key.id);
    }

    function press(word) {
      var at = cols[row] || 0;
      var direction = 0;
      if (word === "up") {
        if (where === "row" && row > 0) {
          row -= 1;
          direction = -1;
        } else if (where === "row") where = "hero";
        else if (where === "hero") where = "top";
        else return false;
      } else if (word === "down") {
        if (where === "top") where = "hero";
        else if (where === "hero" && rows.length) {
          where = "row";
          row = 0;
        } else if (where === "row" && row < rows.length - 1) {
          row += 1;
          direction = 1;
        } else return false;
      } else if (word === "left") {
        if (where === "top" && tab > 0) tab -= 1;
        else if (where === "hero") button = 0;
        else if (where === "row" && at > 0) cols[row] = at - 1;
        else return false;
      } else if (word === "right") {
        if (where === "top" && tab < top.count - 1) tab += 1;
        else if (where === "hero") button = 1;
        else if (where === "row" && at < rows[row].tiles.length - 1) cols[row] = at + 1;
        else return false;
      } else return false;
      draw(direction);
      return true;
    }

    function choose() {
      if (where === "top") top.open(tab);
      else if (where === "row") openTitle(rows[row].tiles[cols[row] || 0]);
      else if (button === 1 || kit.pair(lead.title).kind === "tv") openTitle(lead);
      else kit.go("/watch/" + kit.pair(lead.title).kind + "/" + kit.pair(lead.title).id);
      return true;
    }

    draw();
    return function (word) {
      return word === "ok" ? choose() : press(word);
    };
  };
})();
