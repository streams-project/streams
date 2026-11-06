// Who is watching: a column of profiles on the left, and beside it a title chosen for the one in
// focus. The PIN goes to the core and is never kept.
(function () {
  "use strict";
  var tv = window.tv;
  var kit = tv.kit;
  var make = kit.make;
  var ART_WAIT_MS = 300;
  var FADE_MS = 500;
  var LARGE = 194;
  var SMALL = 130;

  function pin(root, person, leave) {
    kit.empty(root);
    var digits = [0, 0, 0, 0];
    var at = 0;
    var panel = make("div", "who-pin");
    panel.appendChild(tv.parts.avatar(person, LARGE));
    panel.appendChild(make("h1", "", person.name));
    panel.appendChild(make("p", "", "Enter the PIN. Up and down change a number."));
    var row = make("div", "pin");
    var cells = [];
    for (var i = 0; i < digits.length; i += 1) {
      cells.push(make("span", "pin-digit", "0"));
      row.appendChild(cells[i]);
    }
    panel.appendChild(row);
    root.appendChild(panel);
    function mark() {
      for (var n = 0; n < cells.length; n += 1) {
        cells[n].textContent = String(digits[n]);
        cells[n].className = n === at ? "pin-digit on" : "pin-digit";
      }
    }
    mark();
    return function (word) {
      if (word === "left" && at > 0) at -= 1;
      else if (word === "right" && at < digits.length - 1) at += 1;
      else if (word === "up") digits[at] = (digits[at] + 1) % 10;
      else if (word === "down") digits[at] = (digits[at] + 9) % 10;
      else if (/^[0-9]$/.test(word)) {
        digits[at] = Number(word);
        if (at < digits.length - 1) at += 1;
      } else if (word === "ok") {
        kit.act({ action: "enter_profile", profile: person.id, pin: digits.join("") });
        return true;
      } else if (word === "back") {
        leave();
        return true;
      } else return false;
      mark();
      return true;
    };
  }

  // One title beside the column, crossfading as the focus moves.
  function feature(page) {
    var kept = {};
    var waiting = null;
    var layer = null;
    var name = make("p", "who-title");
    var tags = make("p", "who-tags");
    page.appendChild(name);
    page.appendChild(tags);

    function show(tile) {
      var next = make("div", "who-art incoming");
      kit.art(next, tv.parts.wide(tile));
      page.insertBefore(next, page.firstChild);
      window.requestAnimationFrame(function () {
        window.requestAnimationFrame(function () { next.className = "who-art"; });
      });
      var old = layer;
      layer = next;
      if (old) {
        old.className = "who-art leaving";
        window.setTimeout(function () { if (old.parentNode) old.parentNode.removeChild(old); }, FADE_MS);
      }
      name.textContent = tile.name;
      tags.textContent = tile.tags ? tile.tags.join(" · ") : "";
    }

    return function (person) {
      window.clearTimeout(waiting);
      if (!person) return;
      if (kept[person.id]) {
        show(kept[person.id]);
        return;
      }
      waiting = window.setTimeout(function () {
        kit.ask("GET", "/v1/profiles/" + person.id + "/feature", function (status, text) {
          if (status < 200 || status >= 300) return;
          kept[person.id] = JSON.parse(text);
          show(kept[person.id]);
        });
      }, ART_WAIT_MS);
    };
  }

  tv.surfaces.people = function people(root, list, setKeys) {
    var at = 0;
    var page = make("div", "who");
    var shade = make("div", "who-shade");
    page.appendChild(shade);
    var head = make("div", "who-head");
    var mark = make("img", "who-mark");
    mark.src = "/mark.svg";
    mark.alt = "";
    head.appendChild(mark);
    head.appendChild(make("p", "", "Choose a Profile"));
    page.appendChild(head);
    var column = make("div", "who-column");
    var rows = [];
    for (var i = 0; i < list.length; i += 1) {
      var row = make("div", "who-row");
      row.appendChild(make("span", "who-pencil"));
      row.lastChild.appendChild(tv.parts.icon("pencil", 30));
      var face = tv.parts.avatar(list[i], SMALL);
      row.appendChild(face);
      row.appendChild(make("span", "who-name", list[i].name));
      column.appendChild(row);
      rows.push({ node: row, face: face });
    }
    page.appendChild(column);
    root.appendChild(page);
    var showArt = feature(page);

    function draw() {
      for (var n = 0; n < rows.length; n += 1) {
        var on = n === at;
        rows[n].node.className = on ? "who-row on" : "who-row";
        var size = on ? LARGE : SMALL;
        rows[n].face.style.width = size + "px";
        rows[n].face.style.height = size + "px";
        rows[n].face.style.lineHeight = size + "px";
        rows[n].face.style.fontSize = Math.round(size * 0.46) + "px";
      }
      showArt(list[at]);
    }
    draw();

    return function (word) {
      if (word === "up" && at > 0) at -= 1;
      else if (word === "down" && at < rows.length - 1) at += 1;
      else if (word === "ok" && list[at]) {
        if (list[at].has_pin) {
          setKeys(pin(root, list[at], function () {
            kit.empty(root);
            setKeys(people(root, list, setKeys));
          }));
        } else {
          kit.act({ action: "enter_profile", profile: list[at].id });
        }
        return true;
      } else return false;
      draw();
      return true;
    };
  };
})();
